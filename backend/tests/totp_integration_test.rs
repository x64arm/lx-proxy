/// TOTP 双因素认证集成测试
/// 测试 TOTP 设置、验证、备用代码等完整流程

mod common;

use common::{create_test_app_state, create_test_app, run_migrations, cleanup_test_data, post_request, get_request};
use lx_proxy_backend::totp;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

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
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// 创建测试用户并返回用户 ID 和 token
async fn setup_test_user(app: &mut axum::Router) -> (Uuid, String) {
    let (_state, pool) = create_test_app_state().await;
    run_migrations(&pool).await;

    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password("password123".as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, email) 
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user_id)
    .bind("totp_test_user")
    .bind(&password_hash)
    .bind("user")
    .bind(Some("totp_test@example.com"))
    .execute(&pool)
    .await
    .unwrap();

    // 登录获取 token
    let login_req = LoginRequest {
        username: "totp_test_user".to_string(),
        password: "password123".to_string(),
    };

    let (_, login_response): (_, ApiResponse<LoginResponse>) =
        post_request(app, "/api/auth/login", &login_req).await;

    let token = login_response.data.unwrap().token;
    let user_id = Uuid::parse_str(&login_response.data.unwrap().user.id).unwrap();

    (user_id, token)
}

#[tokio::test]
async fn test_totp_setup_flow() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (user_id, token) = setup_test_user(&mut app).await;

    // 1. 初始化 TOTP 设置
    let setup_req = json!({
        "user_id": user_id.to_string()
    });

    let (status, response): (StatusCode, ApiResponse<totp::SetupTotpResponse>) =
        post_request(&mut app, "/api/totp/setup", &setup_req).await;

    assert_eq!(status, 200);
    assert!(response.success);
    
    let setup_data = response.data.unwrap();
    assert!(!setup_data.secret.is_empty());
    assert!(setup_data.qr_code_url.starts_with("otpauth://totp/"));
    assert_eq!(setup_data.backup_codes.len(), 10);

    // 验证备用代码格式（8 位数字）
    for code in &setup_data.backup_codes {
        assert_eq!(code.len(), 8, "Backup code should be 8 digits");
        assert!(code.chars().all(|c| c.is_ascii_digit()), "Backup code should be all digits");
    }

    // 2. 检查 TOTP 状态（应该是未启用）
    let (status, status_response): (StatusCode, serde_json::Value) =
        get_request(&mut app, &format!("/api/totp/{}/status", user_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert_eq!(status_response["enabled"], false);
    assert_eq!(status_response["verified"], false);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_totp_enable_and_verify() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (user_id, token) = setup_test_user(&mut app).await;

    // 1. 设置 TOTP
    let setup_req = json!({
        "user_id": user_id.to_string()
    });

    let (_, setup_response): (_, ApiResponse<totp::SetupTotpResponse>) =
        post_request(&mut app, "/api/totp/setup", &setup_req).await;

    let secret = setup_response.data.unwrap().secret;

    // 2. 生成当前 TOTP 代码（使用 totp-rs 库）
    use totp_rs::{Algorithm, TOTP};
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
    ).expect("Failed to create TOTP");
    
    let code = totp.generate_current().unwrap();

    // 3. 验证并启用 TOTP
    let verify_req = json!({
        "user_id": user_id.to_string(),
        "code": code
    });

    let (status, response): (StatusCode, ApiResponse<totp::VerifyTotpResponse>) =
        post_request(&mut app, &format!("/api/totp/{}/verify", user_id), &verify_req).await;

    assert_eq!(status, 200);
    assert!(response.success);

    // 4. 检查状态（应该已启用）
    let (status, status_response): (StatusCode, serde_json::Value) =
        get_request(&mut app, &format!("/api/totp/{}/status", user_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert_eq!(status_response["enabled"], true);
    assert_eq!(status_response["verified"], true);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_totp_invalid_code() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (user_id, _token) = setup_test_user(&mut app).await;

    // 1. 设置 TOTP
    let setup_req = json!({
        "user_id": user_id.to_string()
    });

    let (_, setup_response): (_, ApiResponse<totp::SetupTotpResponse>) =
        post_request(&mut app, "/api/totp/setup", &setup_req).await;

    // 2. 使用错误的验证码尝试启用
    let verify_req = json!({
        "user_id": user_id.to_string(),
        "code": "000000" // 错误的代码
    });

    let (status, response): (StatusCode, ApiResponse<totp::VerifyTotpResponse>) =
        post_request(&mut app, &format!("/api/totp/{}/verify", user_id), &verify_req).await;

    assert_eq!(status, 200);
    assert!(!response.success);
    assert_eq!(response.data.unwrap().message, "验证码错误");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_totp_disable() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (user_id, token) = setup_test_user(&mut app).await;

    // 1. 设置并启用 TOTP
    let setup_req = json!({
        "user_id": user_id.to_string()
    });

    let (_, setup_response): (_, ApiResponse<totp::SetupTotpResponse>) =
        post_request(&mut app, "/api/totp/setup", &setup_req).await;

    let secret = setup_response.data.unwrap().secret;
    
    use totp_rs::{Algorithm, TOTP};
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
    ).expect("Failed to create TOTP");
    
    let code = totp.generate_current().unwrap();

    let verify_req = json!({
        "user_id": user_id.to_string(),
        "code": code
    });

    let _: (StatusCode, ApiResponse<totp::VerifyTotpResponse>) =
        post_request(&mut app, &format!("/api/totp/{}/verify", user_id), &verify_req).await;

    // 2. 禁用 TOTP
    let disable_req = json!({
        "password": "password123"
    });

    let (status, response): (StatusCode, ApiResponse<totp::VerifyTotpResponse>) =
        post_request(&mut app, &format!("/api/totp/{}/disable", user_id), &disable_req).await;

    assert_eq!(status, 200);
    assert!(response.success);

    // 3. 检查状态（应该已禁用）
    let (status, status_response): (StatusCode, serde_json::Value) =
        get_request(&mut app, &format!("/api/totp/{}/status", user_id), Some(&token)).await;

    assert_eq!(status, 200);
    assert_eq!(status_response["enabled"], false);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_backup_code_login() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (user_id, _token) = setup_test_user(&mut app).await;

    // 1. 设置 TOTP
    let setup_req = json!({
        "user_id": user_id.to_string()
    });

    let (_, setup_response): (_, ApiResponse<totp::SetupTotpResponse>) =
        post_request(&mut app, "/api/totp/setup", &setup_req).await;

    let setup_data = setup_response.data.unwrap();
    let backup_code = setup_data.backup_codes[0].clone();

    // 2. 启用 TOTP
    use totp_rs::{Algorithm, TOTP};
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        setup_data.secret.as_bytes().to_vec(),
    ).expect("Failed to create TOTP");
    
    let code = totp.generate_current().unwrap();

    let verify_req = json!({
        "user_id": user_id.to_string(),
        "code": code
    });

    let _: (StatusCode, ApiResponse<totp::VerifyTotpResponse>) =
        post_request(&mut app, &format!("/api/totp/{}/verify", user_id), &verify_req).await;

    // 3. 使用备用代码登录
    let backup_login_req = json!({
        "username": "totp_test_user",
        "password": "password123",
        "backup_code": backup_code
    });

    let (status, response): (StatusCode, ApiResponse<LoginResponse>) =
        post_request(&mut app, "/api/totp/backup-login", &backup_login_req).await;

    assert_eq!(status, 200);
    assert!(response.success);
    assert!(!response.data.unwrap().token.is_empty());

    // 4. 验证备用代码已被使用（不应该再次成功）
    let backup_login_req2 = json!({
        "username": "totp_test_user",
        "password": "password123",
        "backup_code": backup_code // 相同的备用代码
    });

    let (status, response): (StatusCode, ApiResponse<LoginResponse>) =
        post_request(&mut app, "/api/totp/backup-login", &backup_login_req2).await;

    assert_eq!(status, 401);
    assert!(!response.success);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_totp_login_without_totp_setup() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    let (_user_id, _token) = setup_test_user(&mut app).await;

    // 尝试使用备用代码登录（但该用户未设置 TOTP）
    let backup_login_req = json!({
        "username": "totp_test_user",
        "password": "password123",
        "backup_code": "12345678"
    });

    let (status, response): (StatusCode, ApiResponse<LoginResponse>) =
        post_request(&mut app, "/api/totp/backup-login", &backup_login_req).await;

    // 应该正常登录（因为未启用 TOTP）
    assert_eq!(status, 200);
    assert!(response.success);

    cleanup_test_data(&pool).await;
}
