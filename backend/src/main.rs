use lx_proxy_backend::{create_app, AppState, db, cache, websocket, tasks, xray, plugins};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // 创建应用状态
    let state = AppState {
        pool: pool.clone(),
        ws_manager: ws_manager.clone(),
        cache: cache.clone(),
        plugin_registry: plugin_registry.clone(),
    };

    // 启动定时任务
    tasks::spawn_all_tasks(pool.clone()).await;

    // 创建应用路由
    let app = create_app(state);

    // 启动服务器
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 LX-Proxy Backend running on http://{}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
