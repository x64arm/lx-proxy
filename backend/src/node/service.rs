// 节点管理服务

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error, warn};
use crate::node::{Node, NodeClient, CreateNodeRequest, UpdateNodeRequest, NodeListResponse};

/// 节点管理服务
pub struct NodeService {
    pool: PgPool,
}

impl NodeService {
    /// 创建新的节点服务
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有节点列表
    pub async fn list_nodes(&self) -> Result<NodeListResponse, String> {
        let nodes: Vec<Node> = sqlx::query_as(
            r#"SELECT * FROM nodes ORDER BY is_primary DESC, name"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let total = nodes.len() as i64;
        let online_count = nodes.iter().filter(|n| n.status == "online").count() as i64;
        let offline_count = total - online_count;

        Ok(NodeListResponse {
            nodes,
            total,
            online_count,
            offline_count,
        })
    }

    /// 获取单个节点详情
    pub async fn get_node(&self, node_id: Uuid) -> Result<Option<Node>, String> {
        let node: Option<Node> = sqlx::query_as(
            r#"SELECT * FROM nodes WHERE id = $1 LIMIT 1"#
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(node)
    }

    /// 创建节点
    pub async fn create_node(&self, req: CreateNodeRequest) -> Result<Node, String> {
        // 检查名称是否已存在
        let existing: Option<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM nodes WHERE name = $1 LIMIT 1"#
        )
        .bind(&req.name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        if existing.is_some() {
            return Err(format!("Node name '{}' already exists", req.name));
        }

        // 如果是主节点，先取消其他主节点
        if req.is_primary.unwrap_or(false) {
            sqlx::query(
                r#"UPDATE nodes SET is_primary = FALSE, updated_at = NOW() WHERE is_primary = TRUE"#
            )
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        }

        // 创建节点
        let node: Node = sqlx::query_as(
            r#"INSERT INTO nodes (name, description, api_url, api_key, location, is_primary, is_active, status, sync_status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'offline', 'pending')
               RETURNING *"#
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.api_url)
        .bind(&req.api_key)
        .bind(&req.location)
        .bind(req.is_primary.unwrap_or(false))
        .bind(true)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        info!("Created node: {} ({})", node.name, node.id);
        Ok(node)
    }

    /// 更新节点
    pub async fn update_node(&self, node_id: Uuid, req: UpdateNodeRequest) -> Result<Node, String> {
        // 获取当前节点
        let current = self.get_node(node_id).await?
            .ok_or_else(|| "Node not found".to_string())?;

        // 如果是设置为主节点，先取消其他主节点
        if let Some(true) = req.is_primary {
            sqlx::query(
                r#"UPDATE nodes SET is_primary = FALSE, updated_at = NOW() WHERE is_primary = TRUE AND id != $1"#
            )
            .bind(node_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        }

        // 更新节点
        let node: Node = sqlx::query_as(
            r#"UPDATE nodes SET 
                   name = COALESCE($1, name),
                   description = COALESCE($2, description),
                   api_url = COALESCE($3, api_url),
                   api_key = COALESCE($4, api_key),
                   location = COALESCE($5, location),
                   is_active = COALESCE($6, is_active),
                   is_primary = COALESCE($7, is_primary),
                   updated_at = NOW()
               WHERE id = $8 RETURNING *"#
        )
        .bind(req.name)
        .bind(req.description)
        .bind(req.api_url)
        .bind(req.api_key)
        .bind(req.location)
        .bind(req.is_active)
        .bind(req.is_primary)
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        info!("Updated node: {} ({})", node.name, node.id);
        Ok(node)
    }

    /// 删除节点
    pub async fn delete_node(&self, node_id: Uuid) -> Result<bool, String> {
        // 检查是否是主节点
        let node = self.get_node(node_id).await?;
        if let Some(n) = &node {
            if n.is_primary {
                return Err("Cannot delete primary node".to_string());
            }
        }

        let result = sqlx::query(
            r#"DELETE FROM nodes WHERE id = $1"#
        )
        .bind(node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    /// 检查节点健康状态
    pub async fn check_node_health(&self, node_id: Uuid) -> Result<bool, String> {
        let node = self.get_node(node_id).await?
            .ok_or_else(|| "Node not found".to_string())?;

        let client = NodeClient::new(node.api_url.clone(), node.api_key.clone());
        
        match client.check_health().await {
            Ok(health) => {
                // 更新节点状态
                let status = if health.status == "online" { "online" } else { "offline" };
                
                sqlx::query(
                    r#"UPDATE nodes SET status = $1, last_seen = NOW(), 
                       cpu_usage = $2, memory_usage = $3, disk_usage = $4,
                       connection_count = $5, version = $6, updated_at = NOW()
                       WHERE id = $7"#
                )
                .bind(status)
                .bind(health.cpu_usage)
                .bind(health.memory_usage)
                .bind(health.disk_usage)
                .bind(health.connection_count)
                .bind(health.version)
                .bind(node_id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;

                // 记录健康日志
                sqlx::query(
                    r#"INSERT INTO node_health_logs (node_id, status, response_time_ms, cpu_usage, memory_usage, disk_usage, connection_count)
                       VALUES ($1, $2, $3, $4, $5, $6, $7)"#
                )
                .bind(node_id)
                .bind(&status)
                .bind(Some(100)) // 模拟响应时间
                .bind(health.cpu_usage)
                .bind(health.memory_usage)
                .bind(health.disk_usage)
                .bind(health.connection_count)
                .execute(&self.pool)
                .await
                .ok();

                Ok(true)
            }
            Err(e) => {
                // 更新为离线状态
                sqlx::query(
                    r#"UPDATE nodes SET status = 'offline', last_seen = NOW(), updated_at = NOW() WHERE id = $1"#
                )
                .bind(node_id)
                .execute(&self.pool)
                .await
                .ok();

                error!("Node {} health check failed: {}", node_id, e);
                Ok(false)
            }
        }
    }

    /// 同步配置到节点
    pub async fn sync_to_node(&self, node_id: Uuid, config_data: serde_json::Value) -> Result<bool, String> {
        let node = self.get_node(node_id).await?
            .ok_or_else(|| "Node not found".to_string())?;

        // 更新同步状态为进行中
        sqlx::query(
            r#"UPDATE nodes SET sync_status = 'syncing', updated_at = NOW() WHERE id = $1"#
        )
        .bind(node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let client = NodeClient::new(node.api_url.clone(), node.api_key.clone());
        
        let start_time = Utc::now();
        
        match client.sync_config(config_data).await {
            Ok(response) => {
                let duration = Utc::now().signed_duration_since(start_time).num_milliseconds() as i32;
                
                // 更新同步状态为成功
                sqlx::query(
                    r#"UPDATE nodes SET sync_status = 'synced', last_sync_at = NOW(), updated_at = NOW() WHERE id = $1"#
                )
                .bind(node_id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;

                // 记录同步历史
                sqlx::query(
                    r#"INSERT INTO node_sync_history (node_id, sync_type, status, items_synced, duration_ms, completed_at)
                       VALUES ($1, 'full', 'success', $2, $3, NOW())"#
                )
                .bind(node_id)
                .bind(response.items_synced.unwrap_or(0))
                .bind(duration)
                .execute(&self.pool)
                .await
                .ok();

                info!("Synced config to node {}: {} items", node_id, response.items_synced.unwrap_or(0));
                Ok(true)
            }
            Err(e) => {
                // 更新同步状态为失败
                sqlx::query(
                    r#"UPDATE nodes SET sync_status = 'failed', updated_at = NOW() WHERE id = $1"#
                )
                .bind(node_id)
                .execute(&self.pool)
                .await
                .ok();

                // 记录同步历史
                sqlx::query(
                    r#"INSERT INTO node_sync_history (node_id, sync_type, status, error_message, completed_at)
                       VALUES ($1, 'full', 'failed', $2, NOW())"#
                )
                .bind(node_id)
                .bind(&e)
                .execute(&self.pool)
                .await
                .ok();

                error!("Sync to node {} failed: {}", node_id, e);
                Err(e)
            }
        }
    }

    /// 同步配置到所有活跃节点
    pub async fn sync_to_all_nodes(&self, config_data: serde_json::Value) -> Result<Vec<(Uuid, bool)>, String> {
        let nodes: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"SELECT id, api_url, api_key FROM nodes WHERE is_active = TRUE AND status = 'online' ORDER BY is_primary DESC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let mut results = Vec::new();
        
        for (node_id, api_url, api_key) in nodes {
            let client = NodeClient::new(api_url, api_key);
            let pool = self.pool.clone();
            
            match client.sync_config(config_data.clone()).await {
                Ok(response) => {
                    // 更新同步状态
                    sqlx::query(
                        r#"UPDATE nodes SET sync_status = 'synced', last_sync_at = NOW() WHERE id = $1"#
                    )
                    .bind(node_id)
                    .execute(&pool)
                    .await
                    .ok();
                    
                    results.push((node_id, true));
                }
                Err(_) => {
                    sqlx::query(
                        r#"UPDATE nodes SET sync_status = 'failed' WHERE id = $1"#
                    )
                    .bind(node_id)
                    .execute(&pool)
                    .await
                    .ok();
                    
                    results.push((node_id, false));
                }
            }
        }

        Ok(results)
    }
}
