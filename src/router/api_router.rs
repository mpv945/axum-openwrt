use std::sync::Arc;
use axum::{Router, routing::get, Extension, middleware, Json};
use axum::extract::State;
use axum::routing::{delete, post};
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use uuid::Uuid;
use crate::handler::user_handler;
use crate::middleware::auth::auth_middleware;
use crate::middleware::layers::apply_middleware;
use crate::middleware::panic_handler::panic_handler;
use crate::model::app_state::{AppState, CurrentUser};


// | API             | 作用                     |
// | --------------- | ---------------------- |
// | `route()`       | 添加单个路由                 |
// | `nest()`        | 路由前缀分组                 |
// | `merge()`       | 合并 Router              |
// | `layer()`       | 添加 middleware          |
// | `route_layer()` | 给某个 route 加 middleware |
fn api_router() -> Router {
    let app = Router::new()
        //.route("/users", get(user_handler::list_users_simple))
        .route("/health",get(|| async { "ok" }));
    apply_middleware(app)
}

use crate::{
    handler::user_handler::{
        bad_request_demo, biz_error_demo, internal_error_demo, profile, panic_error_demo,
        create_user,
        list_users,
        get_user,
        delete_user,
    },
    //state::app_state::AppState as OtherAppState,
};
use crate::model::mqtt_message::MqttMessage;
use crate::r#static::embed::Assets;
use crate::r#static::handler::static_handler;

pub fn router(state: Arc<AppState>) ->  Router {
    let middleware = ServiceBuilder::new()
        .layer(CatchPanicLayer::custom(panic_handler));
    // 添加/api顶级路径
    let app = Router::new()
        //.nest("/api", api_router()) //也要返回Router<Arc<AppState>>
        //.route("/users", get(user_handler::list_users_simple))
        .route("/profile1", get(profile1))

        .route("/profile", get(profile)).layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .route("/biz-error", get(biz_error_demo))
        .route("/bad-request", get(bad_request_demo))
        .route("/internal-error", get(internal_error_demo))
        .route("/panic-error", get(panic_error_demo))


        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", delete(delete_user))

        .route("/mqtt/publish", post(publish))

        // 静态页面：*path 是 Axum 提供的 path 捕获，会捕获 / 后的 所有路径，包括子目录。
        //.route("/*path", get(static_handler))

        //.route_layer(TraceLayer::new_for_http()) // 日志
        //.route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .route_layer(middleware) // 错误处理层放在最后
        // 静态资源兜底
        .fallback(get(static_handler))

        /*.layer(CompressionLayer::new())                    // 自动压缩响应（Brotli 11 级最强）
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(10)))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)) // 防大请求
        .layer(TraceLayer::new_for_http())                // 生产 tracing*/
        .with_state(state.clone());

    apply_middleware(app)

    /*let static_page = Router::new()
        .route("/", get(static_handler))
        //.nest_service("/", ServeEmbed::<Assets>::new())
        // 可选：SPA fallback（所有未知路径返回 index.html）
        //.fallback_service(ServeEmbed::<Assets>::new_with_fallback("index.html"))
        .with_state(state.clone());
    apply_middleware(app).merge(static_page)*/

}
async fn profile1(
    Extension(user): Extension<CurrentUser>,
    State(state): State<Arc<AppState>>
) -> String {


    println!("{}",format!("hello {}", user.username));
    println!("{}",format!("hello {}", user.user_id));

    println!("{}",format!("secret = {}", &state.config.jwt_secret));

    format!("hello {}", user.username)
}

#[derive(serde::Deserialize)]
struct PublishReq {
    client_id: String,
    topic: String,
    payload: String,
}
async fn publish(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PublishReq>,
) -> Json<&'static str> {
    let msg = MqttMessage {
        client_id: req.client_id,
        message_id: Uuid::new_v4().to_string(),
        topic: req.topic,
        payload: req.payload,
        timestamp: chrono::Utc::now().timestamp(),
    };

    state.producer.publish(&msg).await.unwrap();

    Json("ok")
}
// 多级嵌套
// /api/users
// /api/orders
/*pub fn user_router() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/:id", get(get_orders))
}

pub fn api_router1() -> Router {
    Router::new()
        .nest("/users", user_router())
}

pub fn app_router() -> Router {
    Router::new()
        .nest("/api", api_router1())
}


async fn get_users() -> &'static str {
    "users"
}

async fn get_orders() -> &'static str {
    "orders"
}*/