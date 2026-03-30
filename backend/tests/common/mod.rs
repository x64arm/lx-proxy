// 测试通用工具模块

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde::de::DeserializeOwned;
use tower::ServiceExt;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

// Import from the library crate
use lx_proxy_backend::{AppState, create_app, cache, websocket};

/// 创建测试数据库连接池
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lx_proxy_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

/// 创建测试应用状态
pub async fn create_test_app_state() -> (AppState, PgPool) {
    let pool = create_test_pool().await;
    
    let cache = cache::CacheClient::new("redis://127.0.0.1:6379");
    let ws_manager = websocket::WebSocketManager::new(cache.clone());
    
    let state = AppState {
        pool: pool.clone(),
        ws_manager,
        cache,
    };
    
    (state, pool)
}

/// 创建测试应用
pub fn create_test_app(state: AppState) -> Router {
    create_app(state)
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

/// 清理测试数据
pub async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM traffic_logs")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM inbounds")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username != 'admin'")
        .execute(pool)
        .await
        .ok();
}

/// 发送 POST 请求并获取响应
pub async fn post_request<T, R>(
    app: &mut axum::Router,
    path: &str,
    body: &T,
) -> (StatusCode, R)
where
    T: serde::Serialize,
    R: DeserializeOwned,
{
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    
    let result: R = serde_json::from_slice(&bytes).unwrap();
    (status, result)
}

/// 发送 GET 请求并获取响应
pub async fn get_request<R>(
    app: &mut axum::Router,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, R)
where
    R: DeserializeOwned,
{
    let mut builder = Request::builder()
        .method("GET")
        .uri(path)
        .header("Content-Type", "application/json");

    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {}", token));
    }

    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    
    let result: R = serde_json::from_slice(&bytes).unwrap();
    (status, result)
}

/// 发送 DELETE 请求并获取响应
pub async fn delete_request<R>(
    app: &mut axum::Router,
    path: &str,
    token: &str,
) -> (StatusCode, R)
where
    R: DeserializeOwned,
{
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    
    let result: R = serde_json::from_slice(&bytes).unwrap();
    (status, result)
}

/// 发送 PUT 请求并获取响应
pub async fn put_request<T, R>(
    app: &mut axum::Router,
    path: &str,
    body: &T,
    token: &str,
) -> (StatusCode, R)
where
    T: serde::Serialize,
    R: DeserializeOwned,
{
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    
    let result: R = serde_json::from_slice(&bytes).unwrap();
    (status, result)
}
