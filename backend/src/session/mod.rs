use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

mod error;
mod manager;
mod proximity;
mod state;

pub use error::{Result, SessionError};
pub use manager::{SessionConnection, SessionManager};
pub use state::*;

pub const MAX_PLAYERS: usize = 12;
pub const MAX_CATEGORIES: usize = 12;
pub const MAX_FINDINGS_PER_CATEGORY: usize = 12;
pub const MIN_DURATION_SECONDS: u64 = 60;
pub const MAX_DURATION_SECONDS: u64 = 20 * 60;

#[derive(Debug)]
struct Player {
    id: Uuid,
    name: String,
    color: usize,
    findings: HashMap<Uuid, Vec<Finding>>,
    selections: HashMap<Uuid, Uuid>,
    is_ready: bool,
    score: u32,
    left: bool,
    connections: HashSet<Uuid>,
}

/// Synchronous game rules. The manager serializes access to each session.
#[derive(Debug)]
struct Session {
    id: String,
    phase: Phase,
    duration_sec: u64,
    settings: RoomSettings,
    presence: HashMap<Uuid, proximity::Presence>,
    players: HashMap<Uuid, Player>,
    categories: HashMap<Uuid, Category>,
    // Explicit insertion order makes review independent of HashMap iteration.
    player_order: Vec<Uuid>,
    category_order: Vec<Uuid>,
    revision: u64,
    step_id: Uuid,
    search_ends_at: Option<i64>,
    inactive_since: Option<DateTime<Utc>>,
    review_queue: Vec<Finding>,
    review_index: usize,
    votes: HashMap<Uuid, Vote>,
    results: Vec<ReviewResult>,
}

impl Session {
    fn new(duration_sec: u64, now: DateTime<Utc>) -> Result<Self> {
        if !(MIN_DURATION_SECONDS..=MAX_DURATION_SECONDS).contains(&duration_sec) {
            return Err(SessionError::InvalidInput(
                "Search duration must be between 60 and 1200 seconds.",
            ));
        }
        Ok(Self {
            id: new_room_code(),
            phase: Phase::Lobby,
            duration_sec,
            settings: RoomSettings::default(),
            presence: HashMap::new(),
            players: HashMap::new(),
            categories: HashMap::new(),
            player_order: Vec::new(),
            category_order: Vec::new(),
            revision: 0,
            step_id: Uuid::new_v4(),
            search_ends_at: None,
            inactive_since: Some(now),
            review_queue: Vec::new(),
            review_index: 0,
            votes: HashMap::new(),
            results: Vec::new(),
        })
    }

    fn active_player(&self, player_id: Uuid) -> Result<&Player> {
        self.players
            .get(&player_id)
            .filter(|player| !player.left)
            .ok_or(SessionError::Forbidden("Join this session to continue."))
    }

    fn require_phase(&self, phases: &[Phase]) -> Result<()> {
        if !phases.contains(&self.phase) {
            return Err(SessionError::Conflict(
                "This action is not available in the current phase.",
            ));
        }
        Ok(())
    }

    fn new_step(&mut self, phase: Phase) {
        self.presence.clear();
        self.phase = phase;
        self.step_id = Uuid::new_v4();
        for player in self.players.values_mut() {
            player.is_ready = false;
        }
    }

    fn join(&mut self, name: String, existing_player: Option<Uuid>) -> Result<Uuid> {
        let name = validate_text(name, 24, "Use between 1 and 24 characters for your name.")?;
        if let Some(player_id) = existing_player {
            let player = self.players.get(&player_id).ok_or(SessionError::Forbidden(
                "This player does not belong to the session.",
            ))?;
            // Resuming a disconnected membership is allowed in every phase.
            if !player.left {
                return Ok(player_id);
            }
        }
        self.require_phase(&[Phase::Lobby, Phase::Categories])?;
        if self.players.values().filter(|player| !player.left).count() >= MAX_PLAYERS {
            return Err(SessionError::Conflict("This session is full (12 players)."));
        }

        let player_id = existing_player.unwrap_or_else(Uuid::new_v4);
        if let Some(player) = self.players.get_mut(&player_id) {
            player.name = name;
            player.left = false;
        } else {
            self.players.insert(
                player_id,
                Player {
                    id: player_id,
                    name,
                    color: self.player_order.len() % 6,
                    findings: HashMap::new(),
                    selections: HashMap::new(),
                    is_ready: false,
                    score: 0,
                    left: false,
                    connections: HashSet::new(),
                },
            );
            self.player_order.push(player_id);
        }
        self.new_step(self.phase);
        Ok(player_id)
    }

    fn leave(&mut self, player_id: Uuid, now: DateTime<Utc>) -> Result<()> {
        let player = self
            .players
            .get_mut(&player_id)
            .ok_or(SessionError::Forbidden(
                "This player does not belong to the session.",
            ))?;
        if player.left {
            return Ok(());
        }
        player.left = true;
        player.is_ready = false;
        player.connections.clear();
        self.presence
            .retain(|_, presence| presence.player_id != player_id);
        self.votes.remove(&player_id);
        self.update_inactivity(now);
        if matches!(self.phase, Phase::Lobby | Phase::Categories) {
            self.new_step(self.phase);
        }
        self.advance(now);
        Ok(())
    }

    fn connect(&mut self, player_id: Uuid) -> Result<Uuid> {
        self.active_player(player_id)?;
        let connection_id = Uuid::new_v4();
        self.players
            .get_mut(&player_id)
            .unwrap()
            .connections
            .insert(connection_id);
        self.inactive_since = None;
        Ok(connection_id)
    }

    fn disconnect(
        &mut self,
        player_id: Uuid,
        connection_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let player = self
            .players
            .get_mut(&player_id)
            .ok_or(SessionError::Forbidden(
                "This player does not belong to the session.",
            ))?;
        // An old connection cannot disconnect a newer socket for the same player.
        player.connections.remove(&connection_id);
        self.presence.remove(&connection_id);
        self.update_inactivity(now);
        Ok(())
    }

    fn update_inactivity(&mut self, now: DateTime<Utc>) {
        if self
            .players
            .values()
            .all(|player| player.connections.is_empty())
        {
            self.inactive_since.get_or_insert(now);
        } else {
            self.inactive_since = None;
        }
    }

    fn ready(
        &mut self,
        player_id: Uuid,
        step_id: Uuid,
        is_ready: bool,
        now: DateTime<Utc>,
    ) -> Result<()> {
        self.active_player(player_id)?;
        if self.step_id != step_id {
            return Err(SessionError::Conflict(
                "The group has moved on. Please check the current step.",
            ));
        }
        self.require_phase(&[
            Phase::Lobby,
            Phase::Categories,
            Phase::Searching,
            Phase::Selection,
            Phase::Review,
        ])?;
        if is_ready && self.phase == Phase::Categories && self.categories.is_empty() {
            return Err(SessionError::Conflict("Add at least one category first."));
        }
        if self.phase == Phase::Review
            && self.review_queue[self.review_index].player_id != player_id
        {
            return Err(SessionError::Conflict(
                "Cast your vote instead of marking yourself ready.",
            ));
        }
        self.players.get_mut(&player_id).unwrap().is_ready = is_ready;
        self.advance(now);
        Ok(())
    }

    fn add_category(&mut self, player_id: Uuid, name: String) -> Result<Uuid> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Categories])?;
        let text = validate_text(
            name,
            100,
            "Use between 1 and 100 characters for a category.",
        )?;
        if self.categories.len() >= MAX_CATEGORIES {
            return Err(SessionError::Conflict(
                "A session can have up to 12 categories.",
            ));
        }
        let id = Uuid::new_v4();
        self.categories.insert(id, Category { id, text });
        self.category_order.push(id);
        self.new_step(Phase::Categories);
        Ok(id)
    }

    fn update_category(&mut self, player_id: Uuid, category_id: Uuid, name: String) -> Result<()> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Categories])?;
        let text = validate_text(
            name,
            100,
            "Use between 1 and 100 characters for a category.",
        )?;
        let category = self
            .categories
            .get_mut(&category_id)
            .ok_or(SessionError::NotFound("Category not found."))?;
        // Categories are shared and editable by any member during collection.
        if category.text != text {
            category.text = text;
            self.new_step(Phase::Categories);
        }
        Ok(())
    }

    fn delete_category(&mut self, player_id: Uuid, category_id: Uuid) -> Result<()> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Categories])?;
        self.categories
            .remove(&category_id)
            .ok_or(SessionError::NotFound("Category not found."))?;
        self.category_order.retain(|id| *id != category_id);
        self.new_step(Phase::Categories);
        Ok(())
    }

    fn add_finding(
        &mut self,
        player_id: Uuid,
        category_id: Uuid,
        view: Viewpoint,
        now: DateTime<Utc>,
    ) -> Result<Uuid> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Searching])?;
        if !self.categories.contains_key(&category_id) {
            return Err(SessionError::NotFound("Category not found."));
        }
        validate_viewpoint(&view)?;
        let player = self.players.get_mut(&player_id).unwrap();
        let candidates = player.findings.entry(category_id).or_default();
        if candidates.len() >= MAX_FINDINGS_PER_CATEGORY {
            return Err(SessionError::Conflict(
                "You have 12 candidates here. Remove one before adding another.",
            ));
        }
        let id = Uuid::new_v4();
        candidates.push(Finding {
            id,
            player_id,
            category_id,
            view,
            created_at: now.timestamp_millis(),
        });
        player.selections.entry(category_id).or_insert(id);
        player.is_ready = false;
        Ok(id)
    }

    fn delete_finding(&mut self, player_id: Uuid, finding_id: Uuid) -> Result<()> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Searching, Phase::Selection])?;
        let player = self.players.get_mut(&player_id).unwrap();
        let (category_id, candidates) = player
            .findings
            .iter_mut()
            .find(|(_, findings)| findings.iter().any(|finding| finding.id == finding_id))
            .ok_or(SessionError::NotFound("Finding not found."))?;
        candidates.retain(|finding| finding.id != finding_id);
        if player.selections.get(category_id) == Some(&finding_id) {
            if let Some(next) = candidates.first() {
                player.selections.insert(*category_id, next.id);
            } else {
                player.selections.remove(category_id);
            }
        }
        player.is_ready = false;
        Ok(())
    }

    fn select_finding(
        &mut self,
        player_id: Uuid,
        category_id: Uuid,
        finding_id: Uuid,
    ) -> Result<()> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Searching, Phase::Selection])?;
        let player = self.players.get_mut(&player_id).unwrap();
        if !player
            .findings
            .get(&category_id)
            .is_some_and(|findings| findings.iter().any(|finding| finding.id == finding_id))
        {
            return Err(SessionError::NotFound(
                "Own finding not found in this category.",
            ));
        }
        if player.selections.insert(category_id, finding_id) != Some(finding_id) {
            player.is_ready = false;
        }
        Ok(())
    }

    fn vote(
        &mut self,
        player_id: Uuid,
        item_id: Uuid,
        vote: Vote,
        now: DateTime<Utc>,
    ) -> Result<()> {
        self.active_player(player_id)?;
        self.require_phase(&[Phase::Review])?;
        let finding = &self.review_queue[self.review_index];
        if finding.id != item_id {
            return Err(SessionError::Conflict(
                "Voting has closed for that finding.",
            ));
        }
        if finding.player_id == player_id {
            return Err(SessionError::Forbidden(
                "You cannot vote on your own finding.",
            ));
        }
        self.votes.insert(player_id, vote);
        self.players.get_mut(&player_id).unwrap().is_ready = true;
        self.advance(now);
        Ok(())
    }

    fn search_expired(&self, now: DateTime<Utc>) -> bool {
        self.phase == Phase::Searching
            && self
                .search_ends_at
                .is_some_and(|deadline| now.timestamp_millis() >= deadline)
    }

    fn tick(&mut self, now: DateTime<Utc>) -> bool {
        if !self.search_expired(now) {
            return false;
        }
        self.new_step(Phase::Selection);
        true
    }

    fn advance(&mut self, now: DateTime<Utc>) {
        let mut active = self
            .players
            .values()
            .filter(|player| !player.left)
            .peekable();
        if active.peek().is_none() || !active.all(|player| player.is_ready) {
            return;
        }
        match self.phase {
            Phase::Lobby => self.new_step(Phase::Categories),
            Phase::Categories if !self.categories.is_empty() => {
                self.search_ends_at =
                    Some(now.timestamp_millis() + self.duration_sec as i64 * 1000);
                self.new_step(Phase::Searching);
            }
            Phase::Searching => self.new_step(Phase::Selection),
            Phase::Selection => {
                // Freeze the selected findings, including submissions of departed players.
                for category_id in &self.category_order {
                    for player_id in &self.player_order {
                        let player = &self.players[player_id];
                        if let Some(finding_id) = player.selections.get(category_id)
                            && let Some(finding) = player.findings[category_id]
                                .iter()
                                .find(|finding| finding.id == *finding_id)
                        {
                            self.review_queue.push(finding.clone());
                        }
                    }
                }
                self.new_step(if self.review_queue.is_empty() {
                    Phase::Finished
                } else {
                    Phase::Review
                });
            }
            Phase::Review => {
                let finding = &self.review_queue[self.review_index];
                let accepts = self
                    .votes
                    .values()
                    .filter(|vote| **vote == Vote::Accept)
                    .count();
                let rejects = self
                    .votes
                    .values()
                    .filter(|vote| **vote == Vote::Reject)
                    .count();
                let accepted = accepts > rejects;
                self.results.push(ReviewResult {
                    item_id: finding.id,
                    category_id: finding.category_id,
                    player_id: finding.player_id,
                    accepted,
                    accepts,
                    rejects,
                });
                if accepted {
                    self.players.get_mut(&finding.player_id).unwrap().score += 1;
                }
                self.review_index += 1;
                self.votes.clear();
                self.new_step(if self.review_index < self.review_queue.len() {
                    Phase::Review
                } else {
                    Phase::Finished
                });
            }
            _ => {}
        }
    }

    fn snapshot(&self, player_id: Uuid, now: DateTime<Utc>) -> Result<SessionState> {
        let player = self.active_player(player_id)?;
        let review = if self.phase == Phase::Review {
            let finding = &self.review_queue[self.review_index];
            Some(ReviewItem {
                id: finding.id,
                finding: finding.clone(),
                index: self.review_index,
                total: self.review_queue.len(),
                revealed: false,
                own_vote: self.votes.get(&player_id).copied(),
                votes_cast: self.votes.len(),
                eligible_voters: self
                    .players
                    .values()
                    .filter(|player| !player.left && player.id != finding.player_id)
                    .count(),
                result: None,
            })
        } else {
            None
        };
        Ok(SessionState {
            id: self.id.clone(),
            revision: self.revision,
            step_id: self.step_id,
            phase: self.phase,
            me_id: player_id,
            demo: false,
            settings: self.settings,
            duration_seconds: self.duration_sec,
            server_time: now.timestamp_millis(),
            search_ends_at: self.search_ends_at,
            players: self
                .player_order
                .iter()
                .map(|id| {
                    let player = &self.players[id];
                    PlayerState {
                        id: player.id,
                        name: player.name.clone(),
                        color: player.color,
                        ready: player.is_ready,
                        connected: !player.connections.is_empty(),
                        score: player.score,
                        left: player.left,
                    }
                })
                .collect(),
            categories: self
                .category_order
                .iter()
                .map(|id| self.categories[id].clone())
                .collect(),
            own_findings: self
                .category_order
                .iter()
                .filter_map(|id| player.findings.get(id))
                .flatten()
                .cloned()
                .collect(),
            selections: player.selections.clone(),
            review,
            results: self.results.clone(),
        })
    }
}

fn new_room_code() -> String {
    Uuid::new_v4().simple().to_string()[..6].to_ascii_uppercase()
}

fn validate_text(value: String, max: usize, message: &'static str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max {
        return Err(SessionError::InvalidInput(message));
    }
    Ok(value.to_owned())
}

fn validate_viewpoint(view: &Viewpoint) -> Result<()> {
    if view.panorama_id.trim().is_empty()
        || !view.position.lat.is_finite()
        || !(-90.0..=90.0).contains(&view.position.lat)
        || !view.position.lng.is_finite()
        || !(-180.0..=180.0).contains(&view.position.lng)
        || !view.heading.is_finite()
        || !(0.0..360.0).contains(&view.heading)
        || !view.pitch.is_finite()
        || !(-90.0..=90.0).contains(&view.pitch)
        || !view.zoom.is_finite()
        || !(0.0..=5.0).contains(&view.zoom)
    {
        return Err(SessionError::InvalidInput("Invalid Street View viewpoint."));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
