use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Body,
    http::{Method, Request, header},
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tokio::{
    net::TcpStream,
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WsError, Message as WsMessage, client::IntoClientRequest},
};
use tower::ServiceExt;

use super::*;

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
}

impl Reply {
    fn cookie(&self) -> String {
        self.headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .into()
    }

    fn id(&self) -> String {
        self.body["id"].as_str().unwrap().into()
    }
}

fn app() -> (AppState, Router) {
    let state = AppState::new(SessionManager::new(), ApiConfig::default());
    (state.clone(), router(state))
}

async fn call(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = if body.is_empty() {
        Value::Null
    } else {
        assert!(
            headers[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        serde_json::from_slice(&body).unwrap()
    };
    if status.is_client_error() || status.is_server_error() {
        assert!(
            body["message"].as_str().is_some(),
            "Missing error message: {body}"
        );
    }
    Reply {
        status,
        headers,
        body,
    }
}

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "localhost:3000");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let body = if let Some(body) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    call(app, builder.body(body).unwrap()).await
}

async fn create(app: &Router, name: &str, cookie: Option<&str>) -> (String, String) {
    let reply = request(
        app,
        Method::POST,
        "/api/sessions",
        cookie,
        Some(json!({
            "name": name, "durationSeconds": 60,
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    (reply.id(), reply.cookie())
}

async fn join(app: &Router, id: &str, name: &str, cookie: Option<&str>) -> String {
    let reply = request(
        app,
        Method::POST,
        &format!("/api/sessions/{id}/join"),
        cookie,
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    reply.cookie()
}

async fn snapshot(app: &Router, id: &str, cookie: &str) -> Value {
    let reply = request(
        app,
        Method::GET,
        &format!("/api/sessions/{id}"),
        Some(cookie),
        None,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    reply.body
}

async fn ready_all(app: &Router, id: &str, cookies: &[&str]) {
    let state = snapshot(app, id, cookies[0]).await;
    for cookie in cookies {
        let reply = request(
            app,
            Method::PUT,
            &format!("/api/sessions/{id}/ready"),
            Some(cookie),
            Some(json!({
                "stepId": state["stepId"], "ready": true,
            })),
        )
        .await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.body);
    }
}

fn viewpoint(panorama: &str) -> Value {
    json!({ "panoramaId": panorama, "position": { "lat": 52.52, "lng": 13.405 }, "heading": 90, "pitch": 0, "zoom": 1 })
}

#[tokio::test(start_paused = true)]
async fn http_routes_play_a_complete_round_with_private_findings_and_scores() {
    let (_, app) = app();
    let (id, alex) = create(&app, "Alex", None).await;
    let mika = join(&app, &id, "Mika", None).await;
    let base = format!("/api/sessions/{id}");
    ready_all(&app, &id, &[&alex, &mika]).await;
    let category = request(
        &app,
        Method::POST,
        &format!("{base}/categories"),
        Some(&alex),
        Some(json!({ "text": "Door" })),
    )
    .await
    .id();
    let spare = request(
        &app,
        Method::POST,
        &format!("{base}/categories"),
        Some(&mika),
        Some(json!({ "text": "Spare" })),
    )
    .await
    .id();
    assert_eq!(
        request(
            &app,
            Method::PATCH,
            &format!("{base}/categories/{category}"),
            Some(&mika),
            Some(json!({ "text": "A blue door" }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &app,
            Method::DELETE,
            &format!("{base}/categories/{spare}"),
            Some(&alex),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    ready_all(&app, &id, &[&alex, &mika]).await;
    let first = request(
        &app,
        Method::POST,
        &format!("{base}/findings"),
        Some(&alex),
        Some(json!({ "categoryId": category, "view": viewpoint("first") })),
    )
    .await
    .id();
    let selected = request(
        &app,
        Method::POST,
        &format!("{base}/findings"),
        Some(&alex),
        Some(json!({ "categoryId": category, "view": viewpoint("alex-private") })),
    )
    .await
    .id();
    let mikas = request(
        &app,
        Method::POST,
        &format!("{base}/findings"),
        Some(&mika),
        Some(json!({ "categoryId": category, "view": viewpoint("mika-private") })),
    )
    .await
    .id();
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/selections/{category}"),
            Some(&alex),
            Some(json!({ "findingId": selected }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &app,
            Method::DELETE,
            &format!("{base}/findings/{first}"),
            Some(&alex),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &app,
            Method::DELETE,
            &format!("{base}/findings/{selected}"),
            Some(&mika),
            None
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/selections/{category}"),
            Some(&mika),
            Some(json!({ "findingId": selected }))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let state = snapshot(&app, &id, &alex).await;
    assert_eq!(state["ownFindings"].as_array().unwrap().len(), 1);
    assert!(!state.to_string().contains("mika-private"));
    let mut demo = viewpoint("demo");
    demo["demoScene"] = json!(1);
    assert_eq!(
        request(
            &app,
            Method::POST,
            &format!("{base}/findings"),
            Some(&alex),
            Some(json!({ "categoryId": category, "view": demo }))
        )
        .await
        .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    tokio::time::advance(Duration::from_secs(61)).await;
    tokio::task::yield_now().await;
    assert_eq!(snapshot(&app, &id, &alex).await["phase"], "selection");
    assert_eq!(
        request(
            &app,
            Method::POST,
            &format!("{base}/findings"),
            Some(&alex),
            Some(json!({ "categoryId": category, "view": viewpoint("late") }))
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    ready_all(&app, &id, &[&alex, &mika]).await;
    let state = snapshot(&app, &id, &mika).await;
    assert_eq!(state["review"]["id"], selected);
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&mika),
            Some(json!({ "stepId": state["stepId"], "ready": true }))
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/review/{selected}/vote"),
            Some(&alex),
            Some(json!({ "vote": "accept" }))
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/review/{selected}/vote"),
            Some(&mika),
            Some(json!({ "vote": "accept" }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    let owner = snapshot(&app, &id, &alex).await;
    assert!(owner["review"]["ownVote"].is_null());
    assert!(owner["review"]["result"].is_null());
    assert_eq!(owner["players"][1]["ready"], true);
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&alex),
            Some(json!({ "stepId": owner["stepId"], "ready": true }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    let next = snapshot(&app, &id, &alex).await;
    assert_eq!(next["results"][0]["accepted"], true);
    assert_eq!(next["players"][0]["score"], 1);
    assert_eq!(next["review"]["id"], mikas);
    assert_eq!(next["review"]["votesCast"], 0);
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&alex),
            Some(json!({ "stepId": state["stepId"], "ready": true }))
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/review/{selected}/vote"),
            Some(&mika),
            Some(json!({ "vote": "reject" }))
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    // Ready the owner first so the last vote itself triggers the final transition.
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&mika),
            Some(json!({ "stepId": next["stepId"], "ready": true }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/review/{mikas}/vote"),
            Some(&alex),
            Some(json!({ "vote": "reject" }))
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    let finished = snapshot(&app, &id, &alex).await;
    assert_eq!(finished["phase"], "finished");
    assert_eq!(finished["results"].as_array().unwrap().len(), 2);
    assert_eq!(finished["players"][0]["score"], 1);
    assert_eq!(finished["players"][1]["score"], 0);
    assert_eq!(
        request(
            &app,
            Method::POST,
            &format!("{base}/leave"),
            Some(&mika),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, Method::GET, &base, Some(&mika), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(snapshot(&app, &id, &alex).await["players"][1]["left"], true);
}

#[tokio::test]
async fn cookies_are_opaque_and_memberships_are_scoped_to_rooms() {
    let (_, app) = app();
    let (id, cookie) = create(&app, "  Alex  ", None).await;
    let base = format!("/api/sessions/{id}");
    let state = snapshot(&app, &id.to_ascii_lowercase(), &cookie).await;
    assert_eq!(state["players"][0]["name"], "Alex");
    assert_eq!(
        request(&app, Method::GET, &base, None, None).await.status,
        StatusCode::UNAUTHORIZED
    );
    let forged = format!("sls_identity={}", state["meId"].as_str().unwrap());
    assert_eq!(
        request(&app, Method::GET, &base, Some(&forged), None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let (other, same_cookie) = create(&app, "Same browser", Some(&cookie)).await;
    assert_eq!(cookie, same_cookie);
    assert_eq!(snapshot(&app, &id, &cookie).await["meId"], state["meId"]);
    assert_ne!(snapshot(&app, &other, &cookie).await["meId"], state["meId"]);
    let (_, foreign) = create(&app, "Stranger", None).await;
    assert_eq!(
        request(&app, Method::GET, &base, Some(&foreign), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    join(&app, &id, "Alex", Some(&cookie)).await;
    assert_eq!(
        snapshot(&app, &id, &cookie).await["players"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        request(
            &app,
            Method::POST,
            &format!("{base}/leave"),
            Some(&cookie),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, Method::GET, &base, Some(&cookie), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        snapshot(&app, &other, &cookie).await["players"][0]["left"],
        false
    );
}

#[tokio::test]
async fn concurrent_joins_with_an_existing_browser_cookie_create_one_membership() {
    let (_, app) = app();
    let (_, browser) = create(&app, "Existing browser", None).await;
    let (id, owner) = create(&app, "Owner", None).await;
    let (first, second) = tokio::join!(
        join(&app, &id, "Peer", Some(&browser)),
        join(&app, &id, "Peer", Some(&browser))
    );
    assert_eq!(first, second);
    assert_eq!(
        snapshot(&app, &id, &owner).await["players"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn delayed_leave_does_not_revoke_a_newer_membership() {
    let (state, app) = app();
    let (id, cookie) = create(&app, "Alex", None).await;
    let original = snapshot(&app, &id, &cookie).await;
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, cookie.parse().unwrap());
    let (_, identity) = state.credentials.lookup(&headers).await.unwrap();
    // Model a request that authenticated before a concurrent leave/rejoin.
    let delayed = auth::Member {
        session_id: id.clone(),
        player_id: original["meId"].as_str().unwrap().parse().unwrap(),
        identity,
    };
    request(
        &app,
        Method::POST,
        &format!("/api/sessions/{id}/leave"),
        Some(&cookie),
        None,
    )
    .await;
    join(&app, &id, "Alex again", Some(&cookie)).await;
    assert_eq!(
        handlers::leave(State(state), delayed).await.ok(),
        Some(StatusCode::NO_CONTENT)
    );
    let returned = snapshot(&app, &id, &cookie).await;
    assert_ne!(returned["meId"], original["meId"]);
    assert_eq!(returned["players"][1]["left"], false);
}

#[tokio::test]
async fn rejection_responses_are_json_and_untrusted_identity_fields_are_rejected() {
    let (_, app) = app();
    let (id, cookie) = create(&app, "Alex", None).await;
    let base = format!("/api/sessions/{id}");
    assert_eq!(
        request(&app, Method::GET, "/missing", None, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&app, Method::DELETE, "/api/sessions", None, None)
            .await
            .status,
        StatusCode::METHOD_NOT_ALLOWED
    );
    assert_eq!(
        request(
            &app,
            Method::POST,
            "/api/sessions/MISSING/join",
            None,
            Some(json!({ "name": "Alex" }))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            Method::POST,
            "/api/sessions",
            None,
            Some(json!({ "name": "Alex", "durationSeconds": 0 }))
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&cookie),
            Some(json!({ "stepId": "invalid", "ready": true }))
        )
        .await
        .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let current = snapshot(&app, &id, &cookie).await;
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &format!("{base}/ready"),
            Some(&cookie),
            Some(
                json!({ "stepId": current["stepId"], "ready": true, "playerId": current["meId"] })
            )
        )
        .await
        .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(
            &app,
            Method::DELETE,
            &format!("{base}/findings/invalid-uuid"),
            Some(&cookie),
            None
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &app,
            Method::GET,
            &format!("{base}/ws"),
            Some(&cookie),
            None
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        snapshot(&app, &id, &cookie).await["players"][0]["connected"],
        false
    );
    let malformed = Request::post("/api/sessions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .unwrap();
    assert_eq!(call(&app, malformed).await.status, StatusCode::BAD_REQUEST);
    let wrong_media = Request::post("/api/sessions")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        call(&app, wrong_media).await.status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    let oversized = Request::post("/api/sessions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(" ".repeat(17000)))
        .unwrap();
    assert_eq!(
        call(&app, oversized).await.status,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn origin_checks_support_the_vite_proxy_and_configured_public_origin() {
    let (_, app) = app();
    for (origin, expected) in [
        ("http://localhost:5173", StatusCode::CREATED),
        ("https://evil.example", StatusCode::FORBIDDEN),
        ("null", StatusCode::FORBIDDEN),
    ] {
        let request = Request::post("/api/sessions")
            .header(header::HOST, "localhost:5173")
            .header(header::ORIGIN, origin)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "name": "Alex", "durationSeconds": 60 }).to_string(),
            ))
            .unwrap();
        assert_eq!(call(&app, request).await.status, expected);
    }
    let config = ApiConfig {
        secure_cookies: true,
        public_origin: Some("https://game.example".parse().unwrap()),
    };
    let app = router(AppState::new(SessionManager::new(), config));
    let request = Request::post("/api/sessions")
        .header(header::HOST, "backend:3000")
        .header(header::ORIGIN, "https://game.example")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Alex", "durationSeconds": 60 }).to_string(),
        ))
        .unwrap();
    let reply = call(&app, request).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    let cookie = reply.headers[header::SET_COOKIE].to_str().unwrap();
    for attribute in ["HttpOnly", "SameSite=Strict", "Path=/api", "Secure"] {
        assert!(cookie.contains(attribute));
    }
    assert!(reply.body.get("playerId").is_none());
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Server {
    address: SocketAddr,
    state: AppState,
    app: Router,
    task: JoinHandle<()>,
}

impl Server {
    async fn start(settings: websocket::SocketSettings) -> Self {
        let mut state = AppState::new(SessionManager::new(), ApiConfig::default());
        state.socket_settings = settings;
        let app = router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let service = app.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, service).await.unwrap();
        });
        Self {
            address,
            state,
            app,
            task,
        }
    }

    async fn socket(
        &self,
        id: &str,
        cookie: Option<&str>,
        origin: Option<&str>,
    ) -> Result<Socket, WsError> {
        let mut request = format!("ws://{}/api/sessions/{id}/ws", self.address)
            .into_client_request()
            .unwrap();
        if let Some(cookie) = cookie {
            request
                .headers_mut()
                .insert(header::COOKIE, cookie.parse().unwrap());
        }
        if let Some(origin) = origin {
            request
                .headers_mut()
                .insert(header::ORIGIN, origin.parse().unwrap());
        }
        connect_async(request).await.map(|(socket, _)| socket)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.state.shutdown();
        self.task.abort();
    }
}

async fn notification(socket: &mut Socket) -> Value {
    loop {
        match timeout(Duration::from_secs(3), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
        {
            WsMessage::Text(text) => {
                let value: Value = serde_json::from_str(&text).unwrap();
                assert_eq!(value["type"], "state-changed");
                assert!(
                    value
                        .as_object()
                        .unwrap()
                        .keys()
                        .all(|key| matches!(key.as_str(), "type" | "revision"))
                );
                return value;
            }
            WsMessage::Ping(data) => socket.send(WsMessage::Pong(data)).await.unwrap(),
            other => panic!("Unexpected frame: {other:?}"),
        }
    }
}

async fn connected(server: &Server, id: &str, cookie: &str, expected: bool) {
    timeout(Duration::from_secs(3), async {
        loop {
            let state = snapshot(&server.app, id, cookie).await;
            let me = state["meId"].clone();
            let player = state["players"]
                .as_array()
                .unwrap()
                .iter()
                .find(|player| player["id"] == me)
                .unwrap();
            if player["connected"] == expected {
                return;
            }
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn websocket_authentication_fanout_and_connection_cleanup_use_the_manager() {
    let server = Server::start(websocket::SocketSettings::default()).await;
    let (id, alex) = create(&server.app, "Alex", None).await;
    let mika = join(&server.app, &id, "Mika", None).await;
    for (cookie, origin, expected) in [
        (None, None, StatusCode::UNAUTHORIZED),
        (
            Some(alex.as_str()),
            Some("https://evil.example"),
            StatusCode::FORBIDDEN,
        ),
    ] {
        match server.socket(&id, cookie, origin).await {
            Err(WsError::Http(response)) => assert_eq!(response.status(), expected),
            _ => panic!("Expected a rejected upgrade"),
        }
    }
    let (_, stranger) = create(&server.app, "Stranger", None).await;
    assert!(
        matches!(server.socket(&id, Some(&stranger), None).await, Err(WsError::Http(response)) if response.status() == StatusCode::FORBIDDEN)
    );
    let origin = format!("http://{}", server.address);
    let mut first = server
        .socket(&id, Some(&alex), Some(&origin))
        .await
        .unwrap();
    notification(&mut first).await;
    let mut replacement = server
        .socket(&id, Some(&alex), Some(&origin))
        .await
        .unwrap();
    notification(&mut replacement).await;
    let mut other = server
        .socket(&id, Some(&mika), Some(&origin))
        .await
        .unwrap();
    notification(&mut other).await;
    connected(&server, &id, &alex, true).await;
    let step = snapshot(&server.app, &id, &alex).await["stepId"].clone();
    request(
        &server.app,
        Method::PUT,
        &format!("/api/sessions/{id}/ready"),
        Some(&alex),
        Some(json!({ "stepId": step, "ready": true })),
    )
    .await;
    let revision = snapshot(&server.app, &id, &alex).await["revision"]
        .as_u64()
        .unwrap();
    for socket in [&mut first, &mut replacement, &mut other] {
        while notification(socket).await["revision"].as_u64().unwrap() < revision {}
    }
    first.close(None).await.unwrap();
    notification(&mut replacement).await;
    connected(&server, &id, &alex, true).await;
    replacement.close(None).await.unwrap();
    connected(&server, &id, &alex, false).await;
    let state = snapshot(&server.app, &id, &alex).await;
    assert_eq!(state["players"][0]["ready"], true);
    assert_eq!(state["players"][0]["left"], false);
    assert_eq!(state["phase"], "lobby");
    other.close(None).await.unwrap();
    connected(&server, &id, &mika, false).await;
}

#[tokio::test]
async fn websocket_heartbeat_expires_silent_peers_and_closes_sockets_on_leave() {
    let settings = websocket::SocketSettings {
        ping_interval: Duration::from_millis(25),
        heartbeat_timeout: Duration::from_millis(200),
        send_timeout: Duration::from_millis(100),
    };
    let server = Server::start(settings).await;
    let (id, cookie) = create(&server.app, "Alex", None).await;
    let mut silent = server.socket(&id, Some(&cookie), None).await.unwrap();
    notification(&mut silent).await;
    connected(&server, &id, &cookie, false).await;
    assert_eq!(
        snapshot(&server.app, &id, &cookie).await["players"][0]["left"],
        false
    );
    drop(silent);
    let mut healthy = server.socket(&id, Some(&cookie), None).await.unwrap();
    notification(&mut healthy).await;
    let until = Instant::now() + Duration::from_millis(300);
    let mut pings = 0;
    while Instant::now() < until {
        match timeout(Duration::from_secs(1), healthy.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
        {
            WsMessage::Ping(data) => {
                pings += 1;
                healthy.send(WsMessage::Pong(data)).await.unwrap();
            }
            WsMessage::Text(_) => {}
            other => panic!("Unexpected heartbeat frame: {other:?}"),
        }
    }
    assert!(pings >= 2);
    connected(&server, &id, &cookie, true).await;
    assert_eq!(
        request(
            &server.app,
            Method::POST,
            &format!("/api/sessions/{id}/leave"),
            Some(&cookie),
            None
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    timeout(Duration::from_secs(2), async {
        while let Some(Ok(message)) = healthy.next().await {
            if matches!(message, WsMessage::Close(_)) {
                return;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(
        request(
            &server.app,
            Method::GET,
            &format!("/api/sessions/{id}"),
            Some(&cookie),
            None
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn websocket_shutdown_unregisters_the_connection() {
    let server = Server::start(websocket::SocketSettings::default()).await;
    let (id, cookie) = create(&server.app, "Alex", None).await;
    let mut socket = server.socket(&id, Some(&cookie), None).await.unwrap();
    notification(&mut socket).await;
    server.state.shutdown();
    connected(&server, &id, &cookie, false).await;
    timeout(Duration::from_secs(1), async {
        while let Some(Ok(message)) = socket.next().await {
            if matches!(message, WsMessage::Close(_)) {
                return;
            }
        }
    })
    .await
    .unwrap();
}
