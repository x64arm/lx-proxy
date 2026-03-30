// 认证模块测试

mod common;

use serde::{Deserialize, Serialize};
use common::{post_request, get_request, create_test_app_state, create_test_app, run_migrations, cleanup_test_data};

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    user: UserResponse,
}

#[derive(Deserialize)]
struct UserResponse {
    id: String,
    username: String,
    email: Option<String>,
    role: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[tokio::test]
async fn test_login_success() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建测试用户
    create_test_user(&pool, "testuser", "password123").await;

    // 尝试登录
    let login_req = LoginRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let (status, response): (StatusCode, ApiResponse<LoginResponse>) = 
        post_request(&mut app, "/api/auth/login", &login_req).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(!response.data.token.is_empty());
    assert_eq!(response.data.user.username, "testuser");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建测试用户
    create_test_user(&pool, "testuser", "password123").await;

    // 尝试使用错误密码登录
    let login_req = LoginRequest {
        username: "testuser".to_string(),
        password: "wrongpassword".to_string(),
    };

    let (status, _response): (StatusCode, serde_json::Value) = 
        post_request(&mut app, "/api/auth/login", &login_req).await;

    assert_eq!(status, 401);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 尝试登录不存在的用户
    let login_req = LoginRequest {
        username: "nonexistent".to_string(),
        password: "password123".to_string(),
    };

    let (status, _response): (StatusCode, serde_json::Value) = 
        post_request(&mut app, "/api/auth/login", &login_req).await;

    assert_eq!(status, 401);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_current_user() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建测试用户并登录
    create_test_user(&pool, "testuser", "password123").await;
    
    let login_req = LoginRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let (_, login_response): (_, ApiResponse<LoginResponse>) = 
        post_request(&mut app, "/api/auth/login", &login_req).await;

    let token = &login_response.data.token;

    // 获取当前用户信息
    let (status, response): (StatusCode, ApiResponse<UserResponse>) = 
        get_request(&mut app, "/api/auth/me", Some(token)).await;

    assert_eq!(status, 200);
    assert_eq!(response.data.username, "testuser");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_current_user_unauthorized() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 未授权访问
    let (status, _response): (StatusCode, serde_json::Value) = 
        get_request(&mut app, "/api/auth/me", None).await;

    assert_eq!(status, 401);

    cleanup_test_data(&pool).await;
}

/// 创建测试用户
async fn create_test_user(pool: &sqlx::PgPool, username: &str, password: &str) {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query(
        "INSERT INTO users (username, password_hash, role, email) 
         VALUES ($1, $2, $3, $4)"
    )
    .bind(username)
    .bind(&password_hash)
    .bind("admin")
    .bind(Some("test@example.com"))
    .execute(pool)
    .await
    .unwrap();
}

use axum::http::StatusCode;
