// LX-Proxy 后端集成测试
// 测试整个 API 系统的端到端功能

mod common;

mod auth_test;
mod user_test;
mod inbound_test;
mod traffic_test;

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use lx_proxy_backend::{
    db::create_pool,
    handlers,
    models::AppState,
};

/// 创建测试用的应用状态
pub async fn create_test_app_state() -> (Arc<AppState>, PgPool) {
    let pool = create_test_pool().await;
    
    let state = Arc::new(AppState {
        pool: pool.clone(),
        jwt_secret: "test-secret-key-for-testing-only-12345".to_string(),
        xray_api_url: std::env::var("XRAY_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:62780".to_string()),
    });
    
    (state, pool)
}

/// 创建测试数据库连接池
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lx_proxy_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

/// 清理测试数据
pub async fn cleanup_test_data(pool: &PgPool) {
    // 按顺序删除（考虑外键约束）
    sqlx::query("DELETE FROM traffic_logs")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM inbound_configs")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .ok();
}

/// 创建测试应用
pub fn create_test_app(state: Arc<AppState>) -> Router {
    handlers::create_router(state)
}
