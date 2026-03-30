/// TOTP 双因素认证 API 处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::totp;

/// TOTP 设置响应
#[derive(Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
    pub message: String,
}

/// TOTP 验证请求
#[derive(Deserialize)]
pub struct TotpVerifyRequest {
    pub code: String,
}

/// TOTP 状态响应
#[derive(Serialize)]
pub struct TotpStatusResponse {
    pub enabled: bool,
    pub verified: bool,
    pub backup_codes_remaining: usize,
}

/// 初始化 TOTP 设置
pub async fn init_totp_setup(
    State(state): State<crate::AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<TotpSetupResponse>, StatusCode> {
    let user_id = req
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // 获取用户名
    let username = sqlx::query_scalar::<_, String>(
        "SELECT username FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // 检查是否已启用 TOTP
    let (enabled, _) = totp::get_totp_status(&state.pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get TOTP status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if enabled {
        return Err(StatusCode::CONFLICT);
    }

    // 设置 TOTP
    let setup_result = totp::setup_totp(&state.pool, user_id, &username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to setup TOTP: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(TotpSetupResponse {
        secret: setup_result.secret,
        qr_code_url: setup_result.qr_code_url,
        backup_codes: setup_result.backup_codes,
        message: "请使用 Authenticator 应用扫描二维码或手动输入密钥".to_string(),
    }))
}

/// 验证并启用 TOTP
pub async fn verify_and_enable_totp(
    State(state): State<crate::AppState>,
    path: Path<Uuid>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<StatusCode, StatusCode> {
    let result = totp::enable_totp(&state.pool, path.0, &req.code)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify TOTP: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.success {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// 禁用 TOTP
pub async fn disable_totp(
    State(state): State<crate::AppState>,
    path: Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let password = req
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // TODO: 验证密码（应该由认证中间件处理）

    let result = totp::disable_totp(&state.pool, path.0, password)
        .await
        .map_err(|e| {
            tracing::error!("Failed to disable TOTP: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.success {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// 获取 TOTP 状态
pub async fn get_totp_status(
    State(state): State<crate::AppState>,
    path: Path<Uuid>,
) -> Result<Json<TotpStatusResponse>, StatusCode> {
    let (enabled, verified) = totp::get_totp_status(&state.pool, path.0)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get TOTP status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 获取剩余备用代码数量
    let backup_codes: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT backup_codes FROM user_totp_configs WHERE user_id = $1"
    )
    .bind(path.0)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch backup codes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let backup_codes_remaining = backup_codes
        .and_then(|v| v.as_array().map(|a| a.len()))
        .unwrap_or(0);

    Ok(Json(TotpStatusResponse {
        enabled,
        verified,
        backup_codes_remaining,
    }))
}

/// 使用备用代码登录
pub async fn login_with_backup_code(
    State(state): State<crate::AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let user_id = req
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let backup_code = req
        .get("backup_code")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let valid = totp::verify_backup_code(&state.pool, user_id, backup_code)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify backup code: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if valid {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
