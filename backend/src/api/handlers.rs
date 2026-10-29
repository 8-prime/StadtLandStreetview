use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    AppState,
    auth::{Member, canonical_room, cookie},
    error::ApiResult,
    extract::{ApiJson, ApiPath},
};
use crate::session::{SessionStartInfo, SessionState, Viewpoint, Vote};

#[derive(Serialize)]
pub(super) struct IdResponse<T> {
    id: T,
}

pub(super) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJson(info): ApiJson<SessionStartInfo>,
) -> ApiResult<Response> {
    let membership = state
        .sessions
        .start_session_with_settings(info.name, info.duration_seconds, info.settings)
        .await?;
    let (token, identity) = state.credentials.establish(&headers).await;
    identity
        .memberships
        .lock()
        .await
        .insert(membership.session_id.clone(), membership.player_id);
    state.credentials.remember(token, identity).await;
    Ok((
        StatusCode::CREATED,
        [(
            header::SET_COOKIE,
            cookie(token, state.config.secure_cookies),
        )],
        Json(IdResponse {
            id: membership.session_id,
        }),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JoinRequest {
    name: String,
}

pub(super) async fn join(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
    headers: HeaderMap,
    ApiJson(info): ApiJson<JoinRequest>,
) -> ApiResult<Response> {
    let id = canonical_room(&id);
    let (token, identity) = state.credentials.establish(&headers).await;
    let mut memberships = identity.memberships.lock().await;
    let existing = memberships.get(&id).copied();
    let player = state
        .sessions
        .join_session(&id, info.name, existing)
        .await?;
    memberships.insert(id, player);
    drop(memberships);
    state.credentials.remember(token, identity).await;
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            cookie(token, state.config.secure_cookies),
        )],
    )
        .into_response())
}

pub(super) async fn leave(State(state): State<AppState>, member: Member) -> ApiResult<StatusCode> {
    let mut memberships = member.identity.memberships.lock().await;
    // A delayed duplicate leave may arrive after this browser has joined again.
    if memberships.get(&member.session_id) != Some(&member.player_id) {
        return Ok(StatusCode::NO_CONTENT);
    }
    state
        .sessions
        .leave_session(&member.session_id, member.player_id)
        .await?;
    memberships.remove(&member.session_id);
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn snapshot(
    State(state): State<AppState>,
    member: Member,
) -> ApiResult<Json<SessionState>> {
    Ok(Json(
        state
            .sessions
            .get_session(&member.session_id, member.player_id)
            .await?,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReadyRequest {
    step_id: Uuid,
    ready: bool,
}

pub(super) async fn ready(
    State(state): State<AppState>,
    member: Member,
    ApiJson(info): ApiJson<ReadyRequest>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .ready(
            &member.session_id,
            member.player_id,
            info.step_id,
            info.ready,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CategoryRequest {
    text: String,
}

pub(super) async fn add_category(
    State(state): State<AppState>,
    member: Member,
    ApiJson(info): ApiJson<CategoryRequest>,
) -> ApiResult<(StatusCode, Json<IdResponse<Uuid>>)> {
    let id = state
        .sessions
        .add_category(&member.session_id, member.player_id, info.text)
        .await?;
    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

pub(super) async fn update_category(
    State(state): State<AppState>,
    member: Member,
    ApiPath((_, category)): ApiPath<(String, Uuid)>,
    ApiJson(info): ApiJson<CategoryRequest>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .update_category(&member.session_id, member.player_id, category, info.text)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn delete_category(
    State(state): State<AppState>,
    member: Member,
    ApiPath((_, category)): ApiPath<(String, Uuid)>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .delete_category(&member.session_id, member.player_id, category)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FindingRequest {
    category_id: Uuid,
    view: Viewpoint,
}

pub(super) async fn add_finding(
    State(state): State<AppState>,
    member: Member,
    ApiJson(info): ApiJson<FindingRequest>,
) -> ApiResult<(StatusCode, Json<IdResponse<Uuid>>)> {
    let id = state
        .sessions
        .add_finding(
            &member.session_id,
            member.player_id,
            info.category_id,
            info.view,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

pub(super) async fn delete_finding(
    State(state): State<AppState>,
    member: Member,
    ApiPath((_, finding)): ApiPath<(String, Uuid)>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .delete_finding(&member.session_id, member.player_id, finding)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SelectionRequest {
    finding_id: Uuid,
}

pub(super) async fn select(
    State(state): State<AppState>,
    member: Member,
    ApiPath((_, category)): ApiPath<(String, Uuid)>,
    ApiJson(info): ApiJson<SelectionRequest>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .select_finding(
            &member.session_id,
            member.player_id,
            category,
            info.finding_id,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VoteRequest {
    vote: Vote,
}

pub(super) async fn vote(
    State(state): State<AppState>,
    member: Member,
    ApiPath((_, item)): ApiPath<(String, Uuid)>,
    ApiJson(info): ApiJson<VoteRequest>,
) -> ApiResult<StatusCode> {
    state
        .sessions
        .vote(&member.session_id, member.player_id, item, info.vote)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
