// P15 多节点管理 API 处理器

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{AppState, node::{NodeService, CreateNodeRequest, UpdateNodeRequest, BatchSyncRequest}};

/// 节点列表查询参数
#[derive(Debug, Deserialize)]
pub struct NodeListQuery {
    pub status: Option<String>,
    pub active: Option<bool>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

// 节点列表响应使用 models 中的 NodeListResponse

/// 列出所有节点
pub async fn list_nodes(
    State(state): State<AppState>,
    Query(query): Query<NodeListQuery>,
) -> Result<Json<crate::node::NodeListResponse>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    let mut nodes = service.list_nodes().await
        .map_err(|e| {
            tracing::error!("Failed to list nodes: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 过滤状态
    if let Some(status) = &query.status {
        nodes.nodes.retain(|n| n.status == *status);
    }

    // 过滤活跃状态
    if let Some(active) = query.active {
        nodes.nodes.retain(|n| n.is_active == active);
    }

    // 分页
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(nodes.nodes.len());
    
    let paginated_nodes = if start < nodes.nodes.len() {
        nodes.nodes[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(crate::node::NodeListResponse {
        nodes: paginated_nodes,
        total: nodes.total,
        online_count: nodes.online_count,
        offline_count: nodes.offline_count,
    }))
}

/// 获取单个节点详情
pub async fn get_node(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<crate::node::Node>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    service.get_node(node_id).await
        .map_err(|e| {
            tracing::error!("Failed to get node: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

/// 创建节点
pub async fn create_node(
    State(state): State<AppState>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<crate::node::Node>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    service.create_node(req).await
        .map_err(|e| {
            tracing::error!("Failed to create node: {}", e);
            StatusCode::BAD_REQUEST
        })
        .map(Json)
}

/// 更新节点
pub async fn update_node(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<crate::node::Node>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    service.update_node(node_id, req).await
        .map_err(|e| {
            tracing::error!("Failed to update node: {}", e);
            StatusCode::BAD_REQUEST
        })
        .map(Json)
}

/// 删除节点
pub async fn delete_node(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    service.delete_node(node_id).await
        .map_err(|e| {
            tracing::error!("Failed to delete node: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Node deleted successfully"
    })))
}

/// 检查节点健康状态
pub async fn check_health(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    let healthy = service.check_node_health(node_id).await
        .map_err(|e| {
            tracing::error!("Failed to check node health: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "node_id": node_id,
        "healthy": healthy,
        "checked_at": chrono::Utc::now()
    })))
}

/// 同步配置到节点
pub async fn sync_node(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
    Json(config_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = NodeService::new(state.pool);
    
    service.sync_to_node(node_id, config_data).await
        .map(|_| Json(serde_json::json!({
            "success": true,
            "message": "Configuration synced successfully"
        })))
        .map_err(|e| {
            tracing::error!("Failed to sync node: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// 批量同步配置到所有节点
pub async fn batch_sync(
    State(state): State<AppState>,
    Json(req): Json<BatchSyncRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 获取要同步的节点
    let nodes_to_sync: Vec<Uuid> = if let Some(node_ids) = req.node_ids {
        node_ids
    } else {
        // 获取所有活跃节点
        let all_nodes: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM nodes WHERE is_active = TRUE ORDER BY is_primary DESC"#
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        all_nodes.iter().map(|n| n.0).collect()
    };

    let service = NodeService::new(state.pool);

    // 构建配置数据
    let config_data = serde_json::json!({
        "sync_type": req.sync_type,
        "timestamp": chrono::Utc::now()
    });

    let mut results = Vec::new();
    
    for node_id in nodes_to_sync {
        match service.sync_to_node(node_id, config_data.clone()).await {
            Ok(_) => {
                results.push(serde_json::json!({
                    "node_id": node_id,
                    "success": true
                }));
            }
            Err(e) => {
                results.push(serde_json::json!({
                    "node_id": node_id,
                    "success": false,
                    "error": e
                }));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "total": results.len(),
        "successful": results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count(),
        "failed": results.iter().filter(|r| !r["success"].as_bool().unwrap_or(true)).count(),
        "results": results
    })))
}

/// 获取节点统计信息
pub async fn get_node_stats(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<crate::node::NodeStats>, StatusCode> {
    let stats: Option<crate::node::NodeStats> = sqlx::query_as(
        r#"SELECT * FROM node_stats WHERE id = $1 LIMIT 1"#
    )
    .bind(node_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    stats.ok_or(StatusCode::NOT_FOUND).map(Json)
}

/// 获取所有节点统计信息
pub async fn get_all_nodes_stats(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::node::NodeStats>>, StatusCode> {
    let stats: Vec<crate::node::NodeStats> = sqlx::query_as(
        r#"SELECT * FROM node_stats ORDER BY is_primary DESC, name"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(stats))
}
