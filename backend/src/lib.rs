use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};

pub mod handlers;
pub mod models;
pub mod db;
pub mod auth;
pub mod xray;
pub mod tasks;
pub mod xray_api;
pub mod email;
pub mod totp;
pub mod cache;
pub mod websocket;
pub mod optimization;
pub mod cache_stats;
pub mod plugins;
pub mod node;
pub mod middleware;
pub mod audit;
pub mod crypto;
pub mod cluster;
pub mod openapi;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ws_manager: websocket::WebSocketManager,
    pub cache: cache::CacheClient,
    pub plugin_registry: plugins::PluginRegistry,
    pub rate_limiter: middleware::RateLimiterStateWrapper,
}

/// 创建应用路由
pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // P17 健康检查端点（公开访问）
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/live", get(handlers::health::liveness_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/metrics", get(handlers::health::get_metrics));

    let public_routes = Router::new()
        .merge(health_routes)
        .route("/api/auth/login", post(handlers::login));

    let protected_routes = Router::new()
        .route("/api/stats", get(handlers::get_stats))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/me", get(handlers::get_current_user))
        
        // 用户管理
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/users/{id}", get(handlers::get_user).put(handlers::update_user).delete(handlers::delete_user))
        
        // 入站配置
        .route("/api/inbounds", get(handlers::list_inbounds).post(handlers::create_inbound))
        .route("/api/inbounds/{id}", get(handlers::get_inbound).put(handlers::update_inbound).delete(handlers::delete_inbound))
        .route("/api/inbounds/{id}/reset", post(handlers::reset_traffic))
        .route("/api/inbounds/{id}/links", get(handlers::get_subscription_links))
        
        // 流量统计
        .route("/api/traffic", get(handlers::traffic::get_all_traffic))
        .route("/api/traffic/{inbound_id}", get(handlers::traffic::get_inbound_traffic_stats))
        .route("/api/traffic/summary", get(handlers::traffic::get_traffic_summary))
        .route("/api/traffic/log", post(handlers::traffic::record_traffic_log))
        
        // 系统配置
        .route("/api/config", get(handlers::get_config).put(handlers::update_config))
        .route("/api/config/xray", get(handlers::get_xray_config).put(handlers::update_xray_config))
        
        // 系统监控
        .route("/api/system/status", get(handlers::get_system_status))
        .route("/api/system/logs", get(handlers::get_system_logs))
        
        // 邮件通知
        .route("/api/email/test", post(handlers::email::send_test_email))
        .route("/api/email/status", get(handlers::email::get_email_config_status))
        
        // TOTP 双因素认证
        .route("/api/totp/setup", post(handlers::totp::init_totp_setup))
        .route("/api/totp/{user_id}/verify", post(handlers::totp::verify_and_enable_totp))
        .route("/api/totp/{user_id}/disable", post(handlers::totp::disable_totp))
        .route("/api/totp/{user_id}/status", get(handlers::totp::get_totp_status))
        .route("/api/totp/backup-login", post(handlers::totp::login_with_backup_code))
        
        // 批量操作
        .route("/api/batch/inbounds/enable", post(handlers::batch::batch_enable_inbounds))
        .route("/api/batch/inbounds/disable", post(handlers::batch::batch_disable_inbounds))
        .route("/api/batch/inbounds/delete", post(handlers::batch::batch_delete_inbounds))
        .route("/api/batch/inbounds/reset-traffic", post(handlers::batch::batch_reset_traffic))
        .route("/api/batch/inbounds/export", post(handlers::batch::batch_export_configs))
        .route("/api/batch/inbounds/import", post(handlers::batch::batch_import_configs))
        .route("/api/batch/users/delete", post(handlers::batch::batch_delete_users))
        
        // 高级统计
        .route("/api/stats/advanced", get(handlers::stats::get_advanced_stats))
        .route("/api/stats/top-users", get(handlers::stats::get_top_users))
        .route("/api/stats/protocol-distribution", get(handlers::stats::get_protocol_distribution))
        .route("/api/stats/hourly-activity", get(handlers::stats::get_hourly_activity))
        .route("/api/stats/traffic-forecast", get(handlers::stats::get_traffic_forecast))
        
        // 缓存统计与监控
        .route("/api/cache/stats", get(cache_stats::get_cache_stats))
        .route("/api/cache/health", get(cache_stats::check_cache_health))
        .route("/api/cache/reset", get(cache_stats::reset_cache_stats))
        
        // P13 订阅链接优化
        .route("/api/sub/{id}/encrypt", post(handlers::subscription::generate_encrypted_link))
        .route("/api/sub/{id}/qrcode", get(handlers::subscription::generate_qrcode))
        .route("/api/sub/qrcode/batch", get(handlers::subscription::batch_generate_qrcodes))
        .route("/api/sub/{id}/stats", get(handlers::subscription::get_access_stats))
        .route("/api/sub/{id}/clash", get(handlers::subscription::generate_clash_config))
        .route("/api/sub/{id}/v2rayn", get(handlers::subscription::generate_v2rayn_config))
        .route("/api/sub/{id}/singbox", get(handlers::subscription::generate_singbox_config))
        
        // P14 插件系统
        .route("/api/plugins", get(handlers::plugins::list_plugins))
        .route("/api/plugins/{plugin_id}", get(handlers::plugins::get_plugin))
        .route("/api/plugins/{plugin_id}/toggle", post(handlers::plugins::toggle_plugin))
        .route("/api/plugins/{plugin_id}/config", put(handlers::plugins::update_config))
        .route("/api/plugins/{plugin_id}/test", post(handlers::plugins::test_plugin))
        
        // P15 多节点管理
        .route("/api/nodes", get(node::list_nodes).post(node::create_node))
        .route("/api/nodes/stats", get(node::get_all_nodes_stats))
        .route("/api/nodes/{node_id}", get(node::get_node).put(node::update_node).delete(node::delete_node))
        .route("/api/nodes/{node_id}/stats", get(node::get_node_stats))
        .route("/api/nodes/{node_id}/health", post(node::check_health))
        .route("/api/nodes/{node_id}/sync", post(node::sync_node))
        .route("/api/nodes/batch/sync", post(node::batch_sync))
        
        // P16 审计日志系统
        .route("/api/audit/logs", get(audit::query_audit_logs))
        .route("/api/audit/logs/{log_id}", get(audit::get_audit_log))
        .route("/api/audit/stats", get(audit::get_audit_stats))
        .route("/api/audit/login-logs", get(audit::query_login_logs))
        .route("/api/audit/config-history", get(audit::query_config_history))
        .route("/api/audit/config-history/{change_id}", get(audit::get_config_change))
        .route("/api/audit/cleanup", post(audit::cleanup_audit_logs))
        .route("/api/audit/ip-bans", get(audit::get_ip_bans))
        .route("/api/audit/ip-bans/{ip_address}", delete(audit::unban_ip));

    // 登录路由使用更严格的速率限制
    let login_routes = Router::new()
        .route("/api/auth/login", post(handlers::login))
        .layer(axum::middleware::from_fn_with_state(
            state.rate_limiter.clone(),
            middleware::login_rate_limiter_middleware,
        ));

    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .merge(login_routes);

    public_routes
        .merge(protected_routes)
        .layer(axum::middleware::from_fn_with_state(
            state.rate_limiter.clone(),
            middleware::rate_limiter_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth::auth_middleware))
        .layer(cors)
        // P18 安全加固 - 安全响应头（最外层，确保所有响应都包含）
        .layer(axum::middleware::from_fn(middleware::security_headers_middleware))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
