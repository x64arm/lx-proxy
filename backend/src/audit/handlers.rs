// P16 审计日志 API 处理器

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::delete,
    Json, Router,
};
use uuid::Uuid;
use crate::{AppState, audit::{AuditService, AuditLogQuery, LoginLog, ConfigChangeHistory, IpBan}};

/// 查询审计日志
pub async fn query_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<crate::audit::AuditLogListResponse>, StatusCode> {
    let service = AuditService::new(state.pool);
    
    service.query_logs(query).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to query audit logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// 获取审计统计信息
pub async fn get_audit_stats(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<crate::audit::AuditStats>, StatusCode> {
    let service = AuditService::new(state.pool);
    
    let days = params.get("days")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(30);
    
    service.get_stats(days).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to get audit stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// 获取单个审计日志详情
pub async fn get_audit_log(
    State(state): State<AppState>,
    Path(log_id): Path<Uuid>,
) -> Result<Json<crate::audit::AuditLog>, StatusCode> {
    let log: Option<crate::audit::AuditLog> = sqlx::query_as(
        r#"SELECT * FROM audit_logs WHERE id = $1 LIMIT 1"#
    )
    .bind(log_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    log.ok_or(StatusCode::NOT_FOUND).map(Json)
}

/// 查询登录日志
pub async fn query_login_logs(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::audit::LoginLog>>, StatusCode> {
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(100);

    let logs: Vec<crate::audit::LoginLog> = sqlx::query_as(
        r#"SELECT * FROM login_logs ORDER BY created_at DESC LIMIT $1"#
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

/// 查询配置变更历史
pub async fn query_config_history(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::audit::ConfigChangeHistory>>, StatusCode> {
    let config_type = params.get("type");
    let limit = params.get("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(100);

    let logs: Vec<crate::audit::ConfigChangeHistory> = if let Some(config_type) = config_type {
        sqlx::query_as(
            r#"SELECT * FROM config_change_history 
               WHERE config_type = $1 
               ORDER BY created_at DESC LIMIT $2"#
        )
        .bind(config_type)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as(
            r#"SELECT * FROM config_change_history 
               ORDER BY created_at DESC LIMIT $1"#
        )
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

/// 获取配置变更详情
pub async fn get_config_change(
    State(state): State<AppState>,
    Path(change_id): Path<Uuid>,
) -> Result<Json<crate::audit::ConfigChangeHistory>, StatusCode> {
    let change: Option<crate::audit::ConfigChangeHistory> = sqlx::query_as(
        r#"SELECT * FROM config_change_history WHERE id = $1 LIMIT 1"#
    )
    .bind(change_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    change.ok_or(StatusCode::NOT_FOUND).map(Json)
}

/// 清理旧审计日志（管理员操作）
pub async fn cleanup_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let retention_days = params.get("days")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(90);

    let service = AuditService::new(state.pool);
    
    match service.cleanup_old_logs(retention_days).await {
        Ok(deleted) => Ok(Json(serde_json::json!({
            "success": true,
            "deleted_count": deleted,
            "retention_days": retention_days
        }))),
        Err(e) => {
            tracing::error!("Failed to cleanup audit logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取 IP 封禁列表
pub async fn get_ip_bans(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::audit::IpBan>>, StatusCode> {
    let bans: Vec<crate::audit::IpBan> = sqlx::query_as(
        r#"SELECT * FROM ip_bans WHERE is_active = TRUE ORDER BY banned_at DESC"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bans))
}

/// 解除 IP 封禁
pub async fn unban_ip(
    State(state): State<AppState>,
    Path(ip_address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query(
        r#"UPDATE ip_bans SET is_active = FALSE WHERE ip_address = $1"#
    )
    .bind(&ip_address)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() > 0 {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("IP {} unbanned successfully", ip_address)
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
