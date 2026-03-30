// 流量统计模块测试

mod common;

use serde::{Deserialize, Serialize};
use common::{post_request, get_request, create_test_app_state, create_test_app, run_migrations, cleanup_test_data};
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
}

#[derive(Deserialize)]
struct TrafficStatsResponse {
    total_upload: i64,
    total_download: i64,
    total_traffic: i64,
    records: Vec<TrafficRecord>,
}

#[derive(Deserialize)]
struct TrafficRecord {
    date: String,
    upload: i64,
    download: i64,
    total: i64,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
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

/// 创建测试入站配置
async fn create_test_inbound(
    app: &mut axum::Router,
    token: &str,
    name: &str,
    port: i32,
) -> String {
    let create_req = CreateInboundRequest {
        name: name.to_string(),
        protocol: "vmess".to_string(),
        port,
        settings: serde_json::json!({
            "clients": [
                {
                    "id": "e1f2a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
                    "email": "traffic@example.com"
                }
            ]
        }),
        enable: true,
        expiry_time: None,
        total_gb: None,
    };

    let (_, create_response): (_, ApiResponse<InboundResponse>) = 
        post_request(app, "/api/inbounds", &create_req, token).await;

    create_response.data.id
}

#[tokio::test]
async fn test_get_traffic_stats() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建测试入站
    let inbound_id = create_test_inbound(&mut app, &token, "Traffic Test", 10200).await;

    // 插入测试流量数据
    insert_test_traffic_data(&pool, &inbound_id).await;

    // 获取流量统计
    let (status, response): (StatusCode, ApiResponse<TrafficStatsResponse>) = 
        get_request(&mut app, "/api/traffic", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(response.data.total_traffic > 0);
    assert!(!response.data.records.is_empty());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_traffic_by_inbound() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建测试入站
    let inbound_id = create_test_inbound(&mut app, &token, "Single Inbound", 10210).await;

    // 插入测试流量数据
    insert_test_traffic_data(&pool, &inbound_id).await;

    // 获取指定入站的流量
    let (status, response): (StatusCode, ApiResponse<TrafficStatsResponse>) = 
        get_request(&mut app, &format!("/api/traffic/{}", inbound_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(response.data.total_traffic > 0);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_traffic_summary() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建多个测试入站
    for i in 0..3 {
        let inbound_id = create_test_inbound(&mut app, &token, &format!("Summary Test {}", i), 10220 + i).await;
        insert_test_traffic_data(&pool, &inbound_id).await;
    }

    // 获取流量汇总
    #[derive(Deserialize)]
    struct TrafficSummary {
        total_upload: i64,
        total_download: i64,
        total_traffic: i64,
        active_inbounds: i64,
    }

    let (status, response): (StatusCode, ApiResponse<TrafficSummary>) = 
        get_request(&mut app, "/api/traffic/summary", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(response.data.total_traffic > 0);
    assert_eq!(response.data.active_inbounds, 3);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_log_traffic() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建测试入站
    let inbound_id = create_test_inbound(&mut app, &token, "Log Test", 10230).await;

    // 记录流量
    #[derive(Serialize)]
    struct LogTrafficRequest {
        inbound_id: String,
        upload: i64,
        download: i64,
    }

    let log_req = LogTrafficRequest {
        inbound_id: inbound_id.clone(),
        upload: 1024 * 1024 * 50, // 50MB
        download: 1024 * 1024 * 100, // 100MB
    };

    let (status, _response): (StatusCode, serde_json::Value) = 
        post_request(&mut app, "/api/traffic/log", &log_req, &token).await;

    assert_eq!(status, 201);

    // 验证流量已记录
    let (status, response): (StatusCode, ApiResponse<TrafficStatsResponse>) = 
        get_request(&mut app, &format!("/api/traffic/{}", inbound_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.data.total_traffic >= 150 * 1024 * 1024); // 至少 150MB

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_traffic_with_date_range() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let token = get_admin_token(&mut app).await;

    // 创建测试入站
    let inbound_id = create_test_inbound(&mut app, &token, "Date Range Test", 10240).await;

    // 插入测试流量数据
    insert_test_traffic_data(&pool, &inbound_id).await;

    // 带日期范围查询
    let (status, response): (StatusCode, ApiResponse<TrafficStatsResponse>) = 
        get_request(&mut app, &format!("/api/traffic/{}?start_date=2026-01-01&end_date=2026-12-31", inbound_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(!response.data.records.is_empty());

    cleanup_test_data(&pool).await;
}

/// 插入测试流量数据
async fn insert_test_traffic_data(pool: &sqlx::PgPool, inbound_id: &str) {
    // 插入最近 7 天的流量数据
    for i in 0..7 {
        let date = chrono::Utc::now() - chrono::Duration::days(i);
        let upload = 1024 * 1024 * 100 * (i as i64 + 1); // 100MB * (i+1)
        let download = 1024 * 1024 * 200 * (i as i64 + 1); // 200MB * (i+1)

        sqlx::query(
            "INSERT INTO traffic_logs (inbound_id, recorded_at, upload, download) 
             VALUES ($1, $2, $3, $4)"
        )
        .bind(inbound_id)
        .bind(date)
        .bind(upload)
        .bind(download)
        .execute(pool)
        .await
        .unwrap();
    }
}
