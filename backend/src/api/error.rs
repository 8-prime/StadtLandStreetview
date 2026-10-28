use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::session::SessionError;

pub(super) struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub(super) fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(super) fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "Join a session to continue.")
    }
}

impl From<SessionError> for ApiError {
    fn from(error: SessionError) -> Self {
        let status = match error {
            SessionError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            SessionError::Forbidden(_) => StatusCode::FORBIDDEN,
            SessionError::NotFound(_) => StatusCode::NOT_FOUND,
            SessionError::Conflict(_) => StatusCode::CONFLICT,
        };
        Self::new(status, error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            message: String,
        }
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

pub(super) type ApiResult<T> = Result<T, ApiError>;
