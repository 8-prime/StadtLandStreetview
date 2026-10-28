use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Viewpoint {
    pub panorama_id: String,
    pub position: Position,
    pub heading: f64,
    pub pitch: f64,
    pub zoom: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: Uuid,
    pub player_id: Uuid,
    pub category_id: Uuid,
    pub view: Viewpoint,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Lobby,
    Categories,
    Searching,
    Selection,
    Review,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Vote {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub id: Uuid,
    pub name: String,
    pub color: usize,
    pub ready: bool,
    pub connected: bool,
    pub score: u32,
    pub left: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub item_id: Uuid,
    pub category_id: Uuid,
    pub player_id: Uuid,
    pub accepted: bool,
    pub accepts: usize,
    pub rejects: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewItem {
    pub id: Uuid,
    pub finding: Finding,
    pub index: usize,
    pub total: usize,
    pub revealed: bool,
    pub own_vote: Option<Vote>,
    pub votes_cast: usize,
    pub eligible_voters: usize,
    pub result: Option<ReviewResult>,
}

/// A personalized response. Internal session state must never be serialized.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub id: String,
    pub revision: u64,
    pub step_id: Uuid,
    pub phase: Phase,
    pub me_id: Uuid,
    pub demo: bool,
    pub duration_seconds: u64,
    pub server_time: i64,
    pub search_ends_at: Option<i64>,
    pub players: Vec<PlayerState>,
    pub categories: Vec<Category>,
    pub own_findings: Vec<Finding>,
    pub selections: HashMap<Uuid, Uuid>,
    pub review: Option<ReviewItem>,
    pub results: Vec<ReviewResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionStartInfo {
    pub name: String,
    pub duration_seconds: u64,
}

/// The API uses this to establish a credential for the newly created member.
#[derive(Debug, Clone)]
pub struct SessionMembership {
    pub session_id: String,
    pub player_id: Uuid,
}

/// Only invalidation metadata is broadcast; snapshots are fetched per player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SessionEvent {
    StateChanged { revision: u64 },
}
