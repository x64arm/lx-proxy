// 入站配置管理模块测试

mod common;

use serde::{Deserialize, Serialize};
use common::{post_request, get_request, put_request, delete_request, create_test_app_state, create_test_app, run_migrations, cleanup_test_data};
use axum::http::StatusCode;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize)]
struct CreateInboundRequest {
    name: String,
    protocol: String,
    port: i32,
    settings: serde_json::Value,
    enable: bool,
    expiry_time: Option<i64>,
    total_gb: Option<i64>,
}

#[derive(Deserialize)]
struct InboundResponse {
    id: String,
    name: String,
    protocol: String,
    port: i32,
    enable: bool,
    traffic_used: i64,
    traffic_total: Option<i64>,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[derive(Deserialize)]
struct InboundsListResponse {
    inbounds: Vec<InboundResponse>,
    total: i64,
}

/// 获取管理员 token
async fn get_admin_token(app: &mut axum::Router) -> String {
    create_admin_user(&app.clone()).await;
    
    let login_req = LoginRequest {
        username: "admin".to_string(),
        password: "admin123".to_string(),
    };

    let (_, login_response): (_, ApiResponse<LoginResponse>) = 
        post_request(app, "/api/auth/login", &login_req).await;

    login_response.data.token
}

/// 创建初始管理员
async fn create_admin_user(app: &axum::Router) {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    use crate::create_test_pool;
    
    let pool = create_test_pool().await;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password("admin123".as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query(
        "INSERT INTO users (username, password_hash, role, email) 
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (username) DO NOTHING"
    )
    .bind("admin")
    .bind(&password_hash)
    .bind("admin")
    .bind(Some("admin@example.com"))
    .execute(&pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_create_inbound() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建入站配置
    let create_req = CreateInboundRequest {
        name: "Test VMess".to_string(),
        protocol: "vmess".to_string(),
        port: 10080,
        settings: serde_json::json!({
            "clients": [
                {
                    "id": "b8a7c6d5-e4f3-2a1b-0c9d-8e7f6a5b4c3d",
                    "email": "user1@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: Some(100 * 1024 * 1024 * 1024), // 100GB
    };

    let (status, response): (StatusCode, ApiResponse<InboundResponse>) = 
        post_request(&mut app, "/api/inbounds", &create_req, &token).await;

    assert_eq!(status, 201);
    assert!(response.success);
    assert_eq!(response.data.name, "Test VMess");
    assert_eq!(response.data.protocol, "vmess");
    assert_eq!(response.data.port, 10080);
    assert!(response.data.enable);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_inbounds() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建多个入站配置
    for i in 0..3 {
        let create_req = CreateInboundRequest {
            name: format!("Inbound {}", i),
            protocol: "vmess".to_string(),
            port: 10080 + i,
            settings: serde_json::json!({
                "clients": [
                    {
                        "id": format!("b8a7c6d5-e4f3-2a1b-0c9d-8e7f6a5b4c{:02x}", i),
                        "email": format!("user{}@example.com", i)
                    }
                ]
            }),
            enable: true,
            expiry_time: None,
            total_gb: None,
        };

        let _: (StatusCode, ApiResponse<InboundResponse>) = 
            post_request(&mut app, "/api/inbounds", &create_req, &token).await;
    }

    // 获取入站列表
    let (status, response): (StatusCode, ApiResponse<InboundsListResponse>) = 
        get_request(&mut app, "/api/inbounds", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert_eq!(response.data.total, 3);
    assert_eq!(response.data.inbounds.len(), 3);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_inbound_by_id() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建入站配置
    let create_req = CreateInboundRequest {
        name: "Get Test".to_string(),
        protocol: "vless".to_string(),
        port: 10090,
        settings: serde_json::json!({
            "clients": [
                {
                    "id": "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d",
                    "email": "gettest@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: None,
    };

    let (_, create_response): (_, ApiResponse<InboundResponse>) = 
        post_request(&mut app, "/api/inbounds", &create_req, &token).await;

    let inbound_id = &create_response.data.id;

    // 获取入站详情
    let (status, response): (StatusCode, ApiResponse<InboundResponse>) = 
        get_request(&mut app, &format!("/api/inbounds/{}", inbound_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert_eq!(response.data.name, "Get Test");
    assert_eq!(response.data.protocol, "vless");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_inbound() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建入站配置
    let create_req = CreateInboundRequest {
        name: "Update Test".to_string(),
        protocol: "vmess".to_string(),
        port: 10100,
        settings: serde_json::json!({
            "clients": [
                {
                    "id": "c1d2e3f4-a5b6-7c8d-9e0f-1a2b3c4d5e6f",
                    "email": "updatetest@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: Some(50 * 1024 * 1024 * 1024), // 50GB
    };

    let (_, create_response): (_, ApiResponse<InboundResponse>) = 
        post_request(&mut app, "/api/inbounds", &create_req, &token).await;

    let inbound_id = &create_response.data.id;

    // 更新入站配置
    #[derive(Serialize)]
    struct UpdateInboundRequest {
        name: String,
        enable: bool,
        total_gb: Option<i64>,
    }

    let update_req = UpdateInboundRequest {
        name: "Updated Name".to_string(),
        enable: false,
        total_gb: Some(200 * 1024 * 1024 * 1024), // 200GB
    };

    let (status, response): (StatusCode, ApiResponse<InboundResponse>) = 
        put_request(&mut app, &format!("/api/inbounds/{}", inbound_id), &update_req, &token).await;

    assert_eq!(status, 200);
    assert_eq!(response.data.name, "Updated Name");
    assert!(!response.data.enable);
    assert_eq!(response.data.traffic_total, Some(200 * 1024 * 1024 * 1024));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_inbound() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建入站配置
    let create_req = CreateInboundRequest {
        name: "Delete Test".to_string(),
        protocol: "trojan".to_string(),
        port: 10110,
        settings: serde_json::json!({
            "clients": [
                {
                    "password": "testpassword123",
                    "email": "deletetest@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: None,
    };

    let (_, create_response): (_, ApiResponse<InboundResponse>) = 
        post_request(&mut app, "/api/inbounds", &create_req, &token).await;

    let inbound_id = &create_response.data.id;

    // 删除入站配置
    let (status, _response): (StatusCode, serde_json::Value) = 
        delete_request(&mut app, &format!("/api/inbounds/{}", inbound_id), &token).await;

    assert_eq!(status, 200);

    // 验证已被删除
    let (status, _response): (StatusCode, serde_json::Value) = 
        get_request(&mut app, &format!("/api/inbounds/{}", inbound_id), Some(&token)).await;

    assert_eq!(status, 404);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_reset_inbound_traffic() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建入站配置
    let create_req = CreateInboundRequest {
        name: "Reset Test".to_string(),
        protocol: "vmess".to_string(),
        port: 10120,
        settings: serde_json::json!({
            "clients": [
                {
                    "id": "d1e2f3a4-b5c6-7d8e-9f0a-1b2c3d4e5f6a",
                    "email": "resettest@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: None,
    };

    let (_, create_response): (_, ApiResponse<InboundResponse>) = 
        post_request(&mut app, "/api/inbounds", &create_req, &token).await;

    let inbound_id = &create_response.data.id;

    // 重置流量
    let (status, _response): (StatusCode, serde_json::Value) = 
        post_request(&mut app, &format!("/api/inbounds/{}/reset", inbound_id), &serde_json::json!({}), &token).await;

    assert_eq!(status, 200);

    cleanup_test_data(&pool).await;
}
