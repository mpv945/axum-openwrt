use axum::response::IntoResponse;

pub fn panic_handler(_: Box<dyn std::any::Any + Send>) -> axum::response::Response {

    let body = serde_json::json!({
        "code":500,
        "message":"internal server error haijun"
    });

    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(body),
    )
        .into_response()
}