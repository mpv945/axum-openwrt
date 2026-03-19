use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use axum::body::Body;
use axum::response::IntoResponse;
use crate::common::response::ApiResponse;
use crate::model::app_state::{AppState, CurrentUser};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {

    // 从 header 获取 token
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    /*let mut x;
    if token.is_none() {
        //return Err(StatusCode::UNAUTHORIZED);
        x = "没有传令牌"
    }

    let token = token.unwrap();
    x = & token;*/
    let x: &str = if let Some(t) = token.as_deref() {
        t
    } else {
        "没有传令牌"
    };
    if token.is_none() {
        let body = serde_json::to_string(&ApiResponse::<()>::fail(401, "Unauthorized")).unwrap();
        return (StatusCode::UNAUTHORIZED, body).into_response();
    }

    // 假装解析 JWT
    // 读取配置
    let jwt_secret = &state.config.jwt_secret;
    println!(" 状态获取到密钥 = {}, 请求头令牌 = {}", jwt_secret , x);
    // 将用户信息放到共享 state
    // state.set_user(user_info.clone());

    let user = CurrentUser {
        user_id: 1,
        username: "admin".to_string(),
    };

    // 👇 放入 request extensions
    req.extensions_mut().insert(user);

    next.run(req).await
}