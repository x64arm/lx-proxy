use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;
mod db;
mod auth;
mod xray;
mod tasks;
mod xray_api;
mod email;
mod totp;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lx_proxy_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 初始化数据库连接池
    let pool = db::create_pool().await.expect("Failed to create database pool");
    tracing::info!("✅ Database connection established");

    // 运行数据库迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("✅ Database migrations completed");

    // 初始化 Xray 配置
    xray::init_xray_config().expect("Failed to initialize Xray config");

    // 启动定时任务
    tasks::spawn_all_tasks(pool.clone()).await;

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由 - 公开路由
    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/auth/login", post(handlers::login));

    // 构建路由 - 需要认证的路由
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
        .route("/api/stats/traffic-forecast", get(handlers::stats::get_traffic_forecast));

    // 合并路由并添加中间件
    let app = public_routes
        .merge(
            protected_routes
                .layer(axum::middleware::from_fn_with_state(pool.clone(), auth::auth_middleware))
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    // 启动服务器
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 LX-Proxy Backend running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
