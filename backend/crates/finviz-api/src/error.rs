//! HTTP error type with a JSON representation.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
}

pub type ApiResult<T> = Result<T, AppError>;

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m),
        };
        (status, Json(ErrorBody { error, message })).into_response()
    }
}
