use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Debug, Clone, Copy)]
enum MuteKind {
    Mute,
    Unmute,
}

struct AppState {
    tx: broadcast::Sender<MuteKind>,
}

// Return 3000 if the PORT environment variable is not set
fn port() -> u16 {
    const DEFAULT_PORT: u16 = 3000;

    match std::env::var("PORT") {
        Ok(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                tracing::error!("Invalid port number specified: {}", port);
                DEFAULT_PORT
            }
        },
        Err(_) => DEFAULT_PORT,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (tx, _rx) = broadcast::channel(32);

    let app = Router::new()
        .route("/mute", post(send_mute_req))
        .route("/unmute", post(send_unmute_req))
        .route("/watch", get(watch_for_req))
        .route("/ok", get(ok))
        .with_state(Arc::new(AppState { tx }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port()));

    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ok() -> StatusCode {
    StatusCode::OK
}

async fn send_mute_req(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.tx.send(MuteKind::Mute) {
        Err(e) => {
            tracing::error!("Failed to issue mute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

async fn send_unmute_req(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.tx.send(MuteKind::Unmute) {
        Err(e) => {
            tracing::error!("Failed to issue unmute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

/// Using WebSocket
async fn watch_for_req(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_req_watcher(socket, state))
}

/// Using WebSocket
async fn handle_req_watcher(ws: WebSocket, state: Arc<AppState>) {
    tracing::info!("Connected via WebSocket");

    let (mut ws_tx, mut ws_rx) = ws.split();

    // Watch close message of websocket
    let watch_ws_close_task = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            if let Ok(msg) = message {
                match msg {
                    Message::Close(_) => break,

                    msg => {
                        tracing::info!("Discard a message sent via WebSocket: {:?}", msg);
                    }
                }
            } else {
                break;
            };
        }
    });

    let mut rx = state.tx.subscribe();

    let mute_task = tokio::spawn(async move {
        while let Ok(kind) = rx.recv().await {
            match kind {
                MuteKind::Mute => {
                    if ws_tx.send(Message::Text("mute".to_string())).await.is_err() {
                        break;
                    }
                }

                MuteKind::Unmute => {
                    if ws_tx
                        .send(Message::Text("unmute".to_string()))
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
        // Wait for WebSocket to close
        _ = watch_ws_close_task => {
            tracing::info!("WebSocket closed");
        },

        // When the sender of a `broadcast channel` is no longer available, this task terminates
        // But because tx is shared `with_state` method, the sender is not lost until the server terminates
        // Therefore, this `select!` should always terminate with the close of websockets
        _ = mute_task => {
            tracing::info!("The websocket handler returned in a way it should not have, but there is no problem");
        },
    }
}
