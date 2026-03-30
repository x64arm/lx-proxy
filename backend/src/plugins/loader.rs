// 插件加载器
// 负责加载插件元数据到注册表

use std::sync::Arc;
use sqlx::PgPool;
use tracing::{info, error};
use crate::plugins::{PluginRegistry, PluginMetadata};

/// 插件加载器
pub struct PluginLoader {
    registry: PluginRegistry,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(registry: PluginRegistry) -> Self {
        Self { registry }
    }
    
    /// 从数据库加载所有插件配置
    pub async fn load_builtin_plugins(&self, pool: &PgPool) -> Result<(), String> {
        info!("🔌 Loading plugin configurations from database...");
        
        let plugins: Vec<(String, String, String, String, String, String, serde_json::Value, bool)> = 
            sqlx::query_as(
                r#"SELECT id, name, description, version, author, type, config_schema, enabled 
                   FROM plugin_configs ORDER BY name"#
            )
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        let mut registry = self.registry.write().await;
        
        for (id, name, description, version, author, plugin_type, config_schema, enabled) in plugins {
            let plugin_name = name.clone();
            let metadata = PluginMetadata {
                id: id.clone(),
                name,
                description,
                version,
                author,
                plugin_type: serde_json::from_str(&plugin_type).unwrap_or(
                    crate::plugins::PluginType::Extension
                ),
                enabled,
                config_schema,
            };
            
            registry.insert(id.clone(), metadata);
            info!("  📦 Loaded plugin: {}", plugin_name);
        }
        
        info!("✅ Plugin configurations loaded: {} plugins", registry.len());
        Ok(())
    }
}
