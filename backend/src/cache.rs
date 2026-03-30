// Redis 缓存模块

use redis::{Client, aio::ConnectionManager, AsyncCommands, RedisResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// 缓存客户端封装
#[derive(Clone)]
pub struct CacheClient {
    client: Option<Arc<Client>>,
    connection: Arc<RwLock<Option<ConnectionManager>>>,
    enabled: bool,
}

impl CacheClient {
    /// 创建新的缓存客户端
    pub fn new(redis_url: &str) -> Self {
        match Client::open(redis_url) {
            Ok(client) => {
                info!("✅ Redis 连接已配置：{}", redis_url.replace(|c: char| c.is_alphanumeric(), "*"));
                Self {
                    client: Some(Arc::new(client)),
                    connection: Arc::new(RwLock::new(None)),
                    enabled: true,
                }
            }
            Err(e) => {
                warn!("⚠️ Redis 连接失败，缓存功能将禁用：{}", e);
                Self {
                    client: None,
                    connection: Arc::new(RwLock::new(None)),
                    enabled: false,
                }
            }
        }
    }

    /// 初始化连接
    pub async fn initialize(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(client) = &self.client {
            match client.get_connection_manager().await {
                Ok(conn) => {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn);
                    info!("✅ Redis 连接管理器初始化成功");
                    Ok(())
                }
                Err(e) => {
                    error!("❌ Redis 连接管理器初始化失败：{}", e);
                    Err(format!("Redis connection failed: {}", e))
                }
            }
        } else {
            Ok(())
        }
    }

    /// 获取字符串值
    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            connection.get(key).await
        } else {
            Ok(None)
        }
    }

    /// 设置字符串值（带过期时间）
    pub async fn set_ex(&self, key: &str, value: &str, seconds: usize) -> RedisResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            connection.set_ex(key, value, seconds as u64).await
        } else {
            Ok(())
        }
    }

    /// 设置 JSON 值（带过期时间）
    pub async fn set_json_ex<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        seconds: usize,
    ) -> RedisResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON serialize error: {}", e)))?;
        self.set_ex(key, &json, seconds).await
    }

    /// 获取 JSON 值
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> RedisResult<Option<T>> {
        match self.get(key).await? {
            Some(json) => {
                let value = serde_json::from_str(&json)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON deserialize error: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 删除键
    pub async fn delete(&self, key: &str) -> RedisResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            let count: i32 = connection.del(key).await?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            connection.exists(key).await
        } else {
            Ok(false)
        }
    }

    /// 自增计数器
    pub async fn incr(&self, key: &str) -> RedisResult<i64> {
        if !self.enabled {
            return Ok(0);
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            connection.incr(key, 1).await
        } else {
            Ok(0)
        }
    }

    /// 发布消息到频道
    pub async fn publish(&self, channel: &str, message: &str) -> RedisResult<usize> {
        if !self.enabled {
            return Ok(0);
        }

        let mut conn = self.connection.write().await;
        if let Some(ref mut connection) = *conn {
            connection.publish(channel, message).await
        } else {
            Ok(0)
        }
    }

    /// 检查缓存是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 缓存键命名空间
pub mod keys {
    pub fn user_stats(user_id: &str) -> String {
        format!("stats:user:{}", user_id)
    }

    pub fn inbound_stats(inbound_id: &str) -> String {
        format!("stats:inbound:{}", inbound_id)
    }

    pub fn traffic_summary() -> String {
        "stats:traffic:summary".to_string()
    }

    pub fn system_status() -> String {
        "system:status".to_string()
    }

    pub fn user_sessions(user_id: &str) -> String {
        format!("sessions:user:{}", user_id)
    }

    pub fn api_rate_limit(ip: &str, endpoint: &str) -> String {
        format!("ratelimit:{}:{}", ip, endpoint)
    }

    pub fn cache_stats() -> String {
        "stats:cache".to_string()
    }
}
