// P14 插件系统 API 处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{AppState, plugins::PluginManager};

/// 插件列表响应
#[derive(Debug, Serialize)]
pub struct PluginListResponse {
    pub plugins: Vec<crate::plugins::PluginMetadata>,
    pub total: usize,
}

/// 切换插件请求
#[derive(Debug, Deserialize)]
pub struct TogglePluginRequest {
    pub enabled: bool,
}

/// 更新配置请求
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub config: serde_json::Value,
}

/// 列出所有插件
pub async fn list_plugins(
    State(state): State<AppState>,
) -> Result<Json<PluginListResponse>, StatusCode> {
    let manager = PluginManager::new(state.plugin_registry.clone(), state.pool);
    
    match manager.list_plugins().await {
        Ok(plugins) => Ok(Json(PluginListResponse {
            total: plugins.len(),
            plugins,
        })),
        Err(e) => {
            tracing::error!("Failed to list plugins: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取单个插件详情
pub async fn get_plugin(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<crate::plugins::PluginMetadata>, StatusCode> {
    let manager = PluginManager::new(state.plugin_registry.clone(), state.pool);
    
    match manager.get_plugin(&plugin_id).await {
        Ok(Some(plugin)) => Ok(Json(plugin)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get plugin: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 启用/禁用插件
pub async fn toggle_plugin(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
    Json(request): Json<TogglePluginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let manager = PluginManager::new(state.plugin_registry.clone(), state.pool);
    
    match manager.toggle_plugin(&plugin_id, request.enabled).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Plugin {} {}", plugin_id, if request.enabled { "enabled" } else { "disabled" })
        }))),
        Err(e) => {
            tracing::error!("Failed to toggle plugin: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新插件配置
pub async fn update_config(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let manager = PluginManager::new(state.plugin_registry.clone(), state.pool);
    
    match manager.update_config(&plugin_id, request.config).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Configuration updated successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to update plugin config: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// 测试插件功能
pub async fn test_plugin(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let manager = PluginManager::new(state.plugin_registry.clone(), state.pool);
    
    match manager.test_plugin(&plugin_id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Failed to test plugin: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
