// P18 安全加固 - 安全响应头中间件
// 添加关键的安全 HTTP 响应头

use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

/// 安全响应头中间件
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content-Security-Policy (CSP)
    // 限制资源加载来源，防止 XSS
    let csp = "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self' ws: wss:; frame-ancestors 'none'";
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(csp),
    );

    // X-Content-Type-Options
    // 防止 MIME 类型嗅探攻击
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options
    // 防止点击劫持攻击
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );

    // X-XSS-Protection
    // 启用浏览器 XSS 过滤器（旧浏览器兼容）
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    // 控制 Referrer 信息泄露
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy
    // 限制浏览器功能使用
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=(), usb=()"
        ),
    );

    // Cache-Control (API 响应不缓存)
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("no-store, no-cache, must-revalidate, proxy-revalidate"),
    );

    // Pragma (HTTP/1.0 兼容)
    headers.insert(
        "Pragma",
        HeaderValue::from_static("no-cache"),
    );

    // Expires
    headers.insert(
        "Expires",
        HeaderValue::from_static("0"),
    );

    Ok(response)
}

/// 为静态资源添加宽松的缓存策略
pub async fn static_assets_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // 检查是否是静态资源
    let is_static = path.ends_with(".js") 
        || path.ends_with(".css") 
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".woff")
        || path.ends_with(".woff2");

    let mut response = next.run(request).await;

    if is_static {
        let headers = response.headers_mut();
        
        // 静态资源缓存 1 年
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn test_security_headers_present() {
        // 创建一个简单的服务用于测试
        async fn echo(request: Request<Body>) -> Result<Response, StatusCode> {
            Ok(Response::new(Body::empty()))
        }

        let mut service = tower::ServiceBuilder::new()
            .layer_fn(security_headers_middleware)
            .service_fn(echo);

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        let headers = response.headers();

        // 验证所有安全头都存在
        assert!(headers.get("Content-Security-Policy").is_some());
        assert!(headers.get("X-Content-Type-Options").is_some());
        assert!(headers.get("X-Frame-Options").is_some());
        assert!(headers.get("X-XSS-Protection").is_some());
        assert!(headers.get("Referrer-Policy").is_some());
        assert!(headers.get("Permissions-Policy").is_some());
        assert!(headers.get("Cache-Control").is_some());
    }

    #[tokio::test]
    async fn test_csp_header_content() {
        async fn echo(request: Request<Body>) -> Result<Response, StatusCode> {
            Ok(Response::new(Body::empty()))
        }

        let mut service = tower::ServiceBuilder::new()
            .layer_fn(security_headers_middleware)
            .service_fn(echo);

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        let csp = response.headers()
            .get("Content-Security-Policy")
            .unwrap()
            .to_str()
            .unwrap();

        // 验证 CSP 包含关键指令
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }
}
