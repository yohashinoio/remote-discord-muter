use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::{routing::get, Router};
use futures::channel::oneshot;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use uuid::Uuid;

#[derive(Debug)]
enum RequestKind {
    Mute,
    Unmute,
    GetMuteSetting { resp: oneshot::Sender<bool> },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Watcher {
    uuid: Uuid,
    username: String,
    user_id: String,
    avatar_id: String,
}

#[derive(Debug)]
enum MuteKind {
    Muted,
    Unmuted,
}

struct AppState {
    // For sending various requests to watchers
    request_senders: Mutex<HashMap<Uuid, mpsc::Sender<RequestKind>>>,

    mute_setting_senders: Arc<tokio::sync::Mutex<HashMap<Uuid, mpsc::Sender<MuteKind>>>>,

    watchers: Mutex<HashMap<Uuid, Watcher>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            request_senders: Mutex::new(HashMap::new()),
            mute_setting_senders: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            watchers: Mutex::new(HashMap::new()),
        }
    }

    fn get_requestor(&self, uuid: &Uuid) -> Option<mpsc::Sender<RequestKind>> {
        self.request_senders
            .lock()
            .unwrap()
            .get(uuid)
            .map(|x| x.clone())
    }

    fn add_requestor(&self, uuid: Uuid, sender: mpsc::Sender<RequestKind>) {
        self.request_senders
            .lock()
            .unwrap()
            .insert(uuid.clone(), sender);
    }

    fn remove_requestor(&self, uuid: &Uuid) {
        self.request_senders.lock().unwrap().remove(uuid).unwrap();
    }
}

fn get_expose_port() -> u16 {
    const P: u16 = 8080;

    match std::env::var("PORT") {
        Ok(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                tracing::error!("Invalid expose port specified: {}", port);
                P
            }
        },
        Err(_) => P,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Serve the client
    let client_service = ServeDir::new("client")
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new("client/404.html"));

    let app = Router::new()
        .route("/mute/:uuid", post(mute_api))
        .route("/unmute/:uuid", post(unmute_api))
        .route("/setting/mute/:uuid", get(get_mute_setting_api))
        .route("/watchers", get(get_request_watchers_api))
        .route(
            "/watch/:username/:user_id/:avatar_id",
            get(watch_mute_request_api),
        )
        .route("/watch/setting/mute/:uuid", get(watch_mute_setting_api))
        .route("/ok", get(ok))
        .fallback_service(client_service)
        .with_state(Arc::new(AppState::new()))
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], get_expose_port()));

    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Serialize)]
struct MuteSetting {
    mute: bool,
}

async fn get_mute_setting_api(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<MuteSetting>) {
    get_mute_setting(uuid, state).await
}

async fn get_mute_setting(uuid: Uuid, state: Arc<AppState>) -> (StatusCode, Json<MuteSetting>) {
    let (resp_tx, resp_rx) = oneshot::channel();

    let tx = match state.get_requestor(&uuid) {
        Some(tx) => tx,
        None => {
            tracing::error!("Non-existent uuid: {}", uuid);
            return (StatusCode::BAD_REQUEST, Json(MuteSetting { mute: false }));
        }
    };

    if let Err(e) = tx.send(RequestKind::GetMuteSetting { resp: resp_tx }).await {
        tracing::error!("Failed to send mute request: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MuteSetting { mute: false }),
        );
    }

    match resp_rx.await {
        Ok(kind) => (StatusCode::OK, Json(MuteSetting { mute: kind })),

        Err(_) => {
            tracing::error!("Getting mute setting was cancelled");
            (StatusCode::NOT_FOUND, Json(MuteSetting { mute: false }))
        }
    }
}

async fn get_request_watchers_api(State(state): State<Arc<AppState>>) -> Json<Vec<Watcher>> {
    Json(
        state
            .watchers
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>(),
    )
}

async fn ok() -> StatusCode {
    StatusCode::OK
}

async fn mute_api(Path(uuid): Path<Uuid>, State(state): State<Arc<AppState>>) -> StatusCode {
    send_mute_request(uuid, state).await
}

async fn send_mute_request(uuid: Uuid, state: Arc<AppState>) -> StatusCode {
    let tx = match state.get_requestor(&uuid) {
        Some(tx) => tx,
        None => {
            tracing::error!("Non-existent uuid: {}", uuid);
            return StatusCode::BAD_REQUEST;
        }
    };

    match tx.send(RequestKind::Mute).await {
        Err(e) => {
            tracing::error!("Failed to send mute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

async fn unmute_api(Path(uuid): Path<Uuid>, State(state): State<Arc<AppState>>) -> StatusCode {
    send_unmute_request(uuid, state).await
}

async fn send_unmute_request(uuid: Uuid, state: Arc<AppState>) -> StatusCode {
    let tx = match state.get_requestor(&uuid) {
        Some(tx) => tx,
        None => {
            tracing::error!("Non-existent uuid: {}", uuid);
            return StatusCode::BAD_REQUEST;
        }
    };

    match tx.send(RequestKind::Unmute).await {
        Err(e) => {
            tracing::error!("Failed to send unmute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

async fn watch_mute_setting_api(
    Path(uuid): Path<Uuid>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| watch_mute_setting(uuid, socket, state))
}

async fn watch_mute_setting(uuid: Uuid, mut ws: WebSocket, state: Arc<AppState>) {
    if !state.watchers.lock().unwrap().contains_key(&uuid) {
        tracing::error!(
            "Tried to watch a mute setting for a uuid that does not exist: {}",
            uuid
        );
        ws.close().await.unwrap();
        return;
    }

    let (tx, mut rx) = mpsc::channel(32);

    state.mute_setting_senders.lock().await.insert(uuid, tx);

    // Send current mute setting
    let (code, mute_setting) = get_mute_setting(uuid, state.clone()).await;
    if code.is_success() {
        let setting_s = match mute_setting.0.mute {
            true => "muted",
            false => "unmuted",
        };

        if ws.send(Message::Text(setting_s.to_string())).await.is_err() {
            tracing::error!("Failed to send mute setting to {}: {}", uuid, setting_s);
        }
    } else {
        tracing::error!("Failed to get current mute setting");
    }

    while let Some(kind) = rx.recv().await {
        match kind {
            MuteKind::Muted => {
                if ws.send(Message::Text("muted".to_string())).await.is_err() {
                    tracing::error!("Failed to send mute setting to {}: {}", uuid, "muted");
                }
            }

            MuteKind::Unmuted => {
                if ws.send(Message::Text("unmuted".to_string())).await.is_err() {
                    tracing::error!("Failed to send mute setting to {}: {}", uuid, "unmuted");
                }
            }
        };
    }
}

async fn watch_mute_request_api(
    Path((username, user_id, avatar_id)): Path<(String, String, String)>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4();

    state.watchers.lock().unwrap().insert(
        uuid,
        Watcher {
            uuid,
            username,
            user_id,
            avatar_id,
        },
    );

    ws.on_upgrade(move |socket| watch_mute_request(uuid, socket, state))
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResponseId(Uuid);

impl fmt::Display for ResponseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

async fn watch_mute_request(uuid: Uuid, ws: WebSocket, state: Arc<AppState>) {
    tracing::info!("Websocket opened: {}", uuid);

    let (mut ws_tx, mut ws_rx) = ws.split();

    let (tx, mut rx) = mpsc::channel(32);
    state.add_requestor(uuid.clone(), tx);

    // Send uuid
    if ws_tx
        .send(Message::Text(format!("Your UUID is {}", uuid)))
        .await
        .is_err()
    {
        tracing::error!("Failed to send UUID via WebSocket");
        return;
    }

    let responders = Arc::new(Mutex::new(HashMap::<
        ResponseId,
        oneshot::Sender<bool>, /* Responder */
    >::new()));
    let cloned_responders = responders.clone();

    let mute_setting_senders = state.mute_setting_senders.clone();

    let ws_task = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            if let Ok(msg) = message {
                match msg {
                    Message::Close(_) => break,

                    Message::Text(text) => {
                        let mut iter = text.split_whitespace();

                        let start = match iter.next() {
                            Some(first) => first,
                            None => continue,
                        };

                        match start {
                            "muted" => {
                                if let Some(tx) = mute_setting_senders.lock().await.get(&uuid) {
                                    tx.send(MuteKind::Muted).await.unwrap();
                                }
                            }

                            "unmuted" => {
                                if let Some(tx) = mute_setting_senders.lock().await.get(&uuid) {
                                    tx.send(MuteKind::Unmuted).await.unwrap();
                                }
                            }

                            // Response
                            "RESP" => {
                                let resp_id = iter.next().unwrap();
                                let response = iter.next().unwrap();
                                assert_eq!(iter.next(), None);

                                let responder = cloned_responders
                                    .lock()
                                    .unwrap()
                                    .remove(&ResponseId(Uuid::from_str(resp_id).unwrap()))
                                    .unwrap();

                                let _ = responder.send(response.parse::<bool>().unwrap());

                                tracing::info!("Get response from muter (id = {})", resp_id);
                            }

                            _ => continue,
                        }
                    }

                    _ => continue,
                }
            } else {
                break;
            };
        }
    });

    let request_task = tokio::spawn(async move {
        while let Some(kind) = rx.recv().await {
            match kind {
                RequestKind::Mute => {
                    if ws_tx.send(Message::Text("mute".to_string())).await.is_err() {
                        break;
                    }
                }

                RequestKind::Unmute => {
                    if ws_tx
                        .send(Message::Text("unmute".to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                RequestKind::GetMuteSetting { resp } => {
                    let resp_id = ResponseId(Uuid::new_v4());

                    tracing::info!("Request to get mute setting (id = {})", resp_id);

                    responders.lock().unwrap().insert(resp_id, resp);

                    if ws_tx
                        .send(Message::Text(format!("GET SETTING MUTE {}", resp_id)))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        // Wait for websocket to close
        _ = ws_task => {
            tracing::info!("Websocket closed: {}", uuid);
        },

        _ = request_task => {
            // Improbable
            tracing::info!("Empty receivers before WebSocket closes");
        },
    }

    // Cleanup
    state.watchers.lock().unwrap().remove(&uuid);
    state.remove_requestor(&uuid);
}
