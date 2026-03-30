/// 定时任务模块
/// 实现流量记录、流量重置、过期检查等定时任务

use sqlx::PgPool;
use tracing::{info, error};
use chrono::{Utc, Timelike};
use std::time::Duration;

mod traffic_recorder;
mod traffic_reset;
mod subscription_notifier;

pub use traffic_recorder::run_traffic_recorder;
pub use traffic_reset::run_traffic_reset;
pub use subscription_notifier::{cleanup_expired_tokens, cleanup_old_access_logs};

/// 启动所有定时任务
pub async fn spawn_all_tasks(pool: PgPool) {
    // 启动流量记录任务（每小时执行一次）
    tokio::spawn(run_traffic_recorder(pool.clone()));
    info!("📊 Traffic recorder task started (hourly)");

    // 启动流量重置任务（每天凌晨 2 点执行）
    tokio::spawn(run_traffic_reset(pool.clone()));
    info!("🔄 Traffic reset task started (daily at 02:00)");

    // 启动订阅令牌清理任务（每天凌晨 3 点执行）
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            let sleep_duration = next_day_3am_duration();
            tokio::time::sleep(sleep_duration).await;
            
            match cleanup_expired_tokens(&cleanup_pool).await {
                Ok(deleted) => info!("🧹 Cleaned up {} expired subscription tokens", deleted),
                Err(e) => error!("Failed to cleanup subscription tokens: {}", e),
            }
        }
    });
    info!("🧹 Subscription token cleanup task started (daily at 03:00)");

    // 启动访问日志清理任务（每 7 天执行一次）
    let logs_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            let sleep_duration = next_week_duration();
            tokio::time::sleep(sleep_duration).await;
            
            match cleanup_old_access_logs(&logs_pool, 30).await {
                Ok(deleted) => info!("🧹 Cleaned up {} old subscription access logs", deleted),
                Err(e) => error!("Failed to cleanup access logs: {}", e),
            }
        }
    });
    info!("🧹 Subscription access log cleanup task started (weekly)");

    info!("✅ All scheduled tasks initialized");
}

/// 计算到下一个整点的小时数
fn next_hour_duration() -> Duration {
    let now = Utc::now();
    let next_hour = now
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
        + chrono::Duration::hours(1);
    
    let duration = next_hour.signed_duration_since(now);
    Duration::from_secs(duration.num_seconds() as u64)
}

/// 计算到明天凌晨 2 点的时间
fn next_day_2am_duration() -> Duration {
    let now = Utc::now();
    let tomorrow_2am = now
        .date_naive()
        .succ_opt()
        .unwrap()
        .and_hms_opt(2, 0, 0)
        .unwrap()
        .and_utc();
    
    let duration = tomorrow_2am.signed_duration_since(now);
    Duration::from_secs(duration.num_seconds() as u64)
}

/// 计算到明天凌晨 3 点的时间
fn next_day_3am_duration() -> Duration {
    let now = Utc::now();
    let tomorrow_3am = now
        .date_naive()
        .succ_opt()
        .unwrap()
        .and_hms_opt(3, 0, 0)
        .unwrap()
        .and_utc();
    
    let duration = tomorrow_3am.signed_duration_since(now);
    Duration::from_secs(duration.num_seconds() as u64)
}

/// 计算到下周的固定时间间隔（7 天）
fn next_week_duration() -> Duration {
    Duration::from_secs(7 * 24 * 60 * 60)  // 7 days in seconds
}
