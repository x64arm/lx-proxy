use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use serde::Deserialize;
use uuid::Uuid;

/// 流量统计查询参数
#[derive(Debug, Deserialize)]
pub struct TrafficQuery {
    pub inbound_id: Option<Uuid>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
}

/// 流量统计记录
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct TrafficRecord {
    pub date: String,
    pub inbound_id: Uuid,
    pub inbound_name: String,
    pub upload: i64,
    pub download: i64,
}

/// 获取所有流量统计
pub async fn get_all_traffic(
    State(pool): State<PgPool>,
    Query(params): Query<TrafficQuery>,
) -> Result<Json<Vec<TrafficRecord>>, StatusCode> {
    let limit = params.limit.unwrap_or(30);

    let records = sqlx::query_as::<_, TrafficRecord>(
        r#"
        SELECT 
            DATE(t.recorded_at)::text as date,
            i.id as inbound_id,
            i.tag as inbound_name,
            COALESCE(SUM(t.upload), 0) as upload,
            COALESCE(SUM(t.download), 0) as download
        FROM traffic_logs t
        JOIN inbound_configs i ON t.inbound_id = i.id
        WHERE ($1::uuid IS NULL OR i.id = $1)
          AND ($2::date IS NULL OR DATE(t.recorded_at) >= $2::date)
          AND ($3::date IS NULL OR DATE(t.recorded_at) <= $3::date)
        GROUP BY DATE(t.recorded_at), i.id, i.tag
        ORDER BY DATE(t.recorded_at) DESC, i.tag
        LIMIT $4
        "#
    )
    .bind(params.inbound_id)
    .bind(params.start_date)
    .bind(params.end_date)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch traffic stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(records))
}

/// 获取指定入站的流量统计
pub async fn get_inbound_traffic_stats(
    State(pool): State<PgPool>,
    Path(inbound_id): Path<Uuid>,
    Query(params): Query<TrafficQuery>,
) -> Result<Json<Vec<TrafficRecord>>, StatusCode> {
    let limit = params.limit.unwrap_or(30);

    let records = sqlx::query_as::<_, TrafficRecord>(
        r#"
        SELECT 
            DATE(t.recorded_at)::text as date,
            i.id as inbound_id,
            i.tag as inbound_name,
            COALESCE(SUM(t.upload), 0) as upload,
            COALESCE(SUM(t.download), 0) as download
        FROM traffic_logs t
        JOIN inbound_configs i ON t.inbound_id = i.id
        WHERE i.id = $1
          AND ($2::date IS NULL OR DATE(t.recorded_at) >= $2::date)
          AND ($3::date IS NULL OR DATE(t.recorded_at) <= $3::date)
        GROUP BY DATE(t.recorded_at), i.id, i.tag
        ORDER BY DATE(t.recorded_at) DESC
        LIMIT $4
        "#
    )
    .bind(inbound_id)
    .bind(params.start_date)
    .bind(params.end_date)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch inbound traffic stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(records))
}

/// 获取流量汇总统计
pub async fn get_traffic_summary(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let summary = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT 
            COALESCE(SUM(upload), 0) as total_upload,
            COALESCE(SUM(download), 0) as total_download,
            COALESCE(SUM(upload + download), 0) as total_traffic
        FROM traffic_logs
        "#
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch traffic summary: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "total_upload": summary.0,
        "total_download": summary.1,
        "total_traffic": summary.2
    })))
}

/// 记录流量日志（供定时任务调用）
pub async fn record_traffic_log(
    State(pool): State<PgPool>,
    Json(req): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let inbound_id = req.get("inbound_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let upload = req.get("upload")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    
    let download = req.get("download")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    sqlx::query(
        r#"
        INSERT INTO traffic_logs (inbound_id, upload, download, recorded_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (inbound_id, DATE(recorded_at)) 
        DO UPDATE SET 
            upload = traffic_logs.upload + $2,
            download = traffic_logs.download + $3
        "#
    )
    .bind(inbound_id)
    .bind(upload)
    .bind(download)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to record traffic log: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}
