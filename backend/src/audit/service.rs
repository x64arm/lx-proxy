// 审计日志服务

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;
use crate::audit::{AuditLog, AuditLogQuery, AuditLogListResponse, AuditStats, ActionCount, DayCount, LoginLog, ConfigChangeHistory};

/// 审计日志服务
pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    /// 创建新的审计服务
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 记录审计日志
    pub async fn log(
        &self,
        user_id: Option<Uuid>,
        username: Option<String>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_method: Option<&str>,
        request_path: Option<&str>,
        request_body: Option<Value>,
        response_status: Option<i32>,
        response_body: Option<Value>,
        duration_ms: Option<i32>,
        status: &str,
        error_message: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<AuditLog, String> {
        let log: AuditLog = sqlx::query_as(
            r#"INSERT INTO audit_logs 
               (user_id, username, action, resource_type, resource_id, ip_address, user_agent,
                request_method, request_path, request_body, response_status, response_body,
                duration_ms, status, error_message, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
               RETURNING *"#
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_method)
        .bind(request_path)
        .bind(request_body)
        .bind(response_status)
        .bind(response_body)
        .bind(duration_ms)
        .bind(status)
        .bind(error_message)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(log)
    }

    /// 记录登录日志
    pub async fn log_login(
        &self,
        user_id: Option<Uuid>,
        username: Option<String>,
        ip_address: &str,
        user_agent: Option<&str>,
        status: &str,
        failure_reason: Option<&str>,
        session_id: Option<Uuid>,
    ) -> Result<LoginLog, String> {
        let log: LoginLog = sqlx::query_as(
            r#"INSERT INTO login_logs 
               (user_id, username, ip_address, user_agent, status, failure_reason, session_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#
        )
        .bind(user_id)
        .bind(username)
        .bind(ip_address)
        .bind(user_agent)
        .bind(status)
        .bind(failure_reason)
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(log)
    }

    /// 记录配置变更
    pub async fn log_config_change(
        &self,
        user_id: Option<Uuid>,
        username: Option<String>,
        config_type: &str,
        config_id: Option<Uuid>,
        action: &str,
        old_value: Option<Value>,
        new_value: Option<Value>,
        changes_summary: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<ConfigChangeHistory, String> {
        let log: ConfigChangeHistory = sqlx::query_as(
            r#"INSERT INTO config_change_history 
               (user_id, username, config_type, config_id, action, old_value, new_value, 
                changes_summary, ip_address)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#
        )
        .bind(user_id)
        .bind(username)
        .bind(config_type)
        .bind(config_id)
        .bind(action)
        .bind(old_value)
        .bind(new_value)
        .bind(changes_summary)
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(log)
    }

    /// 查询审计日志（简化版本）
    pub async fn query_logs(&self, query: AuditLogQuery) -> Result<AuditLogListResponse, String> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 简单查询，支持基本过滤
        let action_ref = query.action.as_ref();
        let status_ref = query.status.as_ref();
        
        let logs: Vec<AuditLog> = sqlx::query_as(
            r#"SELECT * FROM audit_logs 
               WHERE ($1::uuid IS NULL OR user_id = $1)
               AND ($2::text IS NULL OR action = $2)
               AND ($3::text IS NULL OR status = $3)
               ORDER BY created_at DESC
               LIMIT $4 OFFSET $5"#
        )
        .bind(query.user_id)
        .bind(action_ref)
        .bind(status_ref)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 获取总数
        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM audit_logs 
               WHERE ($1::uuid IS NULL OR user_id = $1)
               AND ($2::text IS NULL OR action = $2)
               AND ($3::text IS NULL OR status = $3)"#
        )
        .bind(query.user_id)
        .bind(action_ref)
        .bind(status_ref)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(AuditLogListResponse {
            logs,
            total: total.0,
            page,
            page_size,
        })
    }

    /// 获取审计统计信息
    pub async fn get_stats(&self, days: i32) -> Result<AuditStats, String> {
        // 总操作数
        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM audit_logs WHERE created_at > NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 成功操作数
        let successful: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM audit_logs 
               WHERE status = 'success' AND created_at > NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 失败操作数
        let failed: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM audit_logs 
               WHERE status = 'failure' AND created_at > NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 独立用户数
        let unique_users: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(DISTINCT user_id) FROM audit_logs 
               WHERE user_id IS NOT NULL AND created_at > NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 独立 IP 数
        let unique_ips: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(DISTINCT ip_address) FROM audit_logs 
               WHERE ip_address IS NOT NULL AND created_at > NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 热门操作
        let top_actions: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT action, COUNT(*) as count FROM audit_logs 
               WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
               GROUP BY action ORDER BY count DESC LIMIT 10"#
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // 每日活动
        let actions_by_day: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT DATE(created_at)::text as date, COUNT(*) as count FROM audit_logs 
               WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
               GROUP BY DATE(created_at) ORDER BY date DESC"#
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(AuditStats {
            total_actions: total.0,
            successful_actions: successful.0,
            failed_actions: failed.0,
            unique_users: unique_users.0,
            unique_ips: unique_ips.0,
            top_actions: top_actions.into_iter().map(|(action, count)| ActionCount { action, count }).collect(),
            actions_by_day: actions_by_day.into_iter().map(|(date, count)| DayCount { date, count }).collect(),
        })
    }

    /// 清理旧日志（保留指定天数）
    pub async fn cleanup_old_logs(&self, retention_days: i32) -> Result<u64, String> {
        let result = sqlx::query(
            r#"DELETE FROM audit_logs WHERE created_at < NOW() - ($1 || ' days')::INTERVAL"#
        )
        .bind(retention_days)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected())
    }
}
