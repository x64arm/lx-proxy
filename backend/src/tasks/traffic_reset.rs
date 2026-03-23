/// 定时流量重置任务
/// 每天凌晨 2 点检查并处理：
/// 1. 流量用尽的入站配置
/// 2. 已过期的入站配置
/// 3. 按周期重置流量（如每月 1 日重置）

use sqlx::{PgPool, Row};
use tracing::{info, error, warn};
use chrono::{Utc, Datelike, Timelike};
use std::time::Duration;
use tokio::time::interval;
use crate::email::EmailClient;

/// 运行流量重置任务（每天凌晨 2 点执行）
pub async fn run_traffic_reset(pool: PgPool) {
    info!("🔄 Starting traffic reset task...");

    // 等待到明天凌晨 2 点
    let initial_delay = super::next_day_2am_duration();
    info!("⏰ Next reset will run at {}", Utc::now() + initial_delay);
    tokio::time::sleep(initial_delay).await;

    // 创建每天执行的 interval（24 小时）
    let mut interval = interval(Duration::from_secs(86400));

    loop {
        interval.tick().await;
        
        match reset_traffic(&pool).await {
            Ok(stats) => {
                info!(
                    "🔄 Traffic reset completed at {}: {} expired, {} over limit, {} reset",
                    Utc::now(),
                    stats.expired_count,
                    stats.over_limit_count,
                    stats.reset_count
                );
            }
            Err(e) => {
                error!("❌ Failed to reset traffic: {}", e);
            }
        }
    }
}

/// 流量重置统计
#[derive(Debug, Default)]
struct ResetStats {
    expired_count: usize,
    over_limit_count: usize,
    reset_count: usize,
}

/// 执行流量重置逻辑
async fn reset_traffic(pool: &PgPool) -> Result<ResetStats, sqlx::Error> {
    let mut stats = ResetStats::default();
    let now = Utc::now();

    // 1. 处理已过期的入站配置
    stats.expired_count = handle_expired_inbounds(pool, now).await?;

    // 2. 处理流量超限的入站配置
    stats.over_limit_count = handle_over_limit_inbounds(pool).await?;

    // 3. 处理周期性重置（每月 1 日）
    if now.day() == 1 && now.hour() == 2 {
        stats.reset_count = handle_monthly_reset(pool).await?;
    }

    Ok(stats)
}

/// 处理已过期的入站配置
async fn handle_expired_inbounds(pool: &PgPool, now: chrono::DateTime<Utc>) -> Result<usize, sqlx::Error> {
    // 查找所有已过期的入站配置
    let expired_inbounds = sqlx::query(
        r#"
        UPDATE inbound_configs
        SET enable = false, updated_at = $1
        WHERE expire_at IS NOT NULL 
          AND expire_at < $1
          AND enable = true
        RETURNING id, tag, expire_at, user_id
        "#
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    let count = expired_inbounds.len();

    // 初始化邮件客户端（如果已配置）
    let email_client = EmailClient::from_env();

    for row in expired_inbounds {
        let id: uuid::Uuid = row.get("id");
        let tag: String = row.get("tag");
        let expire_at: chrono::DateTime<Utc> = row.get("expire_at");
        let user_id: Option<uuid::Uuid> = row.get("user_id");
        
        info!("🚫 Disabled expired inbound: {} (ID: {}, Expired: {})", tag, id, expire_at);
        
        // 发送邮件通知
        if let Some(ref client) = email_client {
            if let Some(uid) = user_id {
                // 获取用户邮箱
                if let Ok(user_email) = get_user_email(pool, uid).await {
                    match client.send_disabled_notification(
                        &user_email.email,
                        &user_email.username,
                        &tag,
                        "配置已过期",
                    ) {
                        Ok(_) => info!("📧 Sent expiry notification to {}", user_email.email),
                        Err(e) => warn!("Failed to send email to {}: {}", user_email.email, e),
                    }
                }
            }
        }
    }

    Ok(count)
}

/// 处理流量超限的入站配置
async fn handle_over_limit_inbounds(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // 查找所有流量超限的入站配置
    let over_limit_inbounds = sqlx::query(
        r#"
        UPDATE inbound_configs
        SET enable = false, updated_at = $1
        WHERE traffic_limit IS NOT NULL
          AND traffic_used >= traffic_limit
          AND enable = true
        RETURNING id, tag, traffic_used, traffic_limit, user_id
        "#
    )
    .bind(Utc::now())
    .fetch_all(pool)
    .await?;

    let count = over_limit_inbounds.len();

    // 初始化邮件客户端（如果已配置）
    let email_client = EmailClient::from_env();

    for row in over_limit_inbounds {
        let id: uuid::Uuid = row.get("id");
        let tag: String = row.get("tag");
        let used: i64 = row.get("traffic_used");
        let limit: Option<i64> = row.get("traffic_limit");
        let user_id: Option<uuid::Uuid> = row.get("user_id");
        
        let limit_val = limit.unwrap_or(0);
        info!(
            "🚫 Disabled over-limit inbound: {} (ID: {}, Used: {} bytes, Limit: {} bytes)",
            tag,
            id,
            used,
            limit_val
        );
        
        // 发送邮件通知
        if let Some(ref client) = email_client {
            if let Some(uid) = user_id {
                // 获取用户邮箱
                if let Ok(user_email) = get_user_email(pool, uid).await {
                    match client.send_disabled_notification(
                        &user_email.email,
                        &user_email.username,
                        &tag,
                        "流量已用尽",
                    ) {
                        Ok(_) => info!("📧 Sent over-limit notification to {}", user_email.email),
                        Err(e) => warn!("Failed to send email to {}: {}", user_email.email, e),
                    }
                }
            }
        }
    }

    Ok(count)
}

/// 处理月度流量重置（每月 1 日执行）
async fn handle_monthly_reset(pool: &PgPool) -> Result<usize, sqlx::Error> {
    info!("📅 Performing monthly traffic reset...");

    // 重置所有启用的入站配置的流量计数
    let reset_inbounds = sqlx::query(
        r#"
        UPDATE inbound_configs
        SET traffic_used = 0, updated_at = $1
        WHERE enable = true
          AND traffic_limit IS NOT NULL
        RETURNING id, tag
        "#
    )
    .bind(Utc::now())
    .fetch_all(pool)
    .await?;

    let count = reset_inbounds.len();

    for row in reset_inbounds {
        let id: uuid::Uuid = row.get("id");
        let tag: String = row.get("tag");
        
        info!("🔄 Reset traffic for inbound: {} (ID: {})", tag, id);
    }

    Ok(count)
}

/// 用户邮箱信息
#[derive(Debug)]
struct UserEmail {
    email: String,
    username: String,
}

/// 获取用户邮箱
async fn get_user_email(pool: &PgPool, user_id: uuid::Uuid) -> Result<UserEmail, sqlx::Error> {
    let user = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT username, email
        FROM users
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(UserEmail {
        username: user.0,
        email: user.1,
    })
}

/// 手动重置指定入站的流量（供 API 调用）
#[allow(dead_code)]
pub async fn reset_single_inbound(pool: &PgPool, inbound_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE inbound_configs
        SET traffic_used = 0, updated_at = $1
        WHERE id = $2
        "#
    )
    .bind(Utc::now())
    .bind(inbound_id)
    .execute(pool)
    .await?;

    info!("🔄 Manually reset traffic for inbound: {}", inbound_id);
    Ok(())
}
