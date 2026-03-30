/// 缓存统计模块
/// 提供缓存命中率和性能指标监控

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use axum::Json;

/// 缓存统计原子计数器
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static CACHE_SETS: AtomicU64 = AtomicU64::new(0);
static CACHE_DELETES: AtomicU64 = AtomicU64::new(0);

/// 记录缓存命中
pub fn record_hit() {
    CACHE_HITS.fetch_add(1, Ordering::Relaxed);
}

/// 记录缓存未命中
pub fn record_miss() {
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
}

/// 记录缓存设置操作
pub fn record_set() {
    CACHE_SETS.fetch_add(1, Ordering::Relaxed);
}

/// 记录缓存删除操作
pub fn record_delete() {
    CACHE_DELETES.fetch_add(1, Ordering::Relaxed);
}

/// 缓存统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 设置操作次数
    pub sets: u64,
    /// 删除操作次数
    pub deletes: u64,
    /// 命中率 (0.0 - 1.0)
    pub hit_rate: f64,
    /// 总操作次数
    pub total_operations: u64,
}

impl CacheStats {
    /// 获取当前缓存统计
    pub fn current() -> Self {
        let hits = CACHE_HITS.load(Ordering::Relaxed);
        let misses = CACHE_MISSES.load(Ordering::Relaxed);
        let sets = CACHE_SETS.load(Ordering::Relaxed);
        let deletes = CACHE_DELETES.load(Ordering::Relaxed);
        
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        Self {
            hits,
            misses,
            sets,
            deletes,
            hit_rate,
            total_operations: hits + misses + sets + deletes,
        }
    }

    /// 重置统计
    pub fn reset() {
        CACHE_HITS.store(0, Ordering::Relaxed);
        CACHE_MISSES.store(0, Ordering::Relaxed);
        CACHE_SETS.store(0, Ordering::Relaxed);
        CACHE_DELETES.store(0, Ordering::Relaxed);
    }
}

/// 获取缓存统计 API
pub async fn get_cache_stats() -> Json<CacheStats> {
    Json(CacheStats::current())
}

/// 重置缓存统计 API
pub async fn reset_cache_stats() -> Json<CacheStats> {
    CacheStats::reset();
    Json(CacheStats::current())
}

/// 缓存健康检查
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHealth {
    pub healthy: bool,
    pub hit_rate: f64,
    pub status: String,
    pub recommendations: Vec<String>,
}

pub async fn check_cache_health() -> Json<CacheHealth> {
    let stats = CacheStats::current();
    let mut recommendations = Vec::new();
    let status;
    let mut healthy = true;

    // 分析命中率
    if stats.hit_rate < 0.3 {
        status = "critical";
        healthy = false;
        recommendations.push("缓存命中率极低 (<30%)，建议检查缓存策略".to_string());
    } else if stats.hit_rate < 0.5 {
        status = "warning";
        recommendations.push("缓存命中率偏低 (<50%)，考虑优化缓存键设计".to_string());
    } else if stats.hit_rate < 0.7 {
        status = "good";
        recommendations.push("缓存命中率一般，可以继续优化".to_string());
    } else {
        status = "excellent";
        recommendations.push("缓存命中率良好".to_string());
    }

    // 分析操作频率
    if stats.sets > stats.hits + stats.misses {
        recommendations.push("缓存写入频繁，考虑增加缓存过期时间".to_string());
    }

    if stats.deletes > stats.hits / 10 {
        recommendations.push("缓存删除频繁，检查是否有不必要的清理".to_string());
    }

    Json(CacheHealth {
        healthy,
        hit_rate: stats.hit_rate,
        status: status.to_string(),
        recommendations,
    })
}

/// 注册缓存统计路由
pub fn register_routes(router: axum::Router<crate::AppState>) -> axum::Router<crate::AppState> {
    use axum::routing::get;
    
    router
        .route("/api/cache/stats", get(get_cache_stats))
        .route("/api/cache/health", get(check_cache_health))
        .route("/api/cache/reset", get(reset_cache_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_initial() {
        let stats = CacheStats::current();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_record_hit_and_miss() {
        // 重置统计
        CacheStats::reset();
        
        // 记录一些操作
        for _ in 0..10 {
            record_hit();
        }
        for _ in 0..5 {
            record_miss();
        }
        
        let stats = CacheStats::current();
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.misses, 5);
        assert!((stats.hit_rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_hit_rate_calculation() {
        CacheStats::reset();
        
        // 100 次命中，0 次未命中
        for _ in 0..100 {
            record_hit();
        }
        
        let stats = CacheStats::current();
        assert_eq!(stats.hit_rate, 1.0);
        
        // 50 次命中，50 次未命中
        CacheStats::reset();
        for _ in 0..50 {
            record_hit();
            record_miss();
        }
        
        let stats = CacheStats::current();
        assert!((stats.hit_rate - 0.5).abs() < 0.01);
    }
}
