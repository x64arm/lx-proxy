// 用户管理模块测试

mod common;

use serde::{Deserialize, Serialize};
use common::{post_request, get_request, put_request, delete_request, create_test_app_state, create_test_app, run_migrations, cleanup_test_data};
use axum::http::StatusCode;
use sqlx::{PgPool, Row};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;

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
struct CreateUserRequest {
    username: String,
    password: String,
    email: Option<String>,
    role: String,
}

#[derive(Deserialize)]
struct UserResponse {
    id: String,
    username: String,
    email: Option<String>,
    role: String,
    created_at: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[derive(Deserialize)]
struct UsersListResponse {
    users: Vec<UserResponse>,
    total: i64,
}

/// 获取管理员 token
async fn get_admin_token(app: &mut axum::Router) -> String {
    let login_req = LoginRequest {
        username: "admin".to_string(),
        password: "admin123".to_string(),
    };

    let (_, login_response): (_, ApiResponse<LoginResponse>) = 
        post_request(app, "/api/auth/login", &login_req).await;

    login_response.data.token
}

#[tokio::test]
async fn test_create_user() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建初始管理员
    create_admin_user(&pool).await;
    
    let token = get_admin_token(&mut app).await;

    // 创建新用户
    let create_req = CreateUserRequest {
        username: "newuser".to_string(),
        password: "password123".to_string(),
        email: Some("newuser@example.com".to_string()),
        role: "user".to_string(),
    };

    let (status, response): (StatusCode, ApiResponse<UserResponse>) = 
        post_request(&mut app, "/api/users", &create_req, &token).await;

    assert_eq!(status, 201);
    assert!(response.success);
    assert_eq!(response.data.username, "newuser");
    assert_eq!(response.data.email, Some("newuser@example.com".to_string()));
    assert_eq!(response.data.role, "user");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_user_duplicate_username() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    create_admin_user(&pool).await;
    let token = get_admin_token(&mut app).await;

    // 创建第一个用户
    let create_req = CreateUserRequest {
        username: "duplicate".to_string(),
        password: "password123".to_string(),
        email: Some("user1@example.com".to_string()),
        role: "user".to_string(),
    };

    let _: (StatusCode, ApiResponse<UserResponse>) = 
        post_request(&mut app, "/api/users", &create_req, &token).await;

    // 尝试创建重复用户名的用户
    let create_req2 = CreateUserRequest {
        username: "duplicate".to_string(),
        password: "password456".to_string(),
        email: Some("user2@example.com".to_string()),
        role: "user".to_string(),
    };

    let (status, _response): (StatusCode, serde_json::Value) = 
        post_request(&mut app, "/api/users", &create_req2, &token).await;

    assert_eq!(status, 409); // Conflict

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_list_users() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    create_admin_user(&pool).await;
    let token = get_admin_token(&mut app).await;

    // 创建几个测试用户
    for i in 0..3 {
        let create_req = CreateUserRequest {
            username: format!("user{}", i),
            password: "password123".to_string(),
            email: Some(format!("user{}@example.com", i)),
            role: "user".to_string(),
        };

        let _: (StatusCode, ApiResponse<UserResponse>) = 
            post_request(&mut app, "/api/users", &create_req, &token).await;
    }

    // 获取用户列表
    let (status, response): (StatusCode, ApiResponse<UsersListResponse>) = 
        get_request(&mut app, "/api/users", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert_eq!(response.data.total, 4); // admin + 3 users
    assert_eq!(response.data.users.len(), 4);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_user_by_id() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    create_admin_user(&pool).await;
    let token = get_admin_token(&mut app).await;

    // 创建测试用户
    let create_req = CreateUserRequest {
        username: "getuser".to_string(),
        password: "password123".to_string(),
        email: Some("getuser@example.com".to_string()),
        role: "user".to_string(),
    };

    let (_, create_response): (_, ApiResponse<UserResponse>) = 
        post_request(&mut app, "/api/users", &create_req, &token).await;

    let user_id = &create_response.data.id;

    // 获取用户详情
    let (status, response): (StatusCode, ApiResponse<UserResponse>) = 
        get_request(&mut app, &format!("/api/users/{}", user_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert_eq!(response.data.username, "getuser");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_user() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    create_admin_user(&pool).await;
    let token = get_admin_token(&mut app).await;

    // 创建测试用户
    let create_req = CreateUserRequest {
        username: "updateuser".to_string(),
        password: "password123".to_string(),
        email: Some("old@example.com".to_string()),
        role: "user".to_string(),
    };

    let (_, create_response): (_, ApiResponse<UserResponse>) = 
        post_request(&mut app, "/api/users", &create_req, &token).await;

    let user_id = &create_response.data.id;

    // 更新用户
    #[derive(Serialize)]
    struct UpdateUserRequest {
        email: Option<String>,
        role: String,
    }

    let update_req = UpdateUserRequest {
        email: Some("new@example.com".to_string()),
        role: "admin".to_string(),
    };

    let (status, response): (StatusCode, ApiResponse<UserResponse>) = 
        put_request(&mut app, &format!("/api/users/{}", user_id), &update_req, &token).await;

    assert_eq!(status, 200);
    assert_eq!(response.data.email, Some("new@example.com".to_string()));
    assert_eq!(response.data.role, "admin");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_user() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    create_admin_user(&pool).await;
    let token = get_admin_token(&mut app).await;

    // 创建测试用户
    let create_req = CreateUserRequest {
        username: "deleteuser".to_string(),
        password: "password123".to_string(),
        email: Some("delete@example.com".to_string()),
        role: "user".to_string(),
    };

    let (_, create_response): (_, ApiResponse<UserResponse>) = 
        post_request(&mut app, "/api/users", &create_req, &token).await;

    let user_id = &create_response.data.id;

    // 删除用户
    let (status, _response): (StatusCode, serde_json::Value) = 
        delete_request(&mut app, &format!("/api/users/{}", user_id), &token).await;

    assert_eq!(status, 200);

    // 验证用户已被删除
    let (status, _response): (StatusCode, serde_json::Value) = 
        get_request(&mut app, &format!("/api/users/{}", user_id), Some(&token)).await;

    assert_eq!(status, 404);

    cleanup_test_data(&pool).await;
}

/// 创建初始管理员
async fn create_admin_user(pool: &sqlx::PgPool) {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password("admin123".as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query(
        "INSERT INTO users (username, password_hash, role, email) 
         VALUES ($1, $2, $3, $4)"
    )
    .bind("admin")
    .bind(&password_hash)
    .bind("admin")
    .bind(Some("admin@example.com"))
    .execute(pool)
    .await
    .unwrap();
}
