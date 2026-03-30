/// 邮件通知模块集成测试
/// 测试邮件发送功能和模板

mod common;

use common::{create_test_app_state, create_test_app, run_migrations, cleanup_test_data};
use lx_proxy_backend::email::EmailClient;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// 创建测试用户并获取认证 token
async fn create_admin_token(
    app: &mut axum::Router,
    username: &str,
    password: &str,
) -> String {
    use crate::common::post_request;
    
    // 先创建用户（直接插入数据库）
    let (_state, pool) = create_test_app_state().await;
    run_migrations(&pool).await;
    
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query(
        "INSERT INTO users (username, password_hash, role, email) 
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (username) DO NOTHING"
    )
    .bind(username)
    .bind(&password_hash)
    .bind("admin")
    .bind(Some("admin@example.com"))
    .execute(&pool)
    .await
    .unwrap();

    // 登录获取 token
    let login_req = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let (_, login_response): (_, ApiResponse<LoginResponse>) =
        post_request(app, "/api/auth/login", &login_req).await;

    login_response.data.unwrap().token
}

#[tokio::test]
async fn test_email_client_creation() {
    // 测试从环境变量创建邮件客户端
    std::env::set_var("SMTP_SERVER", "smtp.example.com");
    std::env::set_var("SMTP_PORT", "587");
    std::env::set_var("SMTP_USERNAME", "test@example.com");
    std::env::set_var("SMTP_PASSWORD", "password123");
    std::env::set_var("SMTP_FROM_EMAIL", "test@example.com");
    std::env::set_var("SMTP_FROM_NAME", "LX-Proxy Test");

    let client = EmailClient::from_env();
    assert!(client.is_some(), "Email client should be created from env vars");

    let client = client.unwrap();
    assert_eq!(client.smtp_server, "smtp.example.com");
    assert_eq!(client.smtp_port, 587);
    assert_eq!(client.username, "test@example.com");
    assert_eq!(client.from_name, "LX-Proxy Test");

    // 清理环境变量
    std::env::remove_var("SMTP_SERVER");
    std::env::remove_var("SMTP_PORT");
    std::env::remove_var("SMTP_USERNAME");
    std::env::remove_var("SMTP_PASSWORD");
    std::env::remove_var("SMTP_FROM_EMAIL");
    std::env::remove_var("SMTP_FROM_NAME");
}

#[tokio::test]
async fn test_email_configured_check() {
    use crate::email::is_email_configured;

    // 未配置时应该返回 false
    std::env::remove_var("SMTP_SERVER");
    std::env::remove_var("SMTP_USERNAME");
    std::env::remove_var("SMTP_PASSWORD");
    assert!(!is_email_configured());

    // 配置后应该返回 true
    std::env::set_var("SMTP_SERVER", "smtp.example.com");
    std::env::set_var("SMTP_USERNAME", "test@example.com");
    std::env::set_var("SMTP_PASSWORD", "password123");
    assert!(is_email_configured());

    // 清理
    std::env::remove_var("SMTP_SERVER");
    std::env::remove_var("SMTP_USERNAME");
    std::env::remove_var("SMTP_PASSWORD");
}

#[tokio::test]
async fn test_email_api_test_endpoint() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建管理员用户并获取 token
    let token = create_admin_token(&mut app, "admin", "admin123").await;

    // 测试邮件发送 API（需要配置 SMTP）
    // 注意：这个测试在实际 SMTP 配置时才会成功
    std::env::set_var("SMTP_SERVER", "smtp.example.com");
    std::env::set_var("SMTP_PORT", "587");
    std::env::set_var("SMTP_USERNAME", "test@example.com");
    std::env::set_var("SMTP_PASSWORD", "password123");

    let test_email_req = json!({
        "to": "test@example.com"
    });

    use crate::common::post_request;
    let (status, response): (StatusCode, serde_json::Value) =
        post_request(&mut app, "/api/email/test", &test_email_req).await;

    // 由于 SMTP 服务器不可达，应该返回错误
    // 但 API 端点应该可访问
    assert_eq!(status, 200);
    assert!(response.get("success").is_some());

    // 清理
    std::env::remove_var("SMTP_SERVER");
    std::env::remove_var("SMTP_PORT");
    std::env::remove_var("SMTP_USERNAME");
    std::env::remove_var("SMTP_PASSWORD");
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_email_api_status_endpoint() {
    let (state, pool) = create_test_app_state().await;
    let mut app = create_test_app(state);
    run_migrations(&pool).await;

    // 创建管理员用户并获取 token
    let token = create_admin_token(&mut app, "admin", "admin123").await;

    // 测试邮件状态 API
    use crate::common::get_request;
    let (status, response): (StatusCode, serde_json::Value) =
        get_request(&mut app, "/api/email/status", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(response.get("configured").unwrap().is_boolean());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_email_templates_exist() {
    // 测试邮件模板模块存在且可访问
    use crate::email::templates;
    
    // 测试流量告警模板
    let alert_template = templates::traffic_alert("Test Inbound", 85.5);
    assert!(alert_template.contains("流量告警"));
    assert!(alert_template.contains("Test Inbound"));
    assert!(alert_template.contains("85.5"));

    // 测试到期提醒模板
    let reminder_template = templates::expiry_reminder("Test Inbound", 7);
    assert!(reminder_template.contains("到期提醒"));
    assert!(reminder_template.contains("7"));

    // 测试禁用通知模板
    let disabled_template = templates::disabled_notification("Test Inbound", "流量超限");
    assert!(disabled_template.contains("服务已禁用"));
    assert!(disabled_template.contains("流量超限"));

    // 测试邮件模板
    let test_template = templates::test_email();
    assert!(test_template.contains("邮件测试"));
}

#[tokio::test]
async fn test_email_traffic_alert_thresholds() {
    // 测试流量告警阈值逻辑
    fn should_send_alert(usage_percent: f64, last_alert_percent: Option<f64>) -> bool {
        const THRESHOLDS: [f64; 2] = [70.0, 90.0];
        
        for threshold in &THRESHOLDS {
            if usage_percent >= *threshold {
                if let Some(last) = last_alert_percent {
                    // 只在跨越新阈值时发送
                    if last < *threshold {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
        false
    }

    // 首次达到 70% 应该发送
    assert!(should_send_alert(70.0, None));
    
    // 70% 之后再次 75% 不应该发送（未跨越新阈值）
    assert!(!should_send_alert(75.0, Some(70.0)));
    
    // 达到 90% 应该发送（跨越新阈值）
    assert!(should_send_alert(90.0, Some(75.0)));
    
    // 低于阈值不应该发送
    assert!(!should_send_alert(50.0, None));
}
