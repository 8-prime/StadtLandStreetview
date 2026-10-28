use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use super::error::{ApiError, ApiResult};

// Keep extraction failures in the same JSON error format as game-rule errors.
pub(super) struct ApiJson<T>(pub(super) T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> ApiResult<Self> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|error| ApiError::new(error.status(), error.body_text()))
    }
}

pub(super) struct ApiPath<T>(pub(super) T);

impl<S, T> FromRequestParts<S> for ApiPath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> ApiResult<Self> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|error| ApiError::new(error.status(), error.body_text()))
    }
}
