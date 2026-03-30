// P13 订阅链接自动更新通知任务
// 功能：清理过期的订阅令牌和访问日志

use sqlx::PgPool;
use tracing::info;

/// 清理过期的订阅令牌
pub async fn cleanup_expired_tokens(
    pool: &PgPool,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query(
        r#"DELETE FROM subscription_tokens WHERE expires_at < NOW() OR used = true"#
    )
    .execute(pool)
    .await?;

    let rows_deleted = result.rows_affected();
    
    if rows_deleted > 0 {
        info!("🧹 Cleaned up {} expired subscription tokens", rows_deleted);
    }

    Ok(rows_deleted)
}

/// 清理旧的访问日志（保留 30 天）
pub async fn cleanup_old_access_logs(
    pool: &PgPool,
    retention_days: i32,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let result = sqlx::query(
        r#"DELETE FROM subscription_access_logs WHERE accessed_at < NOW() - ($1 || ' days')::INTERVAL"#
    )
    .bind(retention_days)
    .execute(pool)
    .await?;

    let rows_deleted = result.rows_affected();
    
    if rows_deleted > 0 {
        info!("🧹 Cleaned up {} old subscription access logs", rows_deleted);
    }

    Ok(rows_deleted)
}
