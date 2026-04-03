use lx_proxy_backend::{create_app, AppState, db, cache, websocket, tasks, xray, plugins, middleware, crypto, cluster, openapi};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    // 初始化 Redis 缓存
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let cache = cache::CacheClient::new(&redis_url);
    cache.initialize().await.ok();
    tracing::info!("✅ Cache client initialized");

    // 初始化 WebSocket 管理器
    let ws_manager = websocket::WebSocketManager::new(cache.clone());
    tracing::info!("✅ WebSocket manager initialized");

    // 初始化插件系统
    let plugin_registry = plugins::create_registry();
    let plugin_loader = plugins::PluginLoader::new(plugin_registry.clone());
    plugin_loader.load_builtin_plugins(&pool).await.expect("Failed to load plugins");
    tracing::info!("✅ Plugin system initialized");

    // 初始化速率限制器（P18 安全加固）
    let rate_limiter = middleware::RateLimiterStateWrapper::new(middleware::RateLimiterConfig::default());
    tracing::info!("✅ Rate limiter initialized (100 req/min, 5 login attempts/min)");

    // 初始化数据加密器（P18 安全加固）
    match crypto::DataEncryptor::from_env() {
        Ok(_encryptor) => {
            tracing::info!("✅ Data encryptor initialized (AES-256-GCM)");
            tracing::info!("🔐 Sensitive data encryption enabled");
        }
        Err(e) => {
            tracing::warn!("⚠️  Data encryptor not initialized: {}", e);
            tracing::warn!("⚠️  Set ENCRYPTION_KEY environment variable to enable encryption");
        }
    }

    // 初始化 P19 高可用集群模块 (MVP 简化版)
    let cluster_state = Arc::new(RwLock::new(lx_proxy_backend::cluster::ClusterApiState::default()));
    
    if std::env::var("ENABLE_CLUSTER").unwrap_or_else(|_| "false".to_string()) == "true" {
        let node_id = std::env::var("NODE_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::new_v4);
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| format!("node-{}", node_id.to_string()[..8].to_string()));
        let node_address = std::env::var("NODE_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        
        match cluster::init_cluster(node_id, node_name.clone(), node_address).await {
            Ok(ctx) => {
                let mut state = cluster_state.write().await;
                state.context = Some(ctx);
            }
            Err(e) => {
                tracing::warn!("⚠️  Cluster module initialization failed: {}", e);
            }
        }
    }

    // 创建应用状态
    let state = AppState {
        pool: pool.clone(),
        ws_manager: ws_manager.clone(),
        cache: cache.clone(),
        plugin_registry: plugin_registry.clone(),
        rate_limiter: rate_limiter.clone(),
    };

    // 启动定时任务
    tasks::spawn_all_tasks(pool.clone()).await;

    // 创建应用路由
    let app = create_app(state);
    
    // 添加 Swagger UI（P20 API 文档）
    let app = app.merge(openapi::create_swagger_ui());

    // 启动服务器
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 LX-Proxy Backend running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
