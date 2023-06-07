use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::put;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy)]
enum MuteKind {
    Mute,
    Unmute,
}

struct AppState {
    tx: broadcast::Sender<MuteKind>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (tx, _rx) = broadcast::channel(32);

    let app = Router::new()
        .route("/mute", put(mute))
        .route("/unmute", put(unmute))
        .route("/ws", get(websocket_handler))
        .with_state(Arc::new(AppState { tx }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Issue a mute request
async fn mute(State(state): State<Arc<AppState>>) -> StatusCode {
    tracing::info!("Mute request issued");

    match state.tx.send(MuteKind::Mute) {
        Err(e) => {
            tracing::error!("Failed to issue mute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

/// Issue a unmute request
async fn unmute(State(state): State<Arc<AppState>>) -> StatusCode {
    tracing::info!("Unmute request issued");

    match state.tx.send(MuteKind::Unmute) {
        Err(e) => {
            tracing::error!("Failed to issue unmute request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }

        Ok(_) => StatusCode::OK,
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(ws: WebSocket, state: Arc<AppState>) {
    tracing::info!("Connected via WebSocket");

    let (mut ws_tx, mut ws_rx) = ws.split();

    // Watch close message of websocket
    let watch_ws_close_task = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            if let Ok(msg) = message {
                match msg {
                    Message::Close(_) => break,

                    msg => {
                        tracing::info!("Ignore a message sent via WebSocket: {:?}", msg);
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
