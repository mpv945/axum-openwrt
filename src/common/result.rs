use axum::Json;
use crate::common::error::AppError;
use crate::common::response::ApiResponse;

pub type AppResult<T> = Result<T, AppError>;

pub type ApiResult<T> = AppResult<Json<ApiResponse<T>>>;