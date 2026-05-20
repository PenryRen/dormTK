use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dormtk_api_types::ErrorResponse;
use serde_json::Value;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: String,
    message: String,
    details: Option<Value>,
}

impl ApiError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error".to_owned(),
            message: message.into(),
            details: None,
        }
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "service_unavailable".to_owned(),
            message: message.into(),
            details: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = ErrorResponse {
            code: self.code,
            message: self.message,
            details: self.details,
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::service_unavailable(format!("database error: {error}"))
    }
}
