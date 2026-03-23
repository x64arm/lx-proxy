/// 定时流量记录任务
/// 每小时自动记录所有入站的流量使用情况

use sqlx::PgPool;
use tracing::{info, warn, error};
use chrono::Utc;
use uuid::Uuid;
use std::time::Duration;
use tokio::time::interval;
use crate::xray_api::XrayApiClient;

/// 流量记录数据结构
#[derive(Debug)]
struct TrafficRecord {
    inbound_id: Uuid,
    upload: i64,
    download: i64,
}

/// 运行流量记录任务（每小时执行一次）
pub async fn run_traffic_recorder(pool: PgPool) {
    info!("📊 Starting traffic recorder task...");

    // 等待到下一个整点
    let initial_delay = super::next_hour_duration();
    tokio::time::sleep(initial_delay).await;

    // 创建每小时执行的 interval
    let mut interval = interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;
        
        match record_traffic(&pool).await {
            Ok(count) => {
                info!("📈 Recorded traffic for {} inbounds at {}", count, Utc::now());
            }
            Err(e) => {
                error!("❌ Failed to record traffic: {}", e);
            }
        }
    }
}

/// 记录所有入站的流量
async fn record_traffic(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // 获取所有启用的入站配置
    let inbounds = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT id, tag
        FROM inbound_configs
        WHERE enable = true
        ORDER BY tag
        "#
    )
    .fetch_all(pool)
    .await?;

    // 创建 Xray API 客户端
    let xray_client = XrayApiClient::from_env();
    let mut recorded_count = 0;
    let mut failed_count = 0;

    for (inbound_id, tag) in inbounds {
        // 从 Xray API 获取实时流量
        let (upload, download) = match xray_client.get_inbound_traffic(&tag).await {
            Ok((up, down)) => (up, down),
            Err(e) => {
                warn!("Failed to fetch traffic from Xray API for {}: {}", tag, e);
                failed_count += 1;
                (0, 0)
            }
        };

        // 记录流量日志
        let result = sqlx::query(
            r#"
            INSERT INTO traffic_logs (id, inbound_id, upload, download, recorded_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(inbound_id)
        .bind(upload)
        .bind(download)
        .bind(Utc::now())
        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                recorded_count += 1;
                if upload > 0 || download > 0 {
                    info!("📈 Recorded traffic for {}: ↑{} ↓{}", tag, upload, download);
                }
            }
            Err(e) => {
                warn!("Failed to record traffic for inbound {}: {}", tag, e);
            }
        }
    }

    if failed_count > 0 {
        warn!("⚠️ Failed to fetch traffic for {} inbounds from Xray API", failed_count);
    }

    Ok(recorded_count)
}


