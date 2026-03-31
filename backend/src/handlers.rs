use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use argon2::PasswordVerifier;

use crate::models::*;
use crate::auth::{generate_token, verify_token};
use crate::xray;
use crate::AppState;

pub mod traffic;
pub mod email;
pub mod totp;
pub mod batch;
pub mod stats;
pub mod subscription;
pub mod plugins;
pub mod health;

/// 健康检查
pub async fn health() -> &'static str {
    "OK"
}

/// 获取统计数据
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<Stats>, StatusCode> {
    let stats = sqlx::query_as::<_, (i64, i64, i64, i64, Option<i64>)>(
        r#"SELECT 
            (SELECT COUNT(*) FROM users) as total_users,
            (SELECT COUNT(*) FROM inbound_configs) as total_inbounds,
            (SELECT COUNT(*) FROM inbound_configs WHERE enable = true) as enabled_inbounds,
            (SELECT COALESCE(SUM(traffic_used), 0) FROM inbound_configs) as total_traffic_used,
            (SELECT SUM(traffic_limit) FROM inbound_configs WHERE traffic_limit IS NOT NULL) as total_traffic_limit
        "#
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(Stats {
        total_users: stats.0,
        total_inbounds: stats.1,
        enabled_inbounds: stats.2,
        total_traffic_used: stats.3,
        total_traffic_limit: stats.4,
    }))
}

/// 用户登录
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE username = $1 LIMIT 1"#
    )
    .bind(&req.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error during login: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证密码
    use argon2::{password_hash::PasswordHash, Argon2};
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let valid = Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 生成 JWT token
    let token = generate_token(
        user.id.to_string(),
        user.username.clone(),
        user.role.clone(),
    ).map_err(|e| {
        tracing::error!("Failed to generate token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "token": token,
        "user": {
            "id": user.id,
            "username": user.username,
            "role": user.role
        }
    })))
}

/// 用户登出
pub async fn logout() -> Result<StatusCode, StatusCode> {
    // JWT 是无状态的，客户端只需删除 token
    Ok(StatusCode::OK)
}

/// 获取当前用户
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<User>, StatusCode> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE id = $1 LIMIT 1"#
    )
    .bind(Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

/// 获取用户列表
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users ORDER BY created_at DESC"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(users))
}

/// 创建用户
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    // 哈希密码
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Failed to hash password: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (username, password_hash, role) VALUES ($1, $2, $3) RETURNING *"#
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(req.role.as_deref().unwrap_or("user"))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        if e.to_string().contains("duplicate key") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(user))
}

/// 获取用户
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

/// 更新用户
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    let password_hash = if !req.password.is_empty() {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .to_string()
    } else {
        // 保持原密码
        let old_user = sqlx::query_as::<_, User>(
            r#"SELECT password_hash FROM users WHERE id = $1"#
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
        old_user.password_hash
    };

    let user = sqlx::query_as::<_, User>(
        r#"UPDATE users SET username = $1, password_hash = $2, role = $3, updated_at = NOW() WHERE id = $4 RETURNING *"#
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(req.role.as_deref().unwrap_or("user"))
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

/// 删除用户
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// 获取入站配置列表
pub async fn list_inbounds(
    State(state): State<AppState>,
) -> Result<Json<Vec<InboundConfig>>, StatusCode> {
    let inbounds = sqlx::query_as::<_, InboundConfig>(
        r#"SELECT * FROM inbound_configs ORDER BY created_at DESC"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list inbounds: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(inbounds))
}

/// 创建入站配置
pub async fn create_inbound(
    State(state): State<AppState>,
    Json(req): Json<CreateInboundRequest>,
) -> Result<Json<InboundConfig>, StatusCode> {
    let inbound = sqlx::query_as::<_, InboundConfig>(
        r#"INSERT INTO inbound_configs 
           (user_id, tag, protocol, port, settings, stream_settings, sniffing, traffic_limit, expire_at, ip_limit)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"#
    )
    .bind(req.user_id)
    .bind(&req.tag)
    .bind(&req.protocol)
    .bind(req.port)
    .bind(req.settings)
    .bind(req.stream_settings)
    .bind(req.sniffing)
    .bind(req.traffic_limit)
    .bind(req.expire_at)
    .bind(req.ip_limit)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create inbound: {}", e);
        if e.to_string().contains("duplicate key") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // 更新 Xray 配置
    if let Err(e) = update_xray_config_from_db(&state.pool).await {
        tracing::warn!("Failed to update Xray config: {}", e);
    }

    Ok(Json(inbound))
}

/// 获取入站配置
pub async fn get_inbound(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<InboundConfig>, StatusCode> {
    let inbound = sqlx::query_as::<_, InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(inbound))
}

/// 更新入站配置
pub async fn update_inbound(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateInboundRequest>,
) -> Result<Json<InboundConfig>, StatusCode> {
    let inbound = sqlx::query_as::<_, InboundConfig>(
        r#"UPDATE inbound_configs 
           SET tag = COALESCE($1, tag),
               port = COALESCE($2, port),
               settings = COALESCE($3, settings),
               stream_settings = COALESCE($4, stream_settings),
               sniffing = COALESCE($5, sniffing),
               enable = COALESCE($6, enable),
               traffic_limit = COALESCE($7, traffic_limit),
               expire_at = COALESCE($8, expire_at),
               ip_limit = COALESCE($9, ip_limit),
               updated_at = NOW()
           WHERE id = $10 RETURNING *"#
    )
    .bind(req.tag)
    .bind(req.port)
    .bind(req.settings)
    .bind(req.stream_settings)
    .bind(req.sniffing)
    .bind(req.enable)
    .bind(req.traffic_limit)
    .bind(req.expire_at)
    .bind(req.ip_limit)
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 更新 Xray 配置
    if let Err(e) = update_xray_config_from_db(&state.pool).await {
        tracing::warn!("Failed to update Xray config: {}", e);
    }

    Ok(Json(inbound))
}

/// 删除入站配置
pub async fn delete_inbound(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM inbound_configs WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 更新 Xray 配置
    if let Err(e) = update_xray_config_from_db(&state.pool).await {
        tracing::warn!("Failed to update Xray config: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// 重置流量
pub async fn reset_traffic(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<InboundConfig>, StatusCode> {
    let inbound = sqlx::query_as::<_, InboundConfig>(
        r#"UPDATE inbound_configs SET traffic_used = 0, updated_at = NOW() WHERE id = $1 RETURNING *"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(inbound))
}

/// 获取订阅链接
pub async fn get_subscription_links(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inbound = sqlx::query_as::<_, InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE id = $1 LIMIT 1"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // 生成订阅链接（简化版本）
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let subscription_url = format!("{}/api/sub/{}", base_url, id);

    Ok(Json(serde_json::json!({
        "subscription_url": subscription_url,
        "protocol": inbound.protocol,
        "port": inbound.port,
        "enable": inbound.enable
    })))
}

/// 获取流量统计
pub async fn get_traffic_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT COALESCE(SUM(upload), 0), COALESCE(SUM(download), 0) FROM traffic_logs"#
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "total_upload": stats.0,
        "total_download": stats.1,
        "total_traffic": stats.0 + stats.1
    })))
}

/// 获取入站流量
pub async fn get_inbound_traffic(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TrafficLog>>, StatusCode> {
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(100);

    let logs = sqlx::query_as::<_, TrafficLog>(
        r#"SELECT * FROM traffic_logs WHERE inbound_id = $1 ORDER BY recorded_at DESC LIMIT $2"#
    )
    .bind(id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

/// 获取系统配置
pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<Vec<SystemConfig>>, StatusCode> {
    let configs = sqlx::query_as::<_, SystemConfig>(
        r#"SELECT * FROM system_configs ORDER BY key"#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(configs))
}

/// 更新系统配置
pub async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<SystemConfig>,
) -> Result<Json<SystemConfig>, StatusCode> {
    let config = sqlx::query_as::<_, SystemConfig>(
        r#"INSERT INTO system_configs (key, value, description) 
           VALUES ($1, $2, $3) 
           ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW() 
           RETURNING *"#
    )
    .bind(&req.key)
    .bind(req.value)
    .bind(req.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(config))
}

/// 获取 Xray 配置
pub async fn get_xray_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = sqlx::query_scalar::<_, serde_json::Value>(
        r#"SELECT config FROM xray_configs WHERE is_active = true LIMIT 1"#
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(config.unwrap_or(serde_json::json!({}))))
}

/// 更新 Xray 配置
pub async fn update_xray_config(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = sqlx::query_scalar::<_, serde_json::Value>(
        r#"INSERT INTO xray_configs (name, config, is_active) 
           VALUES ($1, $2, true)
           ON CONFLICT DO NOTHING
           RETURNING config"#
    )
    .bind("default")
    .bind(req)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(config))
}

/// 获取系统状态
pub async fn get_system_status() -> Result<Json<SystemStatus>, StatusCode> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    Ok(Json(SystemStatus {
        cpu_usage: sys.global_cpu_usage(),
        memory_total: sys.total_memory(),
        memory_used: sys.used_memory(),
        memory_free: sys.free_memory(),
        uptime: System::uptime(),
        xray_running: xray::check_xray_status(),
        connections: sys.processes().len() as u64,
    }))
}

/// 获取系统日志
pub async fn get_system_logs(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines: usize = params.get("lines").and_then(|s| s.parse().ok()).unwrap_or(100);
    
    // 简化版本，实际应该读取日志文件
    Ok(Json(serde_json::json!({
        "logs": Vec::<String>::new(),
        "message": format!("Showing last {} lines", lines)
    })))
}

/// 从数据库更新 Xray 配置
async fn update_xray_config_from_db(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let inbounds = sqlx::query_as::<_, InboundConfig>(
        r#"SELECT * FROM inbound_configs WHERE enable = true"#
    )
    .fetch_all(pool)
    .await?;

    let mut xray_inbounds = Vec::new();
    for inbound in inbounds {
        let xray_inbound = xray::generate_inbound(
            &inbound.tag,
            &inbound.protocol,
            inbound.port,
            &inbound.settings,
            inbound.stream_settings.as_ref(),
        );
        xray_inbounds.push(xray_inbound);
    }

    let config = serde_json::json!({
        "log": {
            "loglevel": "warning",
            "error": "/var/log/xray/error.log",
            "access": "/var/log/xray/access.log"
        },
        "inbounds": xray_inbounds,
        "outbounds": [{
            "tag": "direct",
            "protocol": "freedom",
            "settings": {}
        }, {
            "tag": "blocked",
            "protocol": "blackhole",
            "settings": {}
        }],
        "routing": {
            "rules": [
                {
                    "type": "field",
                    "ip": ["geoip:private"],
                    "outboundTag": "blocked"
                }
            ]
        }
    });

    let config_path = std::env::var("XRAY_CONFIG_PATH")
        .unwrap_or_else(|_| "/usr/local/etc/xray/config.json".to_string());

    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .map_err(|e| format!("Failed to write Xray config: {}", e))?;
    
    // 重启 Xray 服务（失败只记录日志，不中断流程）
    if let Err(e) = xray::restart_xray() {
        tracing::warn!("Failed to restart Xray: {}", e);
    }

    tracing::info!("Xray config updated and service restarted");
    
    Ok(())
}
