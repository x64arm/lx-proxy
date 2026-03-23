/// 邮件通知 API 处理器

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use crate::email::EmailClient;

/// 测试邮件请求
#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub email: String,
}

/// 发送测试邮件
pub async fn send_test_email(
    State(pool): State<PgPool>,
    Json(req): Json<TestEmailRequest>,
) -> Result<StatusCode, StatusCode> {
    // 检查邮件配置
    let email_client = EmailClient::from_env()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // 获取当前用户（管理员）
    let admin = sqlx::query_as::<_, (String, Option<String>)>(
        r#"SELECT username, email FROM users WHERE role = 'admin' LIMIT 1"#
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch admin user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (username, _) = admin.unwrap_or_else(|| ("Admin".to_string(), None));

    // 发送测试邮件
    email_client
        .test_connection(&req.email)
        .map_err(|e| {
            tracing::error!("Failed to send test email: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

/// 邮件配置状态
#[derive(serde::Serialize)]
pub struct EmailConfigStatus {
    pub configured: bool,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<u16>,
    pub from_email: Option<String>,
}

/// 获取邮件配置状态
pub async fn get_email_config_status(
    State(_pool): State<PgPool>,
) -> Result<Json<EmailConfigStatus>, StatusCode> {
    use std::env;

    let configured = env::var("SMTP_SERVER").is_ok()
        && env::var("SMTP_USERNAME").is_ok()
        && env::var("SMTP_PASSWORD").is_ok();

    let status = EmailConfigStatus {
        configured,
        smtp_server: env::var("SMTP_SERVER").ok(),
        smtp_port: env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()),
        from_email: env::var("SMTP_FROM_EMAIL").ok(),
    };

    Ok(Json(status))
}
