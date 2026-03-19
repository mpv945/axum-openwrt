use std::sync::Arc;
use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use axum::extract::State;
use crate::model::app_state::AppState;
use crate::r#static::embed::Assets;

pub async fn static_handler(
    State(_state): State<Arc<AppState>>,
    req: Request,
) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/');
    serve_asset(&path)
}

fn serve_asset(path: &str) -> Response<Body> {
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let body = Body::from(content.data);

            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(
                    header::CACHE_CONTROL,
                    if path.contains('.') {
                        "public, max-age=31536000, immutable"
                    } else {
                        "no-cache"
                    },
                )
                .body(body)
                .unwrap()
        }
        None => fallback_index(),
    }
}

fn fallback_index() -> Response<Body> {
    if let Some(index) = Assets::get("index.html") {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(index.data))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap()
    }
}