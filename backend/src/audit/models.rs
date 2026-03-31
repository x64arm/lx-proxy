// 审计日志数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 审计日志记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: Option<String>,
    pub request_path: Option<String>,
    pub request_body: Option<serde_json::Value>,
    pub response_status: Option<i32>,
    pub response_body: Option<serde_json::Value>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 登录日志记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LoginLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub session_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 配置变更历史
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConfigChangeHistory {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub config_type: String,
    pub config_id: Option<Uuid>,
    pub action: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub changes_summary: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// IP 封禁记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IpBan {
    pub id: Uuid,
    pub ip_address: String,
    pub reason: Option<String>,
    pub banned_until: DateTime<Utc>,
    pub banned_at: DateTime<Utc>,
    pub banned_by: Option<Uuid>,
    pub is_active: bool,
}

/// 审计日志查询参数
#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 审计日志列表响应
#[derive(Debug, Serialize)]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 审计统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_actions: i64,
    pub successful_actions: i64,
    pub failed_actions: i64,
    pub unique_users: i64,
    pub unique_ips: i64,
    pub top_actions: Vec<ActionCount>,
    pub actions_by_day: Vec<DayCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayCount {
    pub date: String,
    pub count: i64,
}

/// 创建审计日志请求
#[derive(Debug, Deserialize)]
pub struct CreateAuditLogRequest {
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub status: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
