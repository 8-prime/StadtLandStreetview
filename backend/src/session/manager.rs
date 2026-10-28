use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use super::{
    Phase, Result, Session, SessionError, SessionEvent, SessionMembership, SessionState, Viewpoint,
    Vote, new_room_code,
};

struct SessionHandle {
    state: RwLock<Session>,
    events: broadcast::Sender<SessionEvent>,
}

/// One authenticated WebSocket connection, independent of player membership.
/// Call `disconnect` with this ID when the socket closes or its heartbeat expires.
pub struct SessionConnection {
    pub connection_id: Uuid,
    pub updates: broadcast::Receiver<SessionEvent>,
}

/// Clone this into Axum state. All player IDs must come from verified credentials,
/// never directly from an untrusted request body. No network I/O runs under a lock.
#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<SessionHandle>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// `name` is the creator's display name; the room code is server-generated.
    pub async fn start_session(
        &self,
        name: String,
        duration_sec: u64,
    ) -> Result<SessionMembership> {
        let mut session = Session::new(duration_sec, Utc::now())?;
        let player_id = session.join(name, None)?;
        let mut sessions = self.sessions.write().await;
        while sessions.contains_key(&session.id) {
            session.id = new_room_code();
        }
        let session_id = session.id.clone();
        session.revision = 1;
        let (events, _) = broadcast::channel(32);
        sessions.insert(
            session_id.clone(),
            Arc::new(SessionHandle {
                state: RwLock::new(session),
                events,
            }),
        );
        Ok(SessionMembership {
            session_id,
            player_id,
        })
    }

    /// Pass an existing player only when resuming a verified membership.
    pub async fn join_session(
        &self,
        session_id: &str,
        name: String,
        existing_player: Option<Uuid>,
    ) -> Result<Uuid> {
        self.update(session_id, |session, _| session.join(name, existing_player))
            .await
    }

    pub async fn leave_session(&self, session_id: &str, player_id: Uuid) -> Result<()> {
        self.update(session_id, |session, now| session.leave(player_id, now))
            .await
    }

    pub async fn get_session(&self, session_id: &str, player_id: Uuid) -> Result<SessionState> {
        let handle = self.find(session_id).await?;
        {
            let session = handle.state.read().await;
            let now = Utc::now();
            if !session.search_expired(now) {
                return session.snapshot(player_id, now);
            }
        }
        let mut session = handle.state.write().await;
        let now = Utc::now();
        session.active_player(player_id)?;
        if session.tick(now) {
            handle.publish(&mut session);
        }
        session.snapshot(player_id, now)
    }

    /// Subscribe before publishing the connection change so no update is missed.
    pub async fn connect(&self, session_id: &str, player_id: Uuid) -> Result<SessionConnection> {
        let handle = self.find(session_id).await?;
        let updates = handle.events.subscribe();
        let connection_id = handle
            .update(|session, _| session.connect(player_id))
            .await?;
        Ok(SessionConnection {
            connection_id,
            updates,
        })
    }

    /// Disconnecting never removes membership or changes readiness.
    pub async fn disconnect(
        &self,
        session_id: &str,
        player_id: Uuid,
        connection_id: Uuid,
    ) -> Result<()> {
        self.update(session_id, |session, now| {
            session.disconnect(player_id, connection_id, now)
        })
        .await
    }

    pub async fn ready(
        &self,
        session_id: &str,
        player_id: Uuid,
        step_id: Uuid,
        is_ready: bool,
    ) -> Result<()> {
        self.update(session_id, |session, now| {
            session.ready(player_id, step_id, is_ready, now)
        })
        .await
    }

    pub async fn add_category(
        &self,
        session_id: &str,
        player_id: Uuid,
        text: String,
    ) -> Result<Uuid> {
        self.update(session_id, |session, _| {
            session.add_category(player_id, text)
        })
        .await
    }

    pub async fn update_category(
        &self,
        session_id: &str,
        player_id: Uuid,
        category_id: Uuid,
        text: String,
    ) -> Result<()> {
        self.update(session_id, |session, _| {
            session.update_category(player_id, category_id, text)
        })
        .await
    }

    pub async fn delete_category(
        &self,
        session_id: &str,
        player_id: Uuid,
        category_id: Uuid,
    ) -> Result<()> {
        self.update(session_id, |session, _| {
            session.delete_category(player_id, category_id)
        })
        .await
    }

    pub async fn add_finding(
        &self,
        session_id: &str,
        player_id: Uuid,
        category_id: Uuid,
        view: Viewpoint,
    ) -> Result<Uuid> {
        self.update(session_id, |session, now| {
            session.add_finding(player_id, category_id, view, now)
        })
        .await
    }

    pub async fn delete_finding(
        &self,
        session_id: &str,
        player_id: Uuid,
        finding_id: Uuid,
    ) -> Result<()> {
        self.update(session_id, |session, _| {
            session.delete_finding(player_id, finding_id)
        })
        .await
    }

    pub async fn select_finding(
        &self,
        session_id: &str,
        player_id: Uuid,
        category_id: Uuid,
        finding_id: Uuid,
    ) -> Result<()> {
        self.update(session_id, |session, _| {
            session.select_finding(player_id, category_id, finding_id)
        })
        .await
    }

    pub async fn vote(
        &self,
        session_id: &str,
        player_id: Uuid,
        item_id: Uuid,
        vote: Vote,
    ) -> Result<()> {
        self.update(session_id, |session, _| {
            session.vote(player_id, item_id, vote)
        })
        .await
    }

    async fn find(&self, session_id: &str) -> Result<Arc<SessionHandle>> {
        self.sessions
            .read()
            .await
            .get(&session_id.trim().to_ascii_uppercase())
            .cloned()
            .ok_or(SessionError::NotFound("Session not found."))
    }

    async fn update<T>(
        &self,
        session_id: &str,
        operation: impl FnOnce(&mut Session, DateTime<Utc>) -> Result<T>,
    ) -> Result<T> {
        self.find(session_id).await?.update(operation).await
    }
}

impl SessionHandle {
    async fn update<T>(
        self: &Arc<Self>,
        operation: impl FnOnce(&mut Session, DateTime<Utc>) -> Result<T>,
    ) -> Result<T> {
        let mut session = self.state.write().await;
        let now = Utc::now();
        // A late request cannot save a finding even if the timer task is delayed.
        // Publish expiry even when the requested action subsequently fails.
        if session.tick(now) {
            self.publish(&mut session);
        }
        let was_searching = session.phase == Phase::Searching;
        let result = operation(&mut session, now)?;
        self.publish(&mut session);
        if !was_searching && session.phase == Phase::Searching {
            self.start_search_timer(session.search_ends_at.unwrap());
        }
        Ok(result)
    }

    fn publish(&self, session: &mut Session) {
        session.revision += 1;
        // There may be no sockets yet. Mutations must still succeed.
        let _ = self.events.send(SessionEvent::StateChanged {
            revision: session.revision,
        });
    }

    fn start_search_timer(self: &Arc<Self>, deadline: i64) {
        let delay = deadline
            .saturating_sub(Utc::now().timestamp_millis())
            .max(0) as u64;
        let wake_at = tokio::time::Instant::now() + Duration::from_millis(delay);
        let handle = Arc::downgrade(self);
        tokio::spawn(async move {
            tokio::time::sleep_until(wake_at).await;
            if let Some(handle) = handle.upgrade() {
                let mut session = handle.state.write().await;
                if session.search_ends_at == Some(deadline)
                    && session.tick(DateTime::from_timestamp_millis(deadline).unwrap())
                {
                    handle.publish(&mut session);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Position;

    #[tokio::test]
    async fn late_save_publishes_expiry_even_before_the_timer_runs() {
        let manager = SessionManager::new();
        let created = manager.start_session("Alex".into(), 60).await.unwrap();
        let handle = manager.find(&created.session_id).await.unwrap();
        let mut connection = manager
            .connect(&created.session_id, created.player_id)
            .await
            .unwrap();
        let category;
        let revision;
        {
            let mut state = handle.state.write().await;
            let step = state.step_id;
            state
                .ready(created.player_id, step, true, Utc::now())
                .unwrap();
            category = state
                .add_category(created.player_id, "Door".into())
                .unwrap();
            let step = state.step_id;
            state
                .ready(created.player_id, step, true, Utc::now())
                .unwrap();
            state.search_ends_at = Some(Utc::now().timestamp_millis() - 1);
            revision = state.revision;
        }
        while connection.updates.try_recv().is_ok() {}
        let result = manager
            .add_finding(
                &created.session_id,
                created.player_id,
                category,
                Viewpoint {
                    panorama_id: "real-panorama".into(),
                    position: Position {
                        lat: 52.52,
                        lng: 13.405,
                    },
                    heading: 90.0,
                    pitch: 0.0,
                    zoom: 1.0,
                },
            )
            .await;
        assert!(matches!(result, Err(SessionError::Conflict(_))));
        assert_eq!(
            connection.updates.try_recv().unwrap(),
            SessionEvent::StateChanged {
                revision: revision + 1
            }
        );
        let state = manager
            .get_session(&created.session_id, created.player_id)
            .await
            .unwrap();
        assert_eq!(state.phase, Phase::Selection);
        assert!(state.own_findings.is_empty());
        assert_eq!(state.revision, revision + 1);
    }

    #[tokio::test]
    async fn a_busy_session_does_not_lock_other_rooms() {
        let manager = SessionManager::new();
        let first = manager.start_session("Alex".into(), 60).await.unwrap();
        let second = manager.start_session("Mika".into(), 60).await.unwrap();
        let first_handle = manager.find(&first.session_id).await.unwrap();
        let _guard = first_handle.state.write().await;
        let snapshot = tokio::time::timeout(
            Duration::from_millis(100),
            manager.get_session(&second.session_id, second.player_id),
        )
        .await;
        assert!(snapshot.unwrap().is_ok());
    }
}
