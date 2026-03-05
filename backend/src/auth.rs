use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};
use sqlx::PgPool;

/// JWT 认证中间件
pub async fn auth_middleware(
    State(_pool): State<PgPool>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取 Authorization header
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    // 跳过认证的路径
    let path = req.uri().path();
    if path == "/health" || path.starts_with("/api/auth/login") {
        return Ok(next.run(req).await);
    }

    // 检查 token
    match auth_header {
        Some(header) => {
            if header.starts_with("Bearer ") {
                let token = &header[7..];
                // 验证 JWT token
                match verify_token(token) {
                    Ok(claims) => {
                        tracing::debug!("Authenticated user: {}", claims.username);
                        // 将用户信息注入到 request extensions
                        req.extensions_mut().insert(claims);
                    }
                    Err(_) => {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(req).await)
}

/// 生成 JWT token
pub fn generate_token(user_id: String, username: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};

    let claims = crate::models::Claims {
        sub: user_id,
        username,
        role,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "lx-proxy-secret-key".to_string());
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

/// 验证 JWT token
pub fn verify_token(token: &str) -> Result<crate::models::Claims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "lx-proxy-secret-key".to_string());
    
    decode::<crate::models::Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
