// P14 插件系统模块
// 支持第三方插件扩展：通知、统计、认证等

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod loader;
pub mod manager;
pub mod builtin;

pub use loader::PluginLoader;
pub use manager::PluginManager;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub config_schema: serde_json::Value,
}

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Notification,
    Statistics,
    Authentication,
    Extension,
}

/// 插件注册表
pub type PluginRegistry = Arc<RwLock<HashMap<String, PluginMetadata>>>;

/// 创建插件注册表
pub fn create_registry() -> PluginRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}
