//! P19 高可用集群 - 健康检查
//! 
//! 实现集群节点和服务的健康检查机制

use crate::cluster::types::*;
use etcd_client::Client;
use sqlx::PgPool;
use std::time::Instant;
use tracing::{info, warn, error, debug};

/// 健康检查器
pub struct HealthChecker {
    /// 当前节点 ID
    node_id: NodeId,
    /// etcd 客户端
    etcd_client: Client,
    /// 数据库连接池
    db_pool: PgPool,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(node_id: NodeId, etcd_client: Client, db_pool: PgPool) -> Self {
        Self {
            node_id,
            etcd_client,
            db_pool,
        }
    }

    /// 执行全面健康检查
    pub async fn check_health(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        let mut healthy = true;
        let mut errors = Vec::new();
        
        // 检查 etcd 连接
        let etcd_healthy = match self.check_etcd().await {
            Ok(_) => true,
            Err(e) => {
                healthy = false;
                errors.push(format!("etcd: {}", e));
                false
            }
        };
        
        // 检查数据库连接
        let db_healthy = match self.check_database().await {
            Ok(_) => true,
            Err(e) => {
                healthy = false;
                errors.push(format!("database: {}", e));
                false
            }
        };
        
        // 检查 Xray 服务
        let xray_healthy = match self.check_xray().await {
            Ok(_) => true,
            Err(e) => {
                // Xray 不健康不影响节点整体健康
                warn!("Xray health check failed: {}", e);
                false
            }
        };
        
        let response_time_ms = start.elapsed().as_millis() as u64;
        
        HealthCheckResult {
            node_id: self.node_id,
            healthy,
            etcd_healthy,
            db_healthy,
            xray_healthy,
            response_time_ms,
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        }
    }

    /// 检查 etcd 连接
    async fn check_etcd(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 尝试获取一个键来验证连接
        // etcd-client 的 Client 实现了 Clone，可以安全地在 async 中使用
        let client = self.etcd_client.clone();
        let mut client = client;
        client
            .get("/lx-proxy/health_check", None)
            .await?;
        Ok(())
    }

    /// 检查数据库连接
    async fn check_database(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 执行简单的查询来验证连接
        sqlx::query("SELECT 1")
            .fetch_one(&self.db_pool)
            .await?;
        Ok(())
    }

    /// 检查 Xray 服务
    async fn check_xray(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 实现 Xray 服务健康检查
        // 目前假设 Xray 总是健康的
        Ok(())
    }

    /// 批量检查所有节点健康状态
    pub async fn check_all_nodes_health(
        &self,
        nodes: &[ClusterNode],
    ) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();
        
        for node in nodes {
            let result = HealthCheckResult {
                node_id: node.id,
                healthy: node.is_alive() && node.status == NodeStatus::Running,
                etcd_healthy: true, // 假设 etcd 正常
                db_healthy: true,   // 假设数据库正常
                xray_healthy: true, // 假设 Xray 正常
                response_time_ms: 0,
                error: None,
            };
            results.push(result);
        }
        
        results
    }

    /// 获取集群健康摘要
    pub async fn get_cluster_health_summary(
        &self,
        nodes: &[ClusterNode],
    ) -> ClusterHealthSummary {
        let results = self.check_all_nodes_health(nodes).await;
        
        let total = results.len();
        let healthy = results.iter().filter(|r| r.healthy).count();
        let unhealthy = total - healthy;
        
        ClusterHealthSummary {
            total_nodes: total,
            healthy_nodes: healthy,
            unhealthy_nodes: unhealthy,
            health_percentage: if total > 0 {
                (healthy as f32 / total as f32) * 100.0
            } else {
                0.0
            },
            node_results: results,
            checked_at: chrono::Utc::now(),
        }
    }

    /// 运行定期健康检查
    pub async fn run_health_check_loop(
        &self,
        check_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::time::{interval, Duration};
        
        let mut check_interval = interval(Duration::from_secs(check_interval_secs));
        
        loop {
            check_interval.tick().await;
            
            let result = self.check_health().await;
            
            if result.healthy {
                debug!("Health check passed ({}ms)", result.response_time_ms);
            } else {
                warn!(
                    "Health check failed: {} ({}ms)",
                    result.error.as_ref().unwrap_or(&"unknown".to_string()),
                    result.response_time_ms
                );
            }
        }
    }
}

/// 集群健康摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthSummary {
    /// 节点总数
    pub total_nodes: usize,
    /// 健康节点数
    pub healthy_nodes: usize,
    /// 不健康节点数
    pub unhealthy_nodes: usize,
    /// 健康百分比
    pub health_percentage: f32,
    /// 各节点检查结果
    pub node_results: Vec<HealthCheckResult>,
    /// 检查时间
    pub checked_at: DateTime<Utc>,
}

impl ClusterHealthSummary {
    /// 判断集群是否健康（超过 50% 节点健康）
    pub fn is_healthy(&self) -> bool {
        self.healthy_nodes > self.total_nodes / 2
    }

    /// 判断集群是否严重不健康（超过 50% 节点不健康）
    pub fn is_critical(&self) -> bool {
        self.unhealthy_nodes > self.total_nodes / 2
    }
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// 检查间隔（秒）
    pub check_interval_secs: u64,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 重试次数
    pub retry_count: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            timeout_secs: 10,
            retry_count: 3,
        }
    }
}
