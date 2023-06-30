use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::utils::{
    kind::{MuteKind, RequestKind},
    other::Watcher,
    state::AppState,
};

use super::setting::get_mute_setting_internal;

pub async fn get_watchers(State(state): State<Arc<AppState>>) -> Json<Vec<Watcher>> {
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

pub async fn watch_mute_setting(
    Path(uuid): Path<Uuid>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| watch_mute_setting_internal(uuid, socket, state))
}

pub async fn watch_mute_setting_internal(uuid: Uuid, mut ws: WebSocket, state: Arc<AppState>) {
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
    let (code, mute_setting) = get_mute_setting_internal(uuid, state.clone()).await;
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

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ResponseId(Uuid);

impl std::fmt::Display for ResponseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn watch_mute_request(
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

    ws.on_upgrade(move |socket| watch_mute_request_internal(uuid, socket, state))
}

async fn watch_mute_request_internal(uuid: Uuid, ws: WebSocket, state: Arc<AppState>) {
    tracing::info!("Websocket opened: {}", uuid);

    let (mut ws_tx, mut ws_rx) = ws.split();

    let (tx, mut rx) = mpsc::channel(32);

    // Add a request sender
    state
        .request_senders
        .lock()
        .unwrap()
        .insert(uuid.clone(), tx);

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
    state.request_senders.lock().unwrap().remove(&uuid).unwrap();
}
