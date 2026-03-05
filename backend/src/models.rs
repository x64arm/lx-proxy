use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub role: String,
    pub exp: usize,
}

/// 入站配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundConfig {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub tag: String,
    pub protocol: String,
    pub port: i32,
    pub settings: serde_json::Value,
    pub stream_settings: Option<serde_json::Value>,
    pub sniffing: Option<serde_json::Value>,
    pub enable: bool,
    pub traffic_used: i64,
    pub traffic_limit: Option<i64>,
    pub expire_at: Option<DateTime<Utc>>,
    pub ip_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建入站配置请求
#[derive(Debug, Deserialize)]
pub struct CreateInboundRequest {
    pub user_id: Option<Uuid>,
    pub tag: String,
    pub protocol: String,
    pub port: i32,
    pub settings: serde_json::Value,
    pub stream_settings: Option<serde_json::Value>,
    pub sniffing: Option<serde_json::Value>,
    pub traffic_limit: Option<i64>,
    pub expire_at: Option<DateTime<Utc>>,
    pub ip_limit: Option<i32>,
}

/// 更新入站配置请求
#[derive(Debug, Deserialize)]
pub struct UpdateInboundRequest {
    pub tag: Option<String>,
    pub port: Option<i32>,
    pub settings: Option<serde_json::Value>,
    pub stream_settings: Option<serde_json::Value>,
    pub sniffing: Option<serde_json::Value>,
    pub enable: Option<bool>,
    pub traffic_limit: Option<i64>,
    pub expire_at: Option<DateTime<Utc>>,
    pub ip_limit: Option<i32>,
}

/// 流量统计
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrafficLog {
    pub id: Uuid,
    pub inbound_id: Uuid,
    pub upload: i64,
    pub download: i64,
    pub recorded_at: DateTime<Utc>,
}

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// 系统状态
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_free: u64,
    pub uptime: u64,
    pub xray_running: bool,
    pub connections: u64,
}

/// 统计数据
#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub total_users: i64,
    pub total_inbounds: i64,
    pub enabled_inbounds: i64,
    pub total_traffic_used: i64,
    pub total_traffic_limit: Option<i64>,
}
