use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::types::ProblemDetails;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail, cause) = match self {
            AppError::BadRequest(ref msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                msg.clone(),
                None,
            ),
            AppError::NotFound(ref msg) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                msg.clone(),
                None,
            ),
            AppError::InternalServerError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                msg.clone(),
                Some("SYSTEM_FAILURE"),
            ),
            AppError::ServiceUnavailable(ref msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                msg.clone(),
                Some("SYSTEM_FAILURE"),
            ),
            AppError::Forbidden(ref msg) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                msg.clone(),
                None,
            ),
            AppError::ConfigError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                msg.clone(),
                Some("SYSTEM_FAILURE"),
            ),
        };

        let mut problem_details = ProblemDetails::new(status.as_u16(), title, &detail);
        if let Some(cause) = cause {
            problem_details = problem_details.with_cause(cause);
        }
        let body = serde_json::to_string(&problem_details).unwrap_or_default();

        Response::builder()
            .status(status)
            .header("Content-Type", "application/problem+json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}
