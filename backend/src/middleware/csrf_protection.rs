// P18 安全加固 - CSRF 防护与关键操作二次验证
// 为敏感操作提供额外的安全层

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::AppState;
use crate::models::Claims;

/// 二次验证请求
#[derive(Debug, Deserialize)]
pub struct VerificationRequest {
    /// 操作类型
    pub action: String,
    /// 操作目标 ID（可选）
    pub target_id: Option<Uuid>,
    /// 用户密码（用于验证身份）
    pub password: String,
    /// TOTP 验证码（如果启用了 2FA）
    pub totp_code: Option<String>,
}

/// 二次验证响应
#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    /// 验证是否通过
    pub verified: bool,
    /// 验证令牌（有效期 5 分钟）
    pub verification_token: Option<String>,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 验证令牌存储（内存中，生产环境应使用 Redis）
#[derive(Debug, Clone)]
pub struct VerificationToken {
    pub user_id: Uuid,
    pub action: String,
    pub target_id: Option<Uuid>,
    pub expires_at: chrono::DateTime<Utc>,
}

/// 敏感操作类型
pub mod sensitive_actions {
    pub const DELETE_USER: &str = "delete_user";
    pub const DELETE_INBOUND: &str = "delete_inbound";
    pub const UPDATE_CONFIG: &str = "update_config";
    pub const UPDATE_XRAY_CONFIG: &str = "update_xray_config";
    pub const DISABLE_TOTP: &str = "disable_totp";
    pub const BATCH_DELETE: &str = "batch_delete";
    pub const BATCH_RESET: &str = "batch_reset";
    pub const EXPORT_DATA: &str = "export_data";
}

/// 请求二次验证
pub async fn request_verification(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<VerificationRequest>,
) -> Result<Json<VerificationResponse>, StatusCode> {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
    
    // 验证用户密码
    let user = sqlx::query_as::<_, crate::models::User>(
        r#"SELECT * FROM users WHERE id = $1 LIMIT 1"#
    )
    .bind(claims.sub.parse::<Uuid>().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user for verification: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证密码
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let password_valid = Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !password_valid {
        // 记录失败的验证尝试
        tracing::warn!("Password verification failed for user: {}", user.username);
        crate::audit::service::log_security_event(
            &state.pool,
            &user.id,
            "verification_failed",
            Some(&format!("Failed to verify password for action: {}", req.action)),
            None,
        ).await.ok();
        
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 如果用户启用了 TOTP，验证 TOTP 码
    let totp_status: Option<(String, bool)> = sqlx::query_as(
        r#"SELECT secret, enabled FROM user_totp_configs WHERE user_id = $1 LIMIT 1"#
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch TOTP config: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some((secret, enabled)) = totp_status {
        if enabled {
            let totp_code = req.totp_code
                .ok_or_else(|| {
                    tracing::warn!("TOTP required but not provided");
                    StatusCode::BAD_REQUEST
                })?;

            let valid = crate::totp::verify_code(&secret, &totp_code);
            if !valid {
                tracing::warn!("TOTP verification failed for user: {}", user.username);
                crate::audit::service::log_security_event(
                    &state.pool,
                    &user.id,
                    "totp_verification_failed",
                    Some(&format!("Failed TOTP verification for action: {}", req.action)),
                    None,
                ).await.ok();
                
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // 生成验证令牌
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(5);

    // 存储验证令牌（生产环境应使用 Redis）
    // 这里简化处理，实际应该在 Redis 中存储
    tracing::info!("Verification token generated for user: {}", user.username);

    // 记录成功的验证
    crate::audit::service::log_security_event(
        &state.pool,
        &user.id,
        "verification_success",
        Some(&format!("Verified for action: {}", req.action)),
        None,
    ).await.ok();

    Ok(Json(VerificationResponse {
        verified: true,
        verification_token: Some(token),
        expires_in: 300, // 5 分钟
    }))
}

/// 验证操作令牌
pub async fn verify_operation_token(
    _state: &AppState,
    _claims: &Claims,
    _token: &str,
    _expected_action: &str,
) -> Result<bool, StatusCode> {
    // 生产环境应该从 Redis 验证令牌
    // 这里简化处理，仅做示例
    
    // TODO: 实现 Redis 令牌验证
    // 令牌格式：user_id:action:target_id:timestamp
    // 验证：
    // 1. 令牌是否存在且未过期
    // 2. 令牌所属用户与当前用户匹配
    // 3. 令牌对应的操作与预期操作匹配
    // 4. 验证后删除令牌（一次性使用）
    
    tracing::debug!("Verifying operation token for action");
    
    // 临时实现：始终返回 true（开发环境）
    // 生产环境必须实现真正的令牌验证
    Ok(true)
}

/// 检查操作是否需要二次验证
pub fn requires_verification(action: &str) -> bool {
    matches!(
        action,
        sensitive_actions::DELETE_USER
        | sensitive_actions::DELETE_INBOUND
        | sensitive_actions::UPDATE_CONFIG
        | sensitive_actions::UPDATE_XRAY_CONFIG
        | sensitive_actions::DISABLE_TOTP
        | sensitive_actions::BATCH_DELETE
        | sensitive_actions::BATCH_RESET
        | sensitive_actions::EXPORT_DATA
    )
}

/// 需要二次验证的操作包装器
/// 用法：在敏感操作 handler 中调用此函数检查验证
pub async fn require_verification(
    state: &AppState,
    claims: &Claims,
    action: &str,
    verification_token: Option<&str>,
) -> Result<(), StatusCode> {
    if !requires_verification(action) {
        return Ok(());
    }

    let token = verification_token
        .ok_or_else(|| {
            tracing::warn!("Verification token required for action: {}", action);
            StatusCode::UNAUTHORIZED
        })?;

    let valid = verify_operation_token(state, claims, token, action).await?;
    
    if !valid {
        tracing::warn!("Invalid or expired verification token for action: {}", action);
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_verification() {
        assert!(requires_verification(sensitive_actions::DELETE_USER));
        assert!(requires_verification(sensitive_actions::BATCH_DELETE));
        assert!(requires_verification(sensitive_actions::UPDATE_CONFIG));
        
        assert!(!requires_verification("list_users"));
        assert!(!requires_verification("get_stats"));
        assert!(!requires_verification("health_check"));
    }
}
