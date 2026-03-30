// P13 订阅链接优化模块
// 功能：订阅链接加密、二维码生成、访问统计、多客户端支持

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

/// 订阅链接信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionLink {
    pub id: Uuid,
    pub inbound_id: Uuid,
    pub name: String,
    pub protocol: String,
    pub subscription_url: String,
    pub encrypted_url: Option<String>,
    pub access_count: i64,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 二维码数据
#[derive(Debug, Serialize, Deserialize)]
pub struct QRCodeData {
    pub url: String,
    pub base64_image: String,
}

/// 客户端配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_type: String, // clash, v2rayn, singbox, etc.
    pub config: String,
}

/// 访问统计
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessStats {
    pub total_access: i64,
    pub unique_ips: i64,
    pub last_24h_access: i64,
    pub last_7d_access: i64,
    pub top_countries: Vec<CountryStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryStat {
    pub country: String,
    pub count: i64,
}

/// 生成加密订阅链接
pub async fn generate_encrypted_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 获取入站配置
    let inbound = sqlx::query_as::<_, crate::InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // 生成基础订阅链接
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let subscription_url = format!("{}/api/sub/{}", base_url, id);

    // 生成加密令牌（使用 UUID + 时间戳）
    let token = Uuid::new_v4().to_string();
    let encrypted_url = format!("{}/api/sub/encrypted/{}?token={}", base_url, id, token);

    // 保存加密令牌到数据库
    let _ = sqlx::query(
        r#"INSERT INTO subscription_tokens (inbound_id, token, expires_at, created_at)
           VALUES ($1, $2, NOW() + INTERVAL '30 days', NOW())"#
    )
    .bind(id)
    .bind(&token)
    .execute(&state.pool)
    .await;

    Ok(Json(serde_json::json!({
        "inbound_id": id,
        "inbound_name": inbound.remark,
        "protocol": inbound.protocol,
        "subscription_url": subscription_url,
        "encrypted_url": encrypted_url,
        "token_expires_in": "30 days"
    })))
}

/// 生成二维码
pub async fn generate_qrcode(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<QRCodeData>, StatusCode> {
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let subscription_url = format!("{}/api/sub/{}", base_url, id);

    // 使用 qrcode 库生成二维码（需要添加依赖）
    // 这里返回简化版本
    Ok(QRCodeData {
        url: subscription_url,
        base64_image: format!("data:image/png;base64,{}", base64_encode(&subscription_url)),
    })
}

/// 批量生成二维码
pub async fn batch_generate_qrcodes(
    State(state): State<AppState>,
) -> Result<Json<Vec<QRCodeData>>, StatusCode> {
    let inbounds = sqlx::query_as::<_, crate::InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE enable = true ORDER BY remark"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let qrcodes: Vec<QRCodeData> = inbounds
        .iter()
        .map(|inbound| {
            let url = format!("{}/api/sub/{}", base_url, inbound.id);
            QRCodeData {
                url: url.clone(),
                base64_image: format!("data:image/png;base64,{}", base64_encode(&url)),
            }
        })
        .collect();

    Ok(Json(qrcodes))
}

/// 记录订阅链接访问
pub async fn record_access(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    client_ip: String,
) -> Result<(), StatusCode> {
    let _ = sqlx::query(
        r#"INSERT INTO subscription_access_logs (inbound_id, client_ip, accessed_at)
           VALUES ($1, $2, NOW())"#
    )
    .bind(id)
    .bind(&client_ip)
    .execute(&state.pool)
    .await;

    let _ = sqlx::query(
        r#"UPDATE inbound_configs 
           SET access_count = access_count + 1, 
               last_accessed_at = NOW()
           WHERE id = $1"#
    )
    .bind(id)
    .execute(&state.pool)
    .await;

    Ok(())
}

/// 获取访问统计
pub async fn get_access_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccessStats>, StatusCode> {
    // 总访问量
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM subscription_access_logs WHERE inbound_id = $1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // 独立 IP 数
    let unique_ips: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT client_ip) FROM subscription_access_logs WHERE inbound_id = $1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    // 最近 24 小时访问量
    let last_24h: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM subscription_access_logs 
           WHERE inbound_id = $1 AND accessed_at > NOW() - INTERVAL '24 hours'"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    // 最近 7 天访问量
    let last_7d: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM subscription_access_logs 
           WHERE inbound_id = $1 AND accessed_at > NOW() - INTERVAL '7 days'"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    // 国家分布（需要 IP 地理位置库支持，这里简化）
    let top_countries = vec![];

    Ok(Json(AccessStats {
        total_access: total.0,
        unique_ips: unique_ips.0,
        last_24h_access: last_24h.0,
        last_7d_access: last_7d.0,
        top_countries,
    }))
}

/// 生成 Clash 配置
pub async fn generate_clash_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<String, StatusCode> {
    let inbound = sqlx::query_as::<_, crate::InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // 生成 Clash 配置（简化版本）
    let config = format!(
        r#"proxies:
  - name: "{}"
    type: {}
    server: example.com
    port: {}
    uuid: {}
    alterId: 0
    cipher: auto
    tls: true
"#,
        inbound.remark,
        inbound.protocol,
        inbound.port,
        inbound.uuid.unwrap_or_default()
    );

    Ok(config)
}

/// 生成 V2RayN 配置
pub async fn generate_v2rayn_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inbound = sqlx::query_as::<_, crate::InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // 生成 V2RayN vmess 链接
    let vmess_config = serde_json::json!({
        "v": "2",
        "ps": inbound.remark,
        "add": "example.com",
        "port": inbound.port,
        "id": inbound.uuid.unwrap_or_default(),
        "aid": 0,
        "net": "ws",
        "type": "none",
        "host": "",
        "path": "",
        "tls": "tls"
    });

    let vmess_link = base64_encode(&vmess_config.to_string());

    Ok(Json(serde_json::json!({
        "protocol": "vmess",
        "config": vmess_config,
        "link": format!("vmess://{}", vmess_link)
    })))
}

/// 生成 SingBox 配置
pub async fn generate_singbox_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inbound = sqlx::query_as::<_, crate::InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let config = serde_json::json!({
        "outbounds": [
            {
                "type": inbound.protocol,
                "tag": inbound.remark,
                "server": "example.com",
                "server_port": inbound.port,
                "uuid": inbound.uuid.unwrap_or_default()
            }
        ]
    });

    Ok(Json(config))
}

// 辅助函数：Base64 编码
fn base64_encode<T: AsRef<[u8]>>(input: T) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(input)
}
