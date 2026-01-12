use axum::{
    http::StatusCode,
    response::{Json, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("数据库错误: {0}")]
    Database(String),
    
    #[error("验证错误: {0}")]
    Validation(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("内部服务器错误: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Json(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}