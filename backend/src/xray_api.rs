/// Xray API 客户端模块
/// 用于与 Xray-core 的 API 交互，获取流量统计、控制服务等

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Xray API 客户端
pub struct XrayApiClient {
    client: Client,
    base_url: String,
}

/// Xray 流量统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayTrafficStats {
    pub name: String,
    pub value: i64,
}

/// Xray 系统状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XraySystemStatus {
    pub num_cpu: u32,
    pub current_cpu: f32,
    pub num_goroutine: u32,
    pub alloc: u64,
    pub total_alloc: u64,
    pub sys: u64,
}

impl XrayApiClient {
    /// 创建新的 Xray API 客户端
    pub fn new(api_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: api_url.trim_end_matches('/').to_string(),
        }
    }

    /// 从环境变量获取 API URL
    pub fn from_env() -> Self {
        let api_url = std::env::var("XRAY_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:62780".to_string());
        Self::new(&api_url)
    }

    /// 获取入站流量统计
    pub async fn get_inbound_traffic(&self, tag: &str) -> Result<(i64, i64), Box<dyn std::error::Error + Send + Sync>> {
        let upload = self.get_traffic_value(&format!("inbound>>>{}>>>traffic>>>uplink", tag)).await?;
        let download = self.get_traffic_value(&format!("inbound>>>{}>>>traffic>>>downlink", tag)).await?;
        Ok((upload, download))
    }

    /// 获取单个流量值
    async fn get_traffic_value(&self, pattern: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/stats/gettraffic?pattern={}&reset=false", self.base_url, pattern);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Xray API: {}", e))?;

        if !response.status().is_success() {
            warn!("Xray API returned status {}: {}", response.status(), pattern);
            return Ok(0);
        }

        let stats: XrayTrafficStats = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Xray API response: {}", e))?;

        Ok(stats.value)
    }

    /// 获取所有流量统计
    pub async fn get_all_traffic_stats(&self) -> Result<Vec<XrayTrafficStats>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/stats/query?pattern=", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Xray API: {}", e))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let stats: Vec<XrayTrafficStats> = response
            .json()
            .await
            .unwrap_or_default();

        Ok(stats)
    }

    /// 获取系统状态
    pub async fn get_system_status(&self) -> Result<XraySystemStatus, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/stats/sysinfo", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Xray API: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Xray API returned status {}", response.status()).into());
        }

        let status: XraySystemStatus = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Xray API response: {}", e))?;

        Ok(status)
    }

    /// 重启 Xray 服务
    pub async fn restart_xray(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/command/restart", self.base_url);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send restart command: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to restart Xray: {}", response.status()).into());
        }

        info!("✅ Xray restarted successfully");
        Ok(())
    }

    /// 检查 Xray API 是否可用
    pub async fn check_health(&self) -> bool {
        match self.get_system_status().await {
            Ok(_) => true,
            Err(e) => {
                warn!("Xray API health check failed: {}", e);
                false
            }
        }
    }
}

/// 获取流量统计（便捷函数）
pub async fn fetch_inbound_traffic(tag: &str) -> Result<(i64, i64), Box<dyn std::error::Error + Send + Sync>> {
    let client = XrayApiClient::from_env();
    client.get_inbound_traffic(tag).await
}
