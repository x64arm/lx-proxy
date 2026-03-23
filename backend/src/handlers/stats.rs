/// 高级统计 API 处理器

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use chrono::{Duration, Utc};

/// 统计查询参数
#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub range: Option<String>, // 7d, 30d, 90d
}

/// 高级统计响应
#[derive(serde::Serialize)]
pub struct AdvancedStatsResponse {
    pub traffic: Vec<TrafficData>,
    pub top_users: Vec<UserTraffic>,
    pub protocols: Vec<ProtocolStats>,
    pub hourly: Vec<HourlyActivity>,
    pub forecast: Vec<TrafficForecast>,
    pub inbounds: Vec<InboundStats>,
}

#[derive(serde::Serialize)]
pub struct TrafficData {
    pub date: String,
    pub upload: i64,
    pub download: i64,
}

#[derive(serde::Serialize)]
pub struct UserTraffic {
    pub user_id: String,
    pub username: String,
    pub total_traffic: i64,
}

#[derive(serde::Serialize)]
pub struct ProtocolStats {
    pub protocol: String,
    pub count: i32,
    pub total_traffic: i64,
}

#[derive(serde::Serialize)]
pub struct HourlyActivity {
    pub hour: i32,
    pub active_users: i64,
    pub total_traffic: i64,
}

#[derive(serde::Serialize)]
pub struct TrafficForecast {
    pub date: String,
    pub predicted: i64,
}

#[derive(serde::Serialize)]
pub struct InboundStats {
    pub tag: String,
    pub protocol: String,
    pub traffic_used: i64,
    pub traffic_limit: Option<i64>,
    pub usage_percent: f64,
    pub expire_at: Option<chrono::DateTime<Utc>>,
}

/// 获取高级统计
pub async fn get_advanced_stats(
    State(pool): State<PgPool>,
    Query(params): Query<StatsQuery>,
) -> Result<Json<AdvancedStatsResponse>, StatusCode> {
    let days = match params.range.as_deref() {
        Some("30d") => 30,
        Some("90d") => 90,
        _ => 7,
    };

    // 获取流量数据
    let traffic = get_traffic_data(&pool, days).await?;
    
    // 获取用户排行
    let top_users = get_top_users(&pool, 10).await?;
    
    // 获取协议分布
    let protocols = get_protocol_distribution(&pool).await?;
    
    // 获取活跃时段
    let hourly = get_hourly_activity(&pool).await?;
    
    // 获取流量预测
    let forecast = get_traffic_forecast(&pool, 7).await?;
    
    // 获取入站统计
    let inbounds = get_inbound_stats(&pool).await?;

    Ok(Json(AdvancedStatsResponse {
        traffic,
        top_users,
        protocols,
        hourly,
        forecast,
        inbounds,
    }))
}

/// 获取流量数据
async fn get_traffic_data(pool: &PgPool, days: i32) -> Result<Vec<TrafficData>, StatusCode> {
    let records = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT 
            DATE(recorded_at)::text as date,
            COALESCE(SUM(upload), 0) as upload,
            COALESCE(SUM(download), 0) as download
        FROM traffic_logs
        WHERE recorded_at >= NOW() - INTERVAL '1 day' * $1
        GROUP BY DATE(recorded_at)
        ORDER BY date DESC
        "#
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取流量数据失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(records
        .into_iter()
        .map(|(date, upload, download)| TrafficData { date, upload, download })
        .collect())
}

/// 获取用户流量排行
pub async fn get_top_users(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<UserTraffic>, StatusCode> {
    let records = sqlx::query_as::<_, (String, String, i64)>(
        r#"
        SELECT 
            u.id::text as user_id,
            u.username,
            COALESCE(SUM(ic.traffic_used), 0) as total_traffic
        FROM users u
        LEFT JOIN inbound_configs ic ON u.id = ic.user_id
        GROUP BY u.id, u.username
        ORDER BY total_traffic DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取用户排行失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(records
        .into_iter()
        .map(|(user_id, username, total_traffic)| UserTraffic {
            user_id,
            username,
            total_traffic,
        })
        .collect())
}

/// 获取协议分布
pub async fn get_protocol_distribution(pool: &PgPool) -> Result<Vec<ProtocolStats>, StatusCode> {
    let records = sqlx::query_as::<_, (String, i32, i64)>(
        r#"
        SELECT 
            protocol,
            COUNT(*) as count,
            COALESCE(SUM(traffic_used), 0) as total_traffic
        FROM inbound_configs
        GROUP BY protocol
        ORDER BY count DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取协议分布失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(records
        .into_iter()
        .map(|(protocol, count, total_traffic)| ProtocolStats {
            protocol,
            count,
            total_traffic,
        })
        .collect())
}

/// 获取活跃时段
pub async fn get_hourly_activity(pool: &PgPool) -> Result<Vec<HourlyActivity>, StatusCode> {
    let records = sqlx::query_as::<_, (i32, i64, i64)>(
        r#"
        SELECT 
            EXTRACT(HOUR FROM recorded_at)::int as hour,
            COUNT(DISTINCT ic.user_id) as active_users,
            COALESCE(SUM(tl.upload + tl.download), 0) as total_traffic
        FROM traffic_logs tl
        JOIN inbound_configs ic ON tl.inbound_id = ic.id
        WHERE tl.recorded_at >= NOW() - INTERVAL '7 days'
        GROUP BY EXTRACT(HOUR FROM recorded_at)
        ORDER BY hour
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取活跃时段失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(records
        .into_iter()
        .map(|(hour, active_users, total_traffic)| HourlyActivity {
            hour,
            active_users,
            total_traffic,
        })
        .collect())
}

/// 获取流量预测（简单线性回归）
pub async fn get_traffic_forecast(
    pool: &PgPool,
    days: i32,
) -> Result<Vec<TrafficForecast>, StatusCode> {
    // 获取历史数据
    let historical = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT 
            DATE(recorded_at)::text as date,
            COALESCE(SUM(upload + download), 0) as total
        FROM traffic_logs
        WHERE recorded_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE(recorded_at)
        ORDER BY date
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取历史数据失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 简单预测：取最近 7 天平均值
    let avg_traffic = if !historical.is_empty() {
        historical.iter().map(|(_, t)| t).sum::<i64>() / historical.len() as i64
    } else {
        0
    };

    // 生成预测数据
    let mut forecast = Vec::new();
    for i in 1..=days {
        let date = Utc::now() + Duration::days(i as i64);
        forecast.push(TrafficForecast {
            date: date.format("%Y-%m-%d").to_string(),
            predicted: avg_traffic,
        });
    }

    Ok(forecast)
}

/// 获取入站统计
async fn get_inbound_stats(pool: &PgPool) -> Result<Vec<InboundStats>, StatusCode> {
    let records = sqlx::query_as::<_, (String, String, i64, Option<i64>, Option<chrono::DateTime<Utc>>)>(
        r#"
        SELECT 
            tag,
            protocol,
            traffic_used,
            traffic_limit,
            expire_at
        FROM inbound_configs
        ORDER BY traffic_used DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("获取入站统计失败：{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(records
        .into_iter()
        .map(|(tag, protocol, traffic_used, traffic_limit, expire_at)| {
            let usage_percent = if let Some(limit) = traffic_limit {
                if limit > 0 {
                    (traffic_used as f64 / limit as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            InboundStats {
                tag,
                protocol,
                traffic_used,
                traffic_limit,
                usage_percent,
                expire_at,
            }
        })
        .collect())
}
