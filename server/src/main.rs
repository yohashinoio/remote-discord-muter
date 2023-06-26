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
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use uuid::Uuid;

#[derive(Debug)]
enum RequestKind {
    Mute,
    Unmute,
    GetMuteStatus { resp: oneshot::Sender<bool> },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Watcher {
    uuid: Uuid,
    username: String,
    user_id: String,
    avatar_id: String,
}

struct AppState {
    // For sending various requests to watchers
    requestors: Mutex<HashMap<Uuid, mpsc::Sender<RequestKind>>>,

    watchers_info: Mutex<HashMap<Uuid, Watcher>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            requestors: Mutex::new(HashMap::new()),
            watchers_info: Mutex::new(HashMap::new()),
        }
    }

    fn get_requestor(&self, uuid: &Uuid) -> Option<mpsc::Sender<RequestKind>> {
        self.requestors.lock().unwrap().get(uuid).map(|x| x.clone())
    }

    fn add_requestor(&self, uuid: Uuid, sender: mpsc::Sender<RequestKind>) {
        self.requestors.lock().unwrap().insert(uuid.clone(), sender);
    }

    fn remove_requestor(&self, uuid: &Uuid) {
        self.requestors.lock().unwrap().remove(uuid).unwrap();
    }
}

fn get_expose_port() -> u16 {
    const P: u16 = 3000;

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

    let app = Router::new()
        .route("/mute/:uuid", post(send_mute_request))
        .route("/unmute/:uuid", post(send_unmute_request))
        .route(
            "/watch/:username/:user_id/:avatar_id",
            get(watch_for_requests),
        )
        .route("/watchers", get(get_request_watchers))
        .route("/status/mute/:uuid", get(get_mute_status))
        .route("/ok", get(ok))
        .with_state(Arc::new(AppState::new()))
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
struct MuteStatus {
    mute: bool,
}

async fn get_mute_status(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<MuteStatus>) {
    let (resp_tx, resp_rx) = oneshot::channel();

    let tx = match state.get_requestor(&uuid) {
        Some(tx) => tx,
        None => {
            tracing::error!("Non-existent uuid: {}", uuid);
            return (StatusCode::BAD_REQUEST, Json(MuteStatus { mute: false }));
        }
    };

    if let Err(e) = tx.send(RequestKind::GetMuteStatus { resp: resp_tx }).await {
        tracing::error!("Failed to send mute request: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MuteStatus { mute: false }),
        );
    }

    match resp_rx.await {
        Ok(kind) => (StatusCode::OK, Json(MuteStatus { mute: kind })),

        Err(_) => {
            tracing::error!("Getting mute status was cancelled");
            (StatusCode::NOT_FOUND, Json(MuteStatus { mute: false }))
        }
    }
}

async fn get_request_watchers(State(state): State<Arc<AppState>>) -> Json<Vec<Watcher>> {
    Json(
        state
            .watchers_info
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

async fn send_mute_request(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
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

async fn send_unmute_request(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
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

async fn watch_for_requests(
    Path((username, user_id, avatar_id)): Path<(String, String, String)>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4();

    state.watchers_info.lock().unwrap().insert(
        uuid,
        Watcher {
            uuid,
            username,
            user_id,
            avatar_id,
        },
    );

    ws.on_upgrade(move |socket| handle_watcher(uuid, socket, state))
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResponseId(Uuid);

impl fmt::Display for ResponseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

async fn handle_watcher(uuid: Uuid, ws: WebSocket, state: Arc<AppState>) {
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

    let mute_task = tokio::spawn(async move {
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

                RequestKind::GetMuteStatus { resp } => {
                    let resp_id = ResponseId(Uuid::new_v4());

                    tracing::info!("Request to get mute status (id = {})", resp_id);

                    responders.lock().unwrap().insert(resp_id, resp);

                    if ws_tx
                        .send(Message::Text(format!("GET STATUS MUTE {}", resp_id)))
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

        _ = mute_task => {
            // Improbable
            tracing::info!("Empty receivers before WebSocket closes");
        },
    }

    // Cleanup
    state.watchers_info.lock().unwrap().remove(&uuid);
    state.remove_requestor(&uuid);
}
