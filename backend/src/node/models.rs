// 节点管理数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 节点状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Online,
    Offline,
    Error,
}

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Failed,
}

/// 节点配置
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub api_url: String,
    #[sqlx(skip)]
    pub api_key: String,
    pub status: String,
    pub location: Option<String>,
    pub version: Option<String>,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
    pub bandwidth_upload: Option<i64>,
    pub bandwidth_download: Option<i64>,
    pub connection_count: Option<i32>,
    pub last_seen: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub is_active: bool,
    pub sync_status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建节点请求
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
    pub description: Option<String>,
    pub api_url: String,
    pub api_key: String,
    pub location: Option<String>,
    pub is_primary: Option<bool>,
}

/// 更新节点请求
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub location: Option<String>,
    pub is_active: Option<bool>,
    pub is_primary: Option<bool>,
}

/// 节点统计信息
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NodeStats {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub is_primary: bool,
    pub is_active: bool,
    pub inbound_count: i64,
    pub enabled_inbound_count: i64,
    pub total_traffic_used: i64,
    pub total_connections: i64,
    pub last_seen: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

/// 节点健康状态
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: Uuid,
    pub status: String,
    pub response_time_ms: Option<i32>,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
    pub connection_count: Option<i32>,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// 同步历史
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncHistory {
    pub id: Uuid,
    pub node_id: Uuid,
    pub sync_type: String,
    pub status: String,
    pub items_synced: i32,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 批量同步请求
#[derive(Debug, Deserialize)]
pub struct BatchSyncRequest {
    pub node_ids: Option<Vec<Uuid>>,
    pub sync_type: String,  // full, incremental, config, user
}

/// 节点列表响应
#[derive(Debug, Serialize)]
pub struct NodeListResponse {
    pub nodes: Vec<Node>,
    pub total: i64,
    pub online_count: i64,
    pub offline_count: i64,
}
