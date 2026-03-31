// P19 性能基准测试
// API 性能基准测试脚本

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use reqwest::Client;
use std::time::Duration;

/// 基准测试配置
struct BenchmarkConfig {
    base_url: String,
    api_key: String,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("BENCHMARK_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("BENCHMARK_API_KEY").unwrap_or_else(|_| "test_token".to_string()),
        }
    }
}

/// 测试 API 响应时间
async fn test_api_response_time(client: &Client, config: &BenchmarkConfig, endpoint: &str) -> Duration {
    let start = std::time::Instant::now();
    
    let response = client
        .get(format!("{}{}", config.base_url, endpoint))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            let _ = resp.text().await;
            start.elapsed()
        }
        Err(_) => Duration::from_secs(999), // 失败返回高延迟
    }
}

/// 健康检查基准测试
fn bench_health_check(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("Health Check");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    group.bench_function(BenchmarkId::from_parameter("/api/health"), |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(test_api_response_time(&client, &config, "/api/health").await)
            })
        })
    });

    group.finish();
}

/// 用户列表基准测试
fn bench_user_list(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("User List");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    group.bench_function(BenchmarkId::from_parameter("/api/users"), |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(test_api_response_time(&client, &config, "/api/users").await)
            })
        })
    });

    group.finish();
}

/// 入站列表基准测试
fn bench_inbound_list(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("Inbound List");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    group.bench_function(BenchmarkId::from_parameter("/api/inbounds"), |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(test_api_response_time(&client, &config, "/api/inbounds").await)
            })
        })
    });

    group.finish();
}

/// 并发性能基准测试
fn bench_concurrent_requests(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Concurrent Requests");
    group.measurement_time(Duration::from_secs(60));

    for concurrency in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} concurrent", concurrency)),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    rt.block_on(async {
                        let client = Client::new();
                        let config = config.clone();
                        
                        let tasks: Vec<_> = (0..concurrency)
                            .map(|_| {
                                let client = client.clone();
                                let config = config.clone();
                                tokio::spawn(async move {
                                    test_api_response_time(&client, &config, "/api/health").await
                                })
                            })
                            .collect();

                        let results = futures::future::join_all(tasks).await;
                        black_box(results)
                    })
                })
            },
        );
    }

    group.finish();
}

/// 数据库查询性能基准测试
fn bench_database_queries(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("Database Queries");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    // 测试统计 API（涉及数据库查询）
    group.bench_function(BenchmarkId::from_parameter("/api/stats"), |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(test_api_response_time(&client, &config, "/api/stats").await)
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_health_check,
    bench_user_list,
    bench_inbound_list,
    bench_concurrent_requests,
    bench_database_queries,
);

criterion_main!(benches);
