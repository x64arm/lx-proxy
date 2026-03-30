// WebSocket 实时推送模块

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use crate::cache::CacheClient;

/// WebSocket 连接管理器
#[derive(Clone)]
pub struct WebSocketManager {
    /// 广播频道发送端
    sender: broadcast::Sender<WsMessage>,
    /// 连接计数
    connection_count: Arc<RwLock<usize>>,
    /// 缓存客户端
    cache: CacheClient,
}

impl WebSocketManager {
    /// 创建新的 WebSocket 管理器
    pub fn new(cache: CacheClient) -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            connection_count: Arc::new(RwLock::new(0)),
            cache,
        }
    }

    /// 处理 WebSocket 连接
    pub async fn handle_connection(
        &self,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        let manager = self.clone();
        ws.on_upgrade(move |socket| async move { manager.handle_socket(socket).await })
    }

    /// 处理单个 WebSocket 连接
    async fn handle_socket(&self, socket: WebSocket) {
        let (mut sender, mut receiver) = socket.split();
        
        // 订阅广播频道
        let mut rx = self.sender.subscribe();
        
        // 增加连接计数
        {
            let mut count = self.connection_count.write().await;
            *count += 1;
            info!("🔌 WebSocket 连接建立，当前连接数：{}", *count);
        }

        // 发送欢迎消息
        let welcome = WsMessage {
            event: "connected".to_string(),
            data: serde_json::json!({
                "message": "欢迎连接到 LX-Proxy WebSocket",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        };
        
        if sender.send(Message::Text(serde_json::to_string(&welcome).unwrap().into())).await.is_err() {
            warn!("发送欢迎消息失败");
        }

        // 接收消息任务
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await.is_err() {
                    break;
                }
            }
        });

        // 发送消息任务（处理客户端消息）
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = receiver.next().await {
                info!("收到 WebSocket 消息：{}", text);
                // 可以在这里处理客户端发送的消息
            }
        });

        // 等待任一任务结束
        tokio::select! {
            _ = (&mut send_task) => {},
            _ = (&mut recv_task) => {},
        }

        // 减少连接计数
        {
            let mut count = self.connection_count.write().await;
            *count = count.saturating_sub(1);
            info!("🔌 WebSocket 连接断开，当前连接数：{}", *count);
        }
    }

    /// 广播消息
    pub async fn broadcast(&self, message: WsMessage) {
        let _ = self.sender.send(message);
    }

    /// 广播流量更新
    pub async fn broadcast_traffic_update(&self, inbound_id: &str, upload: i64, download: i64) {
        let message = WsMessage {
            event: "traffic_update".to_string(),
            data: serde_json::json!({
                "inbound_id": inbound_id,
                "upload": upload,
                "download": download,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        };
        let message_json = serde_json::to_string(&message).unwrap();
        self.broadcast(message).await;
        
        // 同时发布到 Redis
        let _ = self.cache.publish("traffic_updates", &message_json).await;
    }

    /// 广播系统状态更新
    pub async fn broadcast_system_status(&self, cpu: f32, memory: f32, disk: f32) {
        let message = WsMessage {
            event: "system_status".to_string(),
            data: serde_json::json!({
                "cpu": cpu,
                "memory": memory,
                "disk": disk,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        };
        self.broadcast(message).await;
    }

    /// 广播告警消息
    pub async fn broadcast_alert(&self, level: &str, title: &str, message: &str) {
        let alert = WsMessage {
            event: "alert".to_string(),
            data: serde_json::json!({
                "level": level,
                "title": title,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        };
        let alert_json = serde_json::to_string(&alert).unwrap();
        self.broadcast(alert).await;
        
        // 同时发布到 Redis
        let _ = self.cache.publish("alerts", &alert_json).await;
    }

    /// 获取当前连接数
    pub async fn get_connection_count(&self) -> usize {
        *self.connection_count.read().await
    }
}

/// WebSocket 消息结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsMessage {
    pub event: String,
    pub data: serde_json::Value,
}

/// WebSocket 状态响应
#[derive(Debug, Serialize)]
pub struct WebSocketStatus {
    pub enabled: bool,
    pub connections: usize,
    pub cache_enabled: bool,
}

/// 处理 WebSocket 连接（暂时移除，使用 main.rs 中的内联实现）
#[allow(dead_code)]
pub async fn handle_ws_connection_stub() {
    // Stub implementation
}

/// 获取 WebSocket 状态
pub async fn get_ws_status(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
) -> Json<WebSocketStatus> {
    let manager = state.ws_manager;
    let connections = manager.get_connection_count().await;
    Json(WebSocketStatus {
        enabled: true,
        connections,
        cache_enabled: manager.cache.is_enabled(),
    })
}
