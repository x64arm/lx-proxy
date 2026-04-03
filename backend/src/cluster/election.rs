//! P19 高可用集群 - 领导者选举
//! 
//! 基于 etcd 实现分布式领导者选举

use crate::cluster::types::*;
use etcd_client::{Client, Compare, CompareOp, Txn, TxnOp};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, debug};

/// 领导者选举器
pub struct LeaderElector {
    /// etcd 客户端
    client: Client,
    /// 当前节点 ID
    node_id: NodeId,
    /// 选举键前缀
    election_key: String,
    /// 当前角色
    current_role: NodeRole,
}

impl LeaderElector {
    /// 创建新的领导者选举器
    pub fn new(client: Client, node_id: NodeId, election_prefix: String) -> Self {
        Self {
            client,
            node_id,
            election_key: election_prefix,
            current_role: NodeRole::Candidate,
        }
    }

    /// 尝试成为领导者
    pub async fn try_become_leader(
        &self,
        node_address: String,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let leader_info = LeaderInfo {
            node_id: self.node_id,
            elected_at: chrono::Utc::now(),
            address: node_address,
        };
        
        let value = serde_json::to_string(&leader_info)?;
        
        // 使用事务实现原子性选举
        // 只有当 election_key 不存在或为空时才能成为领导者
        let txn = Txn::new()
            .when(vec![
                Compare::value(&self.election_key, CompareOp::Equal, "")
                    .or(Compare::value(&self.election_key, CompareOp::Equal, "null")),
            ])
            .and_then(vec![TxnOp::put(&self.election_key, value.clone())]);
        
        let resp = self.client.txn(txn).await?;
        
        if resp.succeeded() {
            info!("Node {} became leader", self.node_id);
            Ok(true)
        } else {
            debug!("Failed to become leader, key already exists");
            Ok(false)
        }
    }

    /// 获取当前领导者信息
    pub async fn get_leader(&self) -> Result<Option<LeaderInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.get(&self.election_key, None).await?;
        
        if let Some(kv) = resp.kvs().first() {
            let value = kv.value();
            if !value.is_empty() {
                match serde_json::from_slice::<LeaderInfo>(value) {
                    Ok(info) => {
                        debug!("Current leader: {}", info.node_id);
                        return Ok(Some(info));
                    }
                    Err(e) => {
                        warn!("Failed to parse leader info: {}", e);
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// 检查当前节点是否是领导者
    pub async fn is_leader(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.get_leader().await? {
            Some(leader) => Ok(leader.node_id == self.node_id),
            None => Ok(false),
        }
    }

    /// 释放领导者身份
    pub async fn resign(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 只有领导者才能辞职
        if !self.is_leader().await? {
            return Ok(());
        }
        
        self.client.delete(&self.election_key, None).await?;
        info!("Node {} resigned as leader", self.node_id);
        Ok(())
    }

    /// 强制删除领导者键（用于故障恢复）
    pub async fn force_remove_leader(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete(&self.election_key, None).await?;
        warn!("Forcibly removed leader key");
        Ok(())
    }

    /// 监听领导者变更事件
    pub async fn watch_leader(
        &self,
    ) -> Result<impl futures_util::Stream<Item = Result<Option<LeaderInfo>, Box<dyn std::error::Error + Send + Sync>>>, Box<dyn std::error::Error + Send + Sync>>
    {
        use futures_util::stream::try_unfold;
        
        let client = self.client.clone();
        let election_key = self.election_key.clone();
        let my_node_id = self.node_id;
        
        let stream = try_unfold(None, move |prev_rev| {
            let client = client.clone();
            let election_key = election_key.clone();
            async move {
                let mut watcher = client.watch_client().await?;
                let (mut watch_stream, _) = watcher.watch(&election_key, None).await?;
                
                while let Some(resp) = watch_stream.message().await? {
                    for event in resp.events() {
                        if let Some(kv) = event.kv() {
                            let leader = if !kv.value().is_empty() {
                                serde_json::from_slice::<LeaderInfo>(kv.value()).ok()
                            } else {
                                None
                            };
                            return Ok(Some((Ok(leader), Some(resp.header().unwrap().revision()))));
                        }
                    }
                }
                
                Ok(None)
            }
        });
        
        Ok(stream)
    }

    /// 运行选举循环
    pub async fn run_election_loop(
        &self,
        node_address: String,
        election_timeout: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut election_interval = interval(Duration::from_secs(election_timeout / 2));
        
        loop {
            election_interval.tick().await;
            
            // 检查当前是否有领导者
            match self.get_leader().await {
                Ok(Some(leader)) => {
                    // 已有领导者，检查是否是自己
                    if leader.node_id == self.node_id {
                        debug!("Still the leader");
                        // 续期领导者身份
                        let leader_info = LeaderInfo {
                            node_id: self.node_id,
                            elected_at: chrono::Utc::now(),
                            address: node_address,
                        };
                        let value = serde_json::to_string(&leader_info)?;
                        self.client.put(&self.election_key, value, None).await?;
                    } else {
                        debug!("{} is the current leader", leader.node_id);
                    }
                }
                Ok(None) | Err(_) => {
                    // 没有领导者或出错，尝试成为领导者
                    info!("No leader detected, attempting election");
                    match self.try_become_leader(node_address).await {
                        Ok(true) => {
                            info!("Successfully elected as leader");
                        }
                        Ok(false) => {
                            debug!("Lost election to another node");
                        }
                        Err(e) => {
                            error!("Election failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// 更新当前角色
    pub fn update_role(&mut self, role: NodeRole) {
        self.current_role = role;
    }

    /// 获取当前角色
    pub fn get_role(&self) -> NodeRole {
        self.current_role
    }
}

/// 选举结果
#[derive(Debug, Clone)]
pub enum ElectionResult {
    /// 成为领导者
    Elected,
    /// 选举失败
    Failed,
    /// 已是领导者
    AlreadyLeader,
}

/// 选举配置
#[derive(Debug, Clone)]
pub struct ElectionConfig {
    /// 选举超时时间（秒）
    pub timeout_secs: u64,
    /// 重试间隔（秒）
    pub retry_interval_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for ElectionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            retry_interval_secs: 5,
            max_retries: 3,
        }
    }
}
