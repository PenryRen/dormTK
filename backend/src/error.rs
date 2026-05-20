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

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized".to_owned(),
            message: message.into(),
            details: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request".to_owned(),
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found".to_owned(),
            message: message.into(),
            details: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict".to_owned(),
            message: message.into(),
            details: None,
        }
    }

    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_IMPLEMENTED,
            code: "not_implemented".to_owned(),
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
        match error {
            sqlx::Error::RowNotFound => Self::not_found("record not found"),
            sqlx::Error::Database(database_error)
                if database_error.code().as_deref() == Some("23505") =>
            {
                Self::conflict(database_error.message().to_owned())
            }
            sqlx::Error::Database(database_error)
                if database_error.code().as_deref() == Some("23503") =>
            {
                Self::bad_request(database_error.message().to_owned())
            }
            other => Self::service_unavailable(format!("database error: {other}")),
        }
    }
}
