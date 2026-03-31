// 节点 API 客户端
// 用于与远程 Xray 节点通信

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};

/// 节点 API 客户端
pub struct NodeClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeHealthResponse {
    pub status: String,
    pub version: Option<String>,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
    pub connection_count: Option<i32>,
    pub uptime: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfigRequest {
    pub config_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub items_synced: Option<i32>,
}

impl NodeClient {
    /// 创建新的节点客户端
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url,
            api_key,
        }
    }

    /// 检查节点健康状态
    pub async fn check_health(&self) -> Result<NodeHealthResponse, String> {
        let url = format!("{}/api/health", self.base_url.trim_end_matches('/'));
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if response.status().is_success() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("Health check failed: {}", response.status()))
        }
    }

    /// 同步配置到节点
    pub async fn sync_config(&self, config_data: serde_json::Value) -> Result<SyncResponse, String> {
        let url = format!("{}/api/sync/config", self.base_url.trim_end_matches('/'));
        
        let request = SyncConfigRequest {
            config_type: "full".to_string(),
            data: config_data,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if response.status().is_success() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("Sync failed: {}", response.status()))
        }
    }

    /// 同步用户数据到节点
    pub async fn sync_users(&self, users_data: serde_json::Value) -> Result<SyncResponse, String> {
        let url = format!("{}/api/sync/users", self.base_url.trim_end_matches('/'));
        
        let request = SyncConfigRequest {
            config_type: "users".to_string(),
            data: users_data,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if response.status().is_success() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("User sync failed: {}", response.status()))
        }
    }

    /// 同步入站配置到节点
    pub async fn sync_inbounds(&self, inbounds_data: serde_json::Value) -> Result<SyncResponse, String> {
        let url = format!("{}/api/sync/inbounds", self.base_url.trim_end_matches('/'));
        
        let request = SyncConfigRequest {
            config_type: "inbounds".to_string(),
            data: inbounds_data,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if response.status().is_success() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("Inbound sync failed: {}", response.status()))
        }
    }

    /// 获取节点统计信息
    pub async fn get_stats(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/stats", self.base_url.trim_end_matches('/'));
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if response.status().is_success() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("Get stats failed: {}", response.status()))
        }
    }

    /// 重启节点 Xray 服务
    pub async fn restart_xray(&self) -> Result<bool, String> {
        let url = format!("{}/api/system/restart", self.base_url.trim_end_matches('/'));
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        Ok(response.status().is_success())
    }
}
