/// 批量操作 API 处理器

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use tracing::info;

/// 批量操作请求
#[derive(Debug, Deserialize)]
pub struct BatchOperationRequest {
    pub ids: Vec<Uuid>,
}

/// 批量创建请求
#[derive(Debug, Deserialize)]
pub struct BatchCreateRequest {
    pub items: Vec<serde_json::Value>,
}

/// 批量操作响应
#[derive(Debug, Serialize)]
pub struct BatchOperationResponse {
    pub success: bool,
    pub processed: usize,
    pub failed: usize,
    pub errors: Vec<BatchError>,
}

/// 批量操作错误
#[derive(Debug, Serialize)]
pub struct BatchError {
    pub id: Uuid,
    pub error: String,
}

/// 批量启用入站配置
pub async fn batch_enable_inbounds(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for id in req.ids {
        match sqlx::query(
            "UPDATE inbound_configs SET enable = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(&state.pool)
        .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量启用入站配置：{}", id);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}

/// 批量禁用入站配置
pub async fn batch_disable_inbounds(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for id in req.ids {
        match sqlx::query(
            "UPDATE inbound_configs SET enable = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(&state.pool)
        .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量禁用入站配置：{}", id);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}

/// 批量删除入站配置
pub async fn batch_delete_inbounds(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for id in req.ids {
        match sqlx::query("DELETE FROM inbound_configs WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量删除入站配置：{}", id);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}

/// 批量重置流量
pub async fn batch_reset_traffic(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for id in req.ids {
        match sqlx::query(
            "UPDATE inbound_configs SET traffic_used = 0, total_upload = 0, total_download = 0, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(&state.pool)
        .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量重置流量：{}", id);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}

/// 批量删除用户
pub async fn batch_delete_users(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for id in req.ids {
        match sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量删除用户：{}", id);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}

/// 批量导出配置
pub async fn batch_export_configs(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchOperationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let placeholders = req
        .ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        "SELECT id, tag, protocol, port, settings, stream_settings, enable, traffic_used, traffic_limit, expire_at 
         FROM inbound_configs 
         WHERE id IN ({})",
        placeholders
    );

    let ids: Vec<Uuid> = req.ids.clone();
    
    // 动态构建查询参数
    let mut db_query = sqlx::query(&query);
    for id in ids {
        db_query = db_query.bind(id);
    }

    let configs = db_query
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("批量导出失败：{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 转换为 JSON 数组
    let result: Vec<serde_json::Value> = configs
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "tag": row.get::<String, _>("tag"),
                "protocol": row.get::<String, _>("protocol"),
                "port": row.get::<i32, _>("port"),
                "settings": row.get::<serde_json::Value, _>("settings"),
                "stream_settings": row.get::<Option<serde_json::Value>, _>("stream_settings"),
                "enable": row.get::<bool, _>("enable"),
                "traffic_used": row.get::<i64, _>("traffic_used"),
                "traffic_limit": row.get::<Option<i64>, _>("traffic_limit"),
                "expire_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expire_at"),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "count": result.len(),
        "data": result
    })))
}

/// 批量导入配置
pub async fn batch_import_configs(
    State(state): State<crate::AppState>,
    Json(req): Json<BatchCreateRequest>,
) -> Result<Json<BatchOperationResponse>, StatusCode> {
    let mut processed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for item in req.items {
        let tag = item.get("tag").and_then(|v| v.as_str()).unwrap_or("");
        let protocol = item.get("protocol").and_then(|v| v.as_str()).unwrap_or("vmess");
        let port = item.get("port").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let settings = item.get("settings").cloned().unwrap_or(serde_json::json!({}));
        let stream_settings = item.get("stream_settings").cloned();
        let user_id = item.get("user_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());
        let traffic_limit = item.get("traffic_limit").and_then(|v| v.as_i64());
        let expire_at = item.get("expire_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc));

        match sqlx::query(
            r#"
            INSERT INTO inbound_configs 
            (user_id, tag, protocol, port, settings, stream_settings, traffic_limit, expire_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(user_id)
        .bind(tag)
        .bind(protocol)
        .bind(port)
        .bind(settings)
        .bind(stream_settings)
        .bind(traffic_limit)
        .bind(expire_at)
        .execute(&state.pool)
        .await
        {
            Ok(_) => {
                processed += 1;
                info!("批量导入配置：{}", tag);
            }
            Err(e) => {
                failed += 1;
                errors.push(BatchError {
                    id: Uuid::nil(),
                    error: format!("{}: {}", tag, e),
                });
            }
        }
    }

    Ok(Json(BatchOperationResponse {
        success: failed == 0,
        processed,
        failed,
        errors,
    }))
}
