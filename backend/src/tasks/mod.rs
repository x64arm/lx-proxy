/// 定时任务模块
/// 实现流量记录、流量重置、过期检查等定时任务

use sqlx::PgPool;
use tracing::info;
use chrono::{Utc, Timelike};
use std::time::Duration;

mod traffic_recorder;
mod traffic_reset;

pub use traffic_recorder::run_traffic_recorder;
pub use traffic_reset::run_traffic_reset;

/// 启动所有定时任务
pub async fn spawn_all_tasks(pool: PgPool) {
    // 启动流量记录任务（每小时执行一次）
    tokio::spawn(run_traffic_recorder(pool.clone()));
    info!("📊 Traffic recorder task started (hourly)");

    // 启动流量重置任务（每天凌晨 2 点执行）
    tokio::spawn(run_traffic_reset(pool.clone()));
    info!("🔄 Traffic reset task started (daily at 02:00)");

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
