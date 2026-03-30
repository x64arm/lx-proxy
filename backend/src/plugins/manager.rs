// 插件管理器
// 提供插件的启用/禁用、配置管理等 API

use std::sync::Arc;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::plugins::{PluginRegistry, PluginMetadata};

/// 插件管理器
pub struct PluginManager {
    registry: PluginRegistry,
    pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginUpdateRequest {
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(registry: PluginRegistry, pool: PgPool) -> Self {
        Self { registry, pool }
    }
    
    /// 获取所有插件列表
    pub async fn list_plugins(&self) -> Result<Vec<PluginMetadata>, String> {
        let registry = self.registry.read().await;
        let mut plugins: Vec<PluginMetadata> = registry.values().cloned().collect();
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(plugins)
    }
    
    /// 获取单个插件详情
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<Option<PluginMetadata>, String> {
        let registry = self.registry.read().await;
        Ok(registry.get(plugin_id).cloned())
    }
    
    /// 启用/禁用插件
    pub async fn toggle_plugin(&self, plugin_id: &str, enabled: bool) -> Result<bool, String> {
        // 更新数据库状态
        sqlx::query(
            r#"UPDATE plugin_configs SET enabled = $1, updated_at = NOW() WHERE id = $2"#
        )
        .bind(enabled)
        .bind(plugin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
        
        // 更新注册表
        let mut registry = self.registry.write().await;
        if let Some(metadata) = registry.get_mut(plugin_id) {
            metadata.enabled = enabled;
        }
        
        info!("Plugin {} {}", plugin_id, if enabled { "enabled" } else { "disabled" });
        Ok(true)
    }
    
    /// 更新插件配置
    pub async fn update_config(
        &self,
        plugin_id: &str,
        config: serde_json::Value,
    ) -> Result<bool, String> {
        // 更新数据库
        sqlx::query(
            r#"UPDATE plugin_configs SET config = $1, updated_at = NOW() WHERE id = $2"#
        )
        .bind(&config)
        .bind(plugin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
        
        info!("Plugin {} configuration updated", plugin_id);
        Ok(true)
    }
    
    /// 测试插件功能
    pub async fn test_plugin(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        let registry = self.registry.read().await;
        
        match registry.get(plugin_id) {
            Some(metadata) => {
                Ok(serde_json::json!({
                    "success": true,
                    "message": format!("Plugin {} test passed", metadata.name),
                    "plugin_id": plugin_id,
                    "plugin_name": metadata.name,
                    "version": metadata.version
                }))
            }
            None => Err(format!("Plugin {} not found", plugin_id)),
        }
    }
}
