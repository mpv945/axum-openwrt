use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;

use axum::http::StatusCode;
use axum::Router;
use std::time::Duration;

pub fn apply_middleware(router: Router) -> Router {

    /*let origins = [
        HeaderValue::from_static("https://example.com"),
        HeaderValue::from_static("http://example.com"),
    ];


    let cors = CorsLayer::new()
        // 只允许 https://example.com 和 https://app.example.com
        //.allow_origin("https://example.com".parse().unwrap())
        //HeaderValue
        //Any（允许所有）
        //regex::Regex（匹配正则域名）
        //.allow_origin(HeaderValue::from_static("https://example.com"));

        .allow_origin(origins)

        // 允许的 HTTP 方法
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])

        // 允许客户端自定义请求头
        .allow_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-request-id"),
        ])
        //.allow_headers(Any) // 允许所有

        // 客户端可以读取的响应
        // 允许客户端读取的响应头
        .expose_headers(vec![
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-trace-id"),
        ])

        // 允许携带 cookie
        .allow_credentials(true)

        // 浏览器缓存预检请求 1 小时
        .max_age(Duration::from_secs(3600));*/

    let middleware_stack = ServiceBuilder::new()
        // 请求日志
        //.layer(TraceLayer::new_for_http())
        // 解决跨域问题（CORS）
        .layer(CorsLayer::permissive()) // 允许所有域名、方法、头
        // 对响应进行压缩（gzip / brotli）
        //.layer(CompressionLayer::new().gzip(true).br(true))
        // 为请求设置超时时间
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ));

        router.layer(middleware_stack)
}