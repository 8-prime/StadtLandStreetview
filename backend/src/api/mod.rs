use std::sync::Arc;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use tokio::sync::watch;

use crate::session::SessionManager;
use auth::Credentials;
use error::ApiError;

mod auth;
mod error;
mod extract;
mod frontend;
mod handlers;
mod websocket;

#[derive(Clone, Default)]
pub struct ApiConfig {
    /// Enable when serving HTTPS, including behind a reverse proxy.
    pub secure_cookies: bool,
    /// Set if a reverse proxy rewrites Host. Otherwise Origin must match Host.
    pub public_origin: Option<Uri>,
}

#[derive(Clone)]
pub struct AppState {
    pub sessions: SessionManager,
    credentials: Credentials,
    config: Arc<ApiConfig>,
    socket_settings: websocket::SocketSettings,
    shutdown: watch::Sender<bool>,
}

impl AppState {
    pub fn new(sessions: SessionManager, config: ApiConfig) -> Self {
        let (shutdown, _) = watch::channel(false);
        Self {
            sessions,
            credentials: Credentials::default(),
            config: Arc::new(config),
            socket_settings: websocket::SocketSettings::default(),
            shutdown,
        }
    }

    /// Close upgraded sockets when the HTTP server starts shutting down.
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/sessions", post(handlers::create))
        .route("/sessions/{session_id}", get(handlers::snapshot))
        .route("/sessions/{session_id}/join", post(handlers::join))
        .route("/sessions/{session_id}/leave", post(handlers::leave))
        .route("/sessions/{session_id}/ready", put(handlers::ready))
        .route(
            "/sessions/{session_id}/categories",
            post(handlers::add_category),
        )
        .route(
            "/sessions/{session_id}/categories/{category_id}",
            axum::routing::patch(handlers::update_category).delete(handlers::delete_category),
        )
        .route(
            "/sessions/{session_id}/findings",
            post(handlers::add_finding),
        )
        .route(
            "/sessions/{session_id}/findings/{finding_id}",
            axum::routing::delete(handlers::delete_finding),
        )
        .route(
            "/sessions/{session_id}/selections/{category_id}",
            put(handlers::select),
        )
        .route(
            "/sessions/{session_id}/review/{item_id}/vote",
            put(handlers::vote),
        )
        .route("/sessions/{session_id}/ws", get(websocket::upgrade));

    Router::new()
        .nest("/api", api)
        .fallback(frontend::serve)
        .method_not_allowed_fallback(|| async {
            ApiError::new(
                StatusCode::METHOD_NOT_ALLOWED,
                "Method not allowed for this endpoint.",
            )
        })
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_policy,
        ))
        .with_state(state)
}

async fn request_policy(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // Public frontend files keep their own caching policy and do not need API origin checks.
    let path = request.uri().path();
    if path != "/api" && !path.starts_with("/api/") {
        return next.run(request).await;
    }
    let mut response = if origin_allowed(request.headers(), &state.config) {
        next.run(request).await
    } else {
        ApiError::new(StatusCode::FORBIDDEN, "Request origin is not allowed.").into_response()
    };
    // Snapshots and credential-establishing responses must never enter shared caches.
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn origin_allowed(headers: &HeaderMap, config: &ApiConfig) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        // Non-browser clients may omit Origin; they still need the opaque cookie.
        return true;
    };
    let Ok(origin) = origin.to_str().unwrap_or_default().parse::<Uri>() else {
        return false;
    };
    if !matches!(origin.scheme_str(), Some("http" | "https"))
        || origin.authority().is_none()
        || origin.query().is_some()
        || origin.path() != "/" && !origin.path().is_empty()
    {
        return false;
    }
    if let Some(expected) = &config.public_origin {
        return origin.scheme() == expected.scheme() && origin.authority() == expected.authority();
    }
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    host.is_some_and(|host| {
        origin
            .authority()
            .unwrap()
            .as_str()
            .eq_ignore_ascii_case(host)
    }) && (!config.secure_cookies || origin.scheme_str() == Some("https"))
}

#[cfg(test)]
mod tests;
