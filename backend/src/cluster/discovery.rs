//! P19 高可用集群 - 节点发现
//! 
//! 实现集群节点的注册、发现和心跳机制

use crate::cluster::types::*;
use etcd_client::{Client, GetOptions, PutOptions, LeaseGrantOptions};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};

/// 节点发现管理器
pub struct NodeDiscovery {
    /// 当前节点 ID
    pub node_id: NodeId,
    /// etcd 客户端
    pub client: Client,
    /// 集群配置
    pub config: ClusterConfig,
    /// 当前节点信息
    local_node: Arc<RwLock<ClusterNode>>,
    /// 已知节点缓存
    known_nodes: Arc<RwLock<Vec<ClusterNode>>>,
}

impl NodeDiscovery {
    /// 创建新的节点发现管理器
    pub async fn new(
        node_id: NodeId,
        node_name: String,
        node_address: SocketAddr,
        config: ClusterConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 连接 etcd
        let client = Client::connect(config.etcd_endpoints.clone(), None).await?;
        
        let local_node = ClusterNode::new(node_name, node_address);
        
        Ok(Self {
            node_id,
            client,
            config,
            local_node: Arc::new(RwLock::new(local_node)),
            known_nodes: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 注册当前节点到集群
    pub async fn register_self(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = format!("{}/{}", self.config.node_prefix, self.node_id);
        
        // 获取当前节点信息
        let node = self.local_node.read().await;
        let value = serde_json::to_string(&*node)?;
        drop(node);
        
        // 创建 lease（30 秒过期）
        let mut client = self.client.clone();
        let lease_resp = client
            .lease_grant(30, Some(LeaseGrantOptions::new()))
            .await?;
        let lease_id = lease_resp.id();
        
        info!("Lease granted: {}", lease_id);
        
        // 注册节点（带 lease）
        let mut client = self.client.clone();
        client
            .put(
                key.as_bytes(),
                value.as_bytes(),
                Some(PutOptions::new().with_lease(lease_id)),
            )
            .await?;
        
        info!("Node {} registered to cluster", self.node_id);
        
        // 启动心跳协程
        let self_arc = Arc::new(self.clone_for_task());
        tokio::spawn(async move {
            self_arc.heartbeat_loop(lease_id).await;
        });
        
        Ok(())
    }

    /// 发现集群中的其他节点
    pub async fn discover_nodes(&self) -> Result<Vec<ClusterNode>, Box<dyn std::error::Error + Send + Sync>> {
        let key_prefix = format!("{}/", self.config.node_prefix);
        
        let mut client = self.client.clone();
        let resp = client
            .get(&key_prefix, Some(GetOptions::new().with_prefix()))
            .await?;
        
        let mut nodes = Vec::new();
        for kv in resp.kvs() {
            match serde_json::from_slice::<ClusterNode>(kv.value()) {
                Ok(node) => {
                    debug!("Discovered node: {} ({})", node.name, node.id);
                    nodes.push(node);
                }
                Err(e) => {
                    warn!("Failed to parse node info: {}", e);
                }
            }
        }
        
        // 更新已知节点缓存
        {
            let mut known = self.known_nodes.write().await;
            *known = nodes.clone();
        }
        
        info!("Discovered {} nodes in cluster", nodes.len());
        Ok(nodes)
    }

    /// 获取已知节点列表
    pub async fn get_known_nodes(&self) -> Vec<ClusterNode> {
        self.known_nodes.read().await.clone()
    }

    /// 获取当前节点信息
    pub async fn get_local_node(&self) -> ClusterNode {
        self.local_node.read().await.clone()
    }

    /// 获取领导者信息
    pub async fn get_leader(&self) -> Result<Option<crate::cluster::LeaderInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use etcd_client::GetOptions;
        
        let key = &self.config.election_prefix;
        let mut client = self.client.clone();
        let resp = client.get(key, Some(GetOptions::new())).await?;
        
        if let Some(kv) = resp.kvs().first() {
            let value = kv.value();
            if !value.is_empty() {
                if let Ok(info) = serde_json::from_slice::<crate::cluster::LeaderInfo>(value) {
                    return Ok(Some(info));
                }
            }
        }
        
        Ok(None)
    }

    /// 更新本地负载信息
    pub async fn update_local_load(&self, load: f32) {
        let mut node = self.local_node.write().await;
        node.update_load(load);
    }

    /// 更新节点状态
    pub async fn update_node_status(&self, status: NodeStatus) {
        let mut node = self.local_node.write().await;
        node.status = status;
        
        // 同步到 etcd
        let key = format!("{}/{}", self.config.node_prefix, self.node_id);
        let value = serde_json::to_string(&*node).unwrap();
        
        if let Err(e) = self.client.put(&key, value, None).await {
            error!("Failed to update node status: {}", e);
        }
    }

    /// 心跳循环
    async fn heartbeat_loop(self: Arc<Self>, lease_id: i64) {
        let mut heartbeat_interval = interval(Duration::from_secs(self.config.heartbeat_interval));
        
        loop {
            heartbeat_interval.tick().await;
            
            // 续租 lease
            let mut client = self.client.clone();
            match client.lease_keep_alive(lease_id).await {
                Ok((_keeper, mut stream)) => {
                    // 从 stream 中获取响应
                    if let Some(Ok(resp)) = stream.message().await.unwrap_or(None) {
                        if resp.ttl() <= 0 {
                            // Lease 已过期，重新注册
                            warn!("Lease expired, re-registering node");
                            if let Err(e) = self.register_self().await {
                                error!("Failed to re-register node: {}", e);
                            }
                        } else {
                            debug!("Heartbeat sent, TTL: {}", resp.ttl());
                        }
                    }
                }
                Err(e) => {
                    error!("Heartbeat failed: {}", e);
                    // 尝试重新注册
                    if let Err(e) = self.register_self().await {
                        error!("Failed to re-register after heartbeat failure: {}", e);
                    }
                }
            }
            
            // 更新本地负载信息
            let load = self.get_system_load().await;
            self.update_local_load(load).await;
            
            // 更新节点状态为运行中
            if self.local_node.read().await.status == NodeStatus::Starting {
                self.update_node_status(NodeStatus::Running).await;
            }
        }
    }

    /// 获取系统负载（简化版本，使用随机值模拟）
    async fn get_system_load(&self) -> f32 {
        // TODO: 实现真实的系统负载获取
        // 目前返回随机值用于测试
        rand::random::<f32>() * 0.5
    }

    /// 克隆用于任务的结构（移除 Arc）
    fn clone_for_task(&self) -> NodeDiscovery {
        NodeDiscovery {
            node_id: self.node_id,
            client: self.client.clone(),
            config: self.config.clone(),
            local_node: self.local_node.clone(),
            known_nodes: self.known_nodes.clone(),
        }
    }

    /// 离开集群（清理注册信息）
    pub async fn leave_cluster(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 更新状态为停止中
        self.update_node_status(NodeStatus::Stopping).await;
        
        // 删除注册信息
        let key = format!("{}/{}", self.config.node_prefix, self.node_id);
        let mut client = self.client.clone();
        client.delete(&key, None).await?;
        
        info!("Node {} left cluster", self.node_id);
        Ok(())
    }

    /// 获取集群节点数量
    pub async fn get_cluster_size(&self) -> usize {
        match self.discover_nodes().await {
            Ok(nodes) => nodes.len(),
            Err(_) => 0,
        }
    }
}

impl Clone for NodeDiscovery {
    fn clone(&self) -> Self {
        self.clone_for_task()
    }
}

/// 集群上下文（供其他模块使用）
pub struct ClusterContext {
    /// 节点发现管理器
    pub discovery: Arc<NodeDiscovery>,
    /// 当前节点 ID
    pub node_id: NodeId,
}

impl ClusterContext {
    /// 创建新的集群上下文
    pub async fn new(
        node_id: NodeId,
        node_name: String,
        node_address: SocketAddr,
        config: ClusterConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let discovery = Arc::new(NodeDiscovery::new(
            node_id,
            node_name,
            node_address,
            config.clone(),
        ).await?);
        
        Ok(Self {
            discovery,
            node_id,
        })
    }
}
