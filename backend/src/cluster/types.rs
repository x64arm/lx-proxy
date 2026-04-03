//! P19 高可用集群 - 类型定义（简化版）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 节点 ID 类型
pub type NodeId = Uuid;

/// 节点角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Leader => write!(f, "leader"),
            NodeRole::Follower => write!(f, "follower"),
            NodeRole::Candidate => write!(f, "candidate"),
        }
    }
}

/// 节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Starting,
    Running,
    Stopping,
    Unreachable,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Starting => write!(f, "starting"),
            NodeStatus::Running => write!(f, "running"),
            NodeStatus::Stopping => write!(f, "stopping"),
            NodeStatus::Unreachable => write!(f, "unreachable"),
        }
    }
}

/// 集群配置（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub etcd_endpoints: Vec<String>,
    pub heartbeat_interval: u64,
    pub election_timeout: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
            heartbeat_interval: 10,
            election_timeout: 30,
        }
    }
}

impl ClusterConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}

/// 集群节点信息（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub id: NodeId,
    pub name: String,
    pub address: String,
    pub role: NodeRole,
    pub status: NodeStatus,
    pub load_avg: f32,
    pub started_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Default for ClusterNode {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            address: "127.0.0.1:0".to_string(),
            role: NodeRole::Candidate,
            status: NodeStatus::Starting,
            load_avg: 0.0,
            started_at: now,
            created_at: now,
        }
    }
}

/// 领导者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderInfo {
    pub node_id: NodeId,
    pub elected_at: DateTime<Utc>,
    pub address: String,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub node_id: NodeId,
    pub healthy: bool,
    pub etcd_healthy: bool,
    pub db_healthy: bool,
    pub xray_healthy: bool,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub config_type: String,
    pub version: u64,
    pub data: serde_json::Value,
    pub changed_by: NodeId,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_role_display() {
        assert_eq!(NodeRole::Leader.to_string(), "leader");
    }

    #[test]
    fn test_cluster_node_default() {
        let node = ClusterNode::default();
        assert_eq!(node.role, NodeRole::Candidate);
        assert_eq!(node.status, NodeStatus::Starting);
    }
}
