//! 集群管理 API Handler - 简化版 (MVP)

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    routing::get,
    Router,
};
use serde::Serialize;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cluster::ClusterApiState;

#[derive(Debug, Serialize)]
pub struct ClusterStatusResponse {
    pub cluster_enabled: bool,
    pub node_id: Uuid,
    pub node_name: String,
    pub message: String,
}

pub async fn get_cluster_status(
    State(state): State<Arc<RwLock<ClusterApiState>>>,
) -> Result<Json<ClusterStatusResponse>, StatusCode> {
    let state = state.read().await;
    
    match &state.context {
        Some(ctx) => Ok(Json(ClusterStatusResponse {
            cluster_enabled: ctx.is_cluster_enabled,
            node_id: ctx.node_id,
            node_name: ctx.node_name.clone(),
            message: "Cluster module initialized (MVP version). Full support coming in v2.1".to_string(),
        })),
        None => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

pub async fn check_health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "cluster": "disabled (MVP)"
    })))
}

pub fn cluster_router() -> Router<Arc<RwLock<ClusterApiState>>> {
    Router::new()
        .route("/status", get(get_cluster_status))
        .route("/health", get(check_health))
}
