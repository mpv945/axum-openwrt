use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

use deadpool_redis::redis::RedisError;
use deadpool_redis::PoolError;

use crate::common::response::ApiResponse;

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("未授权")]
    Unauthorized,

    #[error("参数错误: {0}")]
    BadRequest(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("业务异常: {0}")]
    BizError(String),

    #[error("系统内部错误")]
    Internal,

    #[error("系统内部错误: {0}")]
    InternalWithMsg(String),

    #[error("redis操作出错: {0}")]
    Redis(String),

    #[error("redis连接池操作出错: {0}")]
    RedisPoolError(String),

}

impl AppError {
    pub fn code(&self) -> i32 {
        match self {
            AppError::Unauthorized => 401,
            AppError::BadRequest(_) => 400,
            AppError::NotFound(_) => 404,
            AppError::BizError(_) => 10001,
            AppError::Internal => 500,
            AppError::InternalWithMsg(_) => 500,
            AppError::Redis(_) => 500,
            AppError::RedisPoolError(_) => 500,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BizError(_) => StatusCode::OK,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalWithMsg(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisPoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("request failed: {}", self);

        let status = self.status_code();
        let body = Json(ApiResponse::<()>::fail(self.code(), self.to_string()));

        (status, body).into_response()
    }

    /*fn into_response_plus(self) -> Response {

        let status = match self {
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,

        };

        let body = Json(ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }*/
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        error!("database error: {}", err);
        AppError::InternalWithMsg(err.to_string())
    }
}

impl From<RedisError> for AppError {
    fn from(err: RedisError) -> Self {
        AppError::Redis(err.to_string())
    }
}

impl From<PoolError> for AppError {
    fn from(err: PoolError) -> Self {
        AppError::RedisPoolError(err.to_string())
    }
}