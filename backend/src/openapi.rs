//! P20 OpenAPI 文档配置
//! 
//! 使用 utoipa-swagger-ui 提供 Swagger UI 界面
//! 
//! TODO: 后续为所有 API 处理器添加 #[utoipa::path] 注解以生成完整的 OpenAPI 文档

use utoipa_swagger_ui::SwaggerUi;

/// 创建 Swagger UI 配置
/// 
/// 当前提供基本的 Swagger UI 界面，访问 /api/docs
/// 后续将逐步为 API 端点添加文档注解
pub fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/api/docs")
        .url("/api-docs/openapi.json", create_basic_openapi())
}

/// 创建基础 OpenAPI 规范
/// 
/// 当前版本只包含基本信息，后续将添加完整的路径和 Schema 定义
fn create_basic_openapi() -> utoipa::openapi::OpenApi {
    use utoipa::openapi::{OpenApi, Info, Paths};
    
    let mut openapi = OpenApi::new(
        Info::new("LX-Proxy API", "0.2.0"),
        Paths::new(),
    );
    
    openapi.info.description = Some("LX-Proxy - 基于 Rust 的 Xray 代理管理面板\n\n## 认证\n大部分 API 端点需要 JWT Bearer Token 认证。\n\n## 错误码\n- 400: 请求参数错误\n- 401: 未授权（Token 无效或过期）\n- 403: 权限不足\n- 404: 资源不存在\n- 500: 服务器内部错误".to_string());
    openapi.info.license = Some(utoipa::openapi::License::new("MIT"));
    
    // TODO: 添加完整的路径定义
    // TODO: 添加 Schema 组件定义
    
    openapi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let openapi = create_basic_openapi();
        assert_eq!(openapi.info.title, "LX-Proxy API");
        assert_eq!(openapi.info.version, "0.2.0");
    }
}
