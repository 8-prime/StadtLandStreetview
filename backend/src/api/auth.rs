use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{FromRequestParts, Path},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use super::{
    AppState,
    error::{ApiError, ApiResult},
};

const COOKIE_NAME: &str = "sls_identity";

#[derive(Default)]
pub(super) struct Identity {
    // Serialize membership creation/removal for each browser to avoid duplicate joins.
    pub(super) memberships: Mutex<HashMap<String, Uuid>>,
}

#[derive(Clone, Default)]
pub(super) struct Credentials {
    identities: Arc<RwLock<HashMap<Uuid, Arc<Identity>>>>,
}

impl Credentials {
    pub(super) async fn lookup(&self, headers: &HeaderMap) -> Option<(Uuid, Arc<Identity>)> {
        let token = headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|header| header.to_str().ok())
            .flat_map(|value| value.split(';'))
            .filter_map(|cookie| cookie.trim().split_once('='))
            .find_map(|(name, value)| {
                (name == COOKIE_NAME)
                    .then(|| Uuid::parse_str(value).ok())
                    .flatten()
            })?;
        self.identities
            .read()
            .await
            .get(&token)
            .cloned()
            .map(|identity| (token, identity))
    }

    pub(super) async fn establish(&self, headers: &HeaderMap) -> (Uuid, Arc<Identity>) {
        if let Some(existing) = self.lookup(headers).await {
            return existing;
        }
        let token = Uuid::new_v4();
        let identity = Arc::new(Identity::default());
        (token, identity)
    }

    pub(super) async fn remember(&self, token: Uuid, identity: Arc<Identity>) {
        self.identities.write().await.insert(token, identity);
    }
}

pub(super) fn cookie(token: Uuid, secure: bool) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}={token}; Path=/api; HttpOnly; SameSite=Strict{}",
        if secure { "; Secure" } else { "" },
    ))
    .expect("UUIDs and fixed cookie attributes form a valid header")
}

pub(super) fn canonical_room(id: &str) -> String {
    id.trim().to_ascii_uppercase()
}

pub(super) struct Member {
    pub(super) session_id: String,
    pub(super) player_id: Uuid,
    pub(super) identity: Arc<Identity>,
}

impl FromRequestParts<AppState> for Member {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ApiResult<Self> {
        let Path(parameters) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|error| ApiError::new(error.status(), error.body_text()))?;
        let session_id = canonical_room(
            parameters
                .get("session_id")
                .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "Missing session ID."))?,
        );
        let (_, identity) = state
            .credentials
            .lookup(&parts.headers)
            .await
            .ok_or_else(ApiError::unauthorized)?;
        let player_id = identity
            .memberships
            .lock()
            .await
            .get(&session_id)
            .copied()
            .ok_or_else(|| {
                ApiError::new(StatusCode::FORBIDDEN, "Join this session to continue.")
            })?;
        Ok(Self {
            session_id,
            player_id,
            identity,
        })
    }
}
