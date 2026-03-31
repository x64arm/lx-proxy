// P18 安全加固 - 速率限制中间件
// 防止暴力破解和 API 滥用

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// 速率限制器配置
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// 请求时间窗口
    pub window: Duration,
    /// 时间窗口内最大请求数
    pub max_requests: u32,
    /// 封禁时间
    pub ban_duration: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_secs(60),      // 1 分钟
            max_requests: 100,                     // 100 次请求
            ban_duration: Duration::from_secs(300), // 封禁 5 分钟
        }
    }
}

/// 速率限制器状态
#[derive(Debug, Clone)]
struct RateLimiterState {
    /// 请求计数
    requests: Vec<Instant>,
    /// 封禁截止时间
    banned_until: Option<Instant>,
}

/// 共享的速率限制器
pub type SharedRateLimiter = Arc<RwLock<HashMap<String, RateLimiterState>>>;

/// 速率限制器中间件状态
#[derive(Clone)]
pub struct RateLimiterStateWrapper {
    pub limiter: SharedRateLimiter,
    pub config: RateLimiterConfig,
}

impl RateLimiterStateWrapper {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            limiter: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}

/// 速率限制中间件
pub async fn rate_limiter_middleware(
    State(state): State<RateLimiterStateWrapper>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取客户端 IP
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .to_string();

    // 检查是否被封禁
    {
        let limiters = state.limiter.read().await;
        if let Some(limiter) = limiters.get(&client_ip) {
            if let Some(banned_until) = limiter.banned_until {
                if banned_until > Instant::now() {
                    return Err(StatusCode::TOO_MANY_REQUESTS);
                }
            }
        }
    }

    // 检查速率限制
    let mut limiters = state.limiter.write().await;
    let limiter = limiters.entry(client_ip.clone()).or_insert(RateLimiterState {
        requests: Vec::new(),
        banned_until: None,
    });

    let now = Instant::now();
    let window_start = now - state.config.window;

    // 清理过期的请求记录
    limiter.requests.retain(|&t| t > window_start);

    // 检查是否超过限制
    if limiter.requests.len() as u32 >= state.config.max_requests {
        // 触发封禁
        limiter.banned_until = Some(now + state.config.ban_duration);
        tracing::warn!("Rate limit exceeded for IP: {}, banning for {:?}", client_ip, state.config.ban_duration);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 记录请求
    limiter.requests.push(now);

    drop(limiters);

    // 继续处理请求
    Ok(next.run(request).await)
}

/// 登录速率限制（更严格）
pub async fn login_rate_limiter_middleware(
    State(state): State<RateLimiterStateWrapper>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取客户端 IP
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .to_string();

    // 登录限制更严格：5 次/分钟
    let login_config = RateLimiterConfig {
        window: Duration::from_secs(60),
        max_requests: 5,
        ban_duration: Duration::from_secs(900), // 封禁 15 分钟
    };

    let mut limiters = state.limiter.write().await;
    let limiter = limiters.entry(format!("login:{}", client_ip)).or_insert(RateLimiterState {
        requests: Vec::new(),
        banned_until: None,
    });

    let now = Instant::now();
    let window_start = now - login_config.window;

    // 清理过期的请求记录
    limiter.requests.retain(|&t| t > window_start);

    // 检查是否超过限制
    if limiter.requests.len() as u32 >= login_config.max_requests {
        limiter.banned_until = Some(now + login_config.ban_duration);
        tracing::warn!("Login rate limit exceeded for IP: {}, banning for {:?}", client_ip, login_config.ban_duration);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 记录请求
    limiter.requests.push(now);

    drop(limiters);

    // 继续处理请求
    Ok(next.run(request).await)
}
