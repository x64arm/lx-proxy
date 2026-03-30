// 性能优化模块 - 数据库查询优化和缓存策略

use sqlx::{PgPool, Row};
use tracing::{info, warn};

/// 数据库索引优化
pub async fn optimize_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("🔧 开始数据库索引优化...");

    // 创建常用查询索引
    let indexes = vec![
        // 流量日志索引
        (
            "idx_traffic_logs_inbound_date",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_traffic_logs_inbound_date 
             ON traffic_logs(inbound_id, recorded_at DESC)",
        ),
        (
            "idx_traffic_logs_date",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_traffic_logs_date 
             ON traffic_logs(recorded_at DESC)",
        ),
        // 入站配置索引
        (
            "idx_inbounds_user_id",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_inbounds_user_id 
             ON inbound_configs(user_id)",
        ),
        (
            "idx_inbounds_protocol",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_inbounds_protocol 
             ON inbound_configs(protocol)",
        ),
        (
            "idx_inbounds_enable",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_inbounds_enable 
             ON inbound_configs(enable) WHERE enable = true",
        ),
        // 用户索引
        (
            "idx_users_username",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_username 
             ON users(username)",
        ),
        (
            "idx_users_email",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email 
             ON users(email) WHERE email IS NOT NULL",
        ),
    ];

    for (name, sql) in indexes {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => info!("✅ 索引创建成功：{}", name),
            Err(e) => warn!("⚠️ 索引创建失败 {}：{}", name, e),
        }
    }

    Ok(())
}

/// 创建物化视图（用于预计算统计数据）
pub async fn create_materialized_views(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("📊 创建物化视图...");

    // 每日流量统计视图
    let daily_stats = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_traffic_stats AS
        SELECT 
            DATE(recorded_at) as stat_date,
            inbound_id,
            SUM(upload) as total_upload,
            SUM(download) as total_download,
            COUNT(*) as record_count
        FROM traffic_logs
        GROUP BY DATE(recorded_at), inbound_id
        WITH DATA;
        
        CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_stats_date_inbound 
        ON mv_daily_traffic_stats(stat_date, inbound_id);
    "#;

    // 用户流量汇总视图
    let user_stats = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS mv_user_traffic_summary AS
        SELECT 
            u.id as user_id,
            u.username,
            COUNT(ic.id) as inbound_count,
            COALESCE(SUM(ic.traffic_used), 0) as total_traffic_used,
            COALESCE(SUM(ic.traffic_total), 0) as total_traffic_limit
        FROM users u
        LEFT JOIN inbound_configs ic ON u.id = ic.user_id
        GROUP BY u.id, u.username
        WITH DATA;
        
        CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_user_stats_user_id 
        ON mv_user_traffic_summary(user_id);
    "#;

    // 协议分布视图
    let protocol_stats = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS mv_protocol_distribution AS
        SELECT 
            protocol,
            COUNT(*) as config_count,
            COALESCE(SUM(traffic_used), 0) as total_traffic_used,
            COALESCE(SUM(traffic_total), 0) as total_traffic_limit
        FROM inbound_configs
        GROUP BY protocol
        WITH DATA;
        
        CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_protocol_protocol 
        ON mv_protocol_distribution(protocol);
    "#;

    for (name, sql) in vec![
        ("mv_daily_traffic_stats", daily_stats),
        ("mv_user_traffic_summary", user_stats),
        ("mv_protocol_distribution", protocol_stats),
    ] {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => info!("✅ 物化视图创建成功：{}", name),
            Err(e) => warn!("⚠️ 物化视图创建失败 {}：{}", name, e),
        }
    }

    Ok(())
}

/// 刷新物化视图
pub async fn refresh_materialized_view(
    pool: &PgPool,
    view_name: &str,
) -> Result<(), sqlx::Error> {
    let sql = format!("REFRESH MATERIALIZED VIEW {}", view_name);
    sqlx::query(&sql).execute(pool).await?;
    info!("✅ 物化视图刷新成功：{}", view_name);
    Ok(())
}

/// 定期刷新物化视图（每 5 分钟）
pub async fn start_view_refresh_scheduler(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 分钟
        
        loop {
            interval.tick().await;
            
            for view_name in &["mv_daily_traffic_stats", "mv_user_traffic_summary", "mv_protocol_distribution"] {
                if let Err(e) = refresh_materialized_view(&pool, view_name).await {
                    warn!("刷新物化视图 {} 失败：{}", view_name, e);
                }
            }
        }
    });
}

/// 查询优化辅助函数 - 使用物化视图获取统计数据
pub async fn get_cached_traffic_stats(
    pool: &PgPool,
    inbound_id: Option<&str>,
    days: i32,
) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    let sql = if let Some(id) = inbound_id {
        r#"
        SELECT 
            stat_date::text as date,
            COALESCE(SUM(total_upload), 0) as upload,
            COALESCE(SUM(total_download), 0) as download
        FROM mv_daily_traffic_stats
        WHERE inbound_id = $1 
          AND stat_date >= NOW() - INTERVAL '1 day' * $2
        GROUP BY stat_date
        ORDER BY stat_date DESC
        "#
    } else {
        r#"
        SELECT 
            stat_date::text as date,
            COALESCE(SUM(total_upload), 0) as upload,
            COALESCE(SUM(total_download), 0) as download
        FROM mv_daily_traffic_stats
        WHERE stat_date >= NOW() - INTERVAL '1 day' * $1
        GROUP BY stat_date
        ORDER BY stat_date DESC
        "#
    };

    let records = if let Some(id) = inbound_id {
        sqlx::query_as::<_, (String, i64, i64)>(sql)
            .bind(id)
            .bind(days)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, (String, i64, i64)>(sql)
            .bind(days)
            .fetch_all(pool)
            .await?
    };

    Ok(records)
}

/// 分析查询性能
pub async fn analyze_slow_queries(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("🔍 分析慢查询...");

    // 启用 pg_stat_statements 扩展（如果未启用）
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pg_stat_statements")
        .execute(pool)
        .await?;

    // 查询最慢的 10 个查询
    let slow_queries = sqlx::query(
        r#"
        SELECT 
            query,
            calls,
            total_exec_time,
            mean_exec_time,
            rows
        FROM pg_stat_statements
        ORDER BY mean_exec_time DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in slow_queries {
        let query: String = row.get("query");
        let mean_time: f64 = row.get("mean_exec_time");
        let calls: i64 = row.get("calls");
        
        if mean_time > 100.0 {
            warn!("⚠️ 慢查询 (平均 {:.2}ms, 调用 {} 次): {}", mean_time, calls, query[..200].to_string());
        }
    }

    Ok(())
}

/// 清理过期数据
pub async fn cleanup_old_data(
    pool: &PgPool,
    retention_days: i32,
) -> Result<i64, sqlx::Error> {
    info!("🧹 清理 {} 天前的旧数据...", retention_days);

    let result = sqlx::query(
        "DELETE FROM traffic_logs WHERE recorded_at < NOW() - INTERVAL '1 day' * $1"
    )
    .bind(retention_days)
    .execute(pool)
    .await?;

    let deleted = result.rows_affected() as i64;
    info!("✅ 清理完成，删除 {} 条记录", deleted);

    Ok(deleted)
}
