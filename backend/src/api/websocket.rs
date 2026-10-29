use std::time::Duration;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket, rejection::WebSocketUpgradeRejection},
    },
    response::Response,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::Deserialize;
use tokio::{
    sync::{broadcast, watch},
    time::{Instant, MissedTickBehavior, interval_at, sleep_until, timeout},
};

use super::{
    AppState,
    auth::Member,
    error::{ApiError, ApiResult},
};
use crate::session::{Position, SessionConnection, SessionEvent};

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum PresenceMessage {
    Position {
        step_id: uuid::Uuid,
        position: Option<Position>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct SocketSettings {
    pub(super) ping_interval: Duration,
    pub(super) heartbeat_timeout: Duration,
    pub(super) send_timeout: Duration,
}

impl Default for SocketSettings {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(15),
            heartbeat_timeout: Duration::from_secs(45),
            send_timeout: Duration::from_secs(10),
        }
    }
}

pub(super) async fn upgrade(
    State(state): State<AppState>,
    member: Member,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> ApiResult<Response> {
    // Reject invalid membership before returning a successful upgrade.
    state
        .sessions
        .get_session(&member.session_id, member.player_id)
        .await?;
    let upgrade = upgrade.map_err(|error| ApiError::new(error.status(), error.body_text()))?;
    Ok(upgrade
        .max_message_size(16 * 1024)
        .max_frame_size(16 * 1024)
        .on_upgrade(move |socket| serve(socket, state, member)))
}

async fn serve(socket: WebSocket, state: AppState, member: Member) {
    // Register only after upgrade succeeds; failed HTTP upgrades leave no presence.
    let Ok(connection) = state
        .sessions
        .connect(&member.session_id, member.player_id)
        .await
    else {
        return;
    };
    let SessionConnection {
        connection_id,
        updates,
    } = connection;
    let (mut outgoing, incoming) = socket.split();
    let (pongs, last_pong) = watch::channel(Instant::now());
    let mut shutdown = state.shutdown.subscribe();
    {
        let send = send_updates(&mut outgoing, updates, &state, &member);
        let receive = receive_messages(incoming, pongs, &state, &member, connection_id);
        let heartbeat = heartbeat_expired(last_pong, state.socket_settings.heartbeat_timeout);
        let stop = async {
            if !*shutdown.borrow() {
                let _ = shutdown.wait_for(|stop| *stop).await;
            }
        };
        tokio::select! {
            _ = send => {}
            _ = receive => {}
            _ = heartbeat => {}
            _ = stop => {}
        }
    }
    // Cancel all socket work before cleanup. Only this connection is unregistered.
    let _ = state
        .sessions
        .disconnect(&member.session_id, member.player_id, connection_id)
        .await;
    let _ = timeout(
        state.socket_settings.send_timeout,
        outgoing.send(Message::Close(None)),
    )
    .await;
}

async fn send_updates(
    outgoing: &mut SplitSink<WebSocket, Message>,
    mut updates: broadcast::Receiver<SessionEvent>,
    state: &AppState,
    member: &Member,
) {
    let settings = state.socket_settings;
    let mut ping = interval_at(
        Instant::now() + settings.ping_interval,
        settings.ping_interval,
    );
    ping.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut proximity_tick = interval_at(Instant::now(), Duration::from_secs(1));
    proximity_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_proximity = None;
    loop {
        let message = tokio::select! {
            event = updates.recv() => {
                // Explicit leave revokes membership and must also close its sockets.
                if state.sessions.get_session(&member.session_id, member.player_id).await.is_err() {
                    break;
                }
                let payload = match event {
                    Ok(event) => match serde_json::to_string(&event) {
                        Ok(payload) => payload,
                        Err(_) => break,
                    },
                    // Notify rather than terminate a slow subscriber: GET repairs its state.
                    Err(broadcast::error::RecvError::Lagged(_)) => r#"{"type":"state-changed"}"#.into(),
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                Message::Text(payload.into())
            }
            _ = ping.tick() => Message::Ping(Default::default()),
            _ = proximity_tick.tick() => {
                let Ok(proximity) = state.sessions.proximity(&member.session_id, member.player_id).await else { break; };
                if proximity == last_proximity { continue; }
                last_proximity = proximity;
                let Some(proximity) = proximity else { continue; };
                Message::Text(serde_json::json!({ "type": "proximity", "stepId": proximity.step_id, "band": proximity.band }).to_string().into())
            }
        };
        if !matches!(
            timeout(settings.send_timeout, outgoing.send(message)).await,
            Ok(Ok(()))
        ) {
            break;
        }
    }
}

async fn receive_messages(
    mut incoming: SplitStream<WebSocket>,
    pongs: watch::Sender<Instant>,
    state: &AppState,
    member: &Member,
    connection_id: uuid::Uuid,
) {
    let mut last_position = None;
    while let Some(Ok(message)) = incoming.next().await {
        match message {
            Message::Close(_) => break,
            Message::Pong(_) => {
                pongs.send_replace(Instant::now());
            }
            Message::Text(text) => {
                if let Ok(PresenceMessage::Position { step_id, position }) =
                    serde_json::from_str(&text)
                {
                    let now = Instant::now();
                    if position.is_some()
                        && last_position.is_some_and(|last| {
                            now.duration_since(last) < Duration::from_millis(750)
                        })
                    {
                        continue;
                    }
                    if position.is_some() {
                        last_position = Some(now);
                    }
                    let _ = state
                        .sessions
                        .report_position(
                            &member.session_id,
                            member.player_id,
                            connection_id,
                            step_id,
                            position,
                        )
                        .await;
                }
            }
            // Axum handles protocol Ping replies. Game actions use authenticated HTTP.
            _ => {}
        }
    }
}

async fn heartbeat_expired(mut pongs: watch::Receiver<Instant>, duration: Duration) {
    loop {
        let deadline = *pongs.borrow_and_update() + duration;
        tokio::select! {
            _ = sleep_until(deadline) => break,
            changed = pongs.changed() => if changed.is_err() { break; },
        }
    }
}
