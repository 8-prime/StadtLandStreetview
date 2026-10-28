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
use tokio::{
    sync::{broadcast, watch},
    time::{Instant, MissedTickBehavior, interval_at, sleep_until, timeout},
};

use super::{
    AppState,
    auth::Member,
    error::{ApiError, ApiResult},
};
use crate::session::{SessionConnection, SessionEvent};

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
        let receive = receive_messages(incoming, pongs);
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
        };
        if !matches!(
            timeout(settings.send_timeout, outgoing.send(message)).await,
            Ok(Ok(()))
        ) {
            break;
        }
    }
}

async fn receive_messages(mut incoming: SplitStream<WebSocket>, pongs: watch::Sender<Instant>) {
    while let Some(Ok(message)) = incoming.next().await {
        match message {
            Message::Close(_) => break,
            Message::Pong(_) => {
                pongs.send_replace(Instant::now());
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
