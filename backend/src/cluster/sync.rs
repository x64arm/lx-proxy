//! P19 高可用集群 - 配置同步
//! 
//! 实现集群配置的统一管理和实时同步

use crate::cluster::types::*;
use etcd_client::{Client, GetOptions, PutOptions, WatchOptions};
use tokio::sync::broadcast;
use tracing::{info, warn, error, debug};
use sha2::{Sha256, Digest};

/// 配置同步器
pub struct ConfigSync {
    /// etcd 客户端
    client: Client,
    /// 当前节点 ID
    node_id: NodeId,
    /// 配置前缀
    config_prefix: String,
    /// 事件广播发送器
    tx: broadcast::Sender<ConfigChangeEvent>,
    /// 事件广播接收器（用于本地订阅）
    _rx: broadcast::Receiver<ConfigChangeEvent>,
}

impl ConfigSync {
    /// 创建新的配置同步器
    pub fn new(client: Client, node_id: NodeId, config_prefix: String) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        
        Self {
            client,
            node_id,
            config_prefix,
            tx,
            _rx,
        }
    }

    /// 发布配置变更
    pub async fn publish_config(
        &self,
        config_type: &str,
        data: serde_json::Value,
        version: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = format!("{}/{}", self.config_prefix, config_type);
        
        let event = ConfigChangeEvent {
            config_type: config_type.to_string(),
            version,
            data: data.clone(),
            changed_by: self.node_id,
            timestamp: chrono::Utc::now(),
        };
        
        let value = serde_json::to_string(&event)?;
        let mut client = self.client.clone();
        client.put(&key, value, Some(PutOptions::new())).await?;
        
        // 本地广播
        let _ = self.tx.send(event);
        
        info!("Published config change: {} (version {})", config_type, version);
        Ok(())
    }

    /// 获取配置
    pub async fn get_config(
        &self,
        config_type: &str,
    ) -> Result<Option<ConfigChangeEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let key = format!("{}/{}", self.config_prefix, config_type);
        
        let mut client = self.client.clone();
        let resp = client.get(&key, None).await?;
        
        if let Some(kv) = resp.kvs().first() {
            let event = serde_json::from_slice::<ConfigChangeEvent>(kv.value())?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    /// 获取所有配置
    pub async fn get_all_configs(
        &self,
    ) -> Result<Vec<ConfigChangeEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let key_prefix = format!("{}/", self.config_prefix);
        
        let mut client = self.client.clone();
        let resp = client
            .get(&key_prefix, Some(GetOptions::new().with_prefix()))
            .await?;
        
        let mut configs = Vec::new();
        for kv in resp.kvs() {
            if let Ok(event) = serde_json::from_slice::<ConfigChangeEvent>(kv.value()) {
                configs.push(event);
            }
        }
        
        Ok(configs)
    }

    /// 订阅配置变更
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.tx.subscribe()
    }

    /// 监听特定配置的变更
    pub async fn watch_config(
        &self,
        config_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = format!("{}/{}", self.config_prefix, config_type);
        
        let client = self.client.clone();
        let mut watcher = client.watch_client().await?;
        let (mut watch_stream, _) = watcher.watch(&key, Some(WatchOptions::new())).await?;
        
        while let Some(resp) = watch_stream.message().await? {
            for event in resp.events() {
                if let Some(kv) = event.kv() {
                    if let Ok(change_event) = serde_json::from_slice::<ConfigChangeEvent>(kv.value()) {
                        debug!("Watch received config change: {}", config_type);
                        let _ = self.tx.send(change_event);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 监听所有配置变更
    pub async fn watch_all_configs(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key_prefix = format!("{}/", self.config_prefix);
        
        let client = self.client.clone();
        let mut watcher = client.watch_client().await?;
        let (mut watch_stream, _) = watcher
            .watch(&key_prefix, Some(WatchOptions::new().with_prefix()))
            .await?;
        
        while let Some(resp) = watch_stream.message().await? {
            for event in resp.events() {
                if let Some(kv) = event.kv() {
                    if let Ok(change_event) = serde_json::from_slice::<ConfigChangeEvent>(kv.value()) {
                        debug!("Watch received config change: {}", change_event.config_type);
                        let _ = self.tx.send(change_event);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 删除配置
    pub async fn delete_config(
        &self,
        config_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = format!("{}/{}", self.config_prefix, config_type);
        let mut client = self.client.clone();
        client.delete(&key, None).await?;
        
        info!("Deleted config: {}", config_type);
        Ok(())
    }

    /// 计算配置 checksum
    pub fn calculate_checksum(data: &serde_json::Value) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl Clone for ConfigSync {
    fn clone(&self) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            client: self.client.clone(),
            node_id: self.node_id,
            config_prefix: self.config_prefix.clone(),
            tx,
            _rx,
        }
    }
}

/// 配置类型枚举
#[derive(Debug, Clone, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigType {
    /// 系统配置
    System,
    /// Xray 配置
    Xray,
    /// 用户配置
    User,
    /// 入站配置
    Inbound,
    /// 流量配置
    Traffic,
    /// 集群配置
    Cluster,
}

/// 配置操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "lowercase")]
pub enum ConfigOperation {
    /// 创建配置
    Create { key: String, value: serde_json::Value },
    /// 更新配置
    Update {
        key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    /// 删除配置
    Delete { key: String, old_value: serde_json::Value },
}

/// 配置同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// 最后同步时间
    pub last_sync: DateTime<Utc>,
    /// 同步的配置数量
    pub configs_synced: usize,
    /// 同步错误数量
    pub sync_errors: usize,
    /// 是否是领导者
    pub is_leader: bool,
}

impl SyncStatus {
    pub fn new() -> Self {
        Self {
            last_sync: chrono::Utc::now(),
            configs_synced: 0,
            sync_errors: 0,
            is_leader: false,
        }
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::new()
    }
}
