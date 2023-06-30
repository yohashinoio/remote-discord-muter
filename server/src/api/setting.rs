use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::utils::{kind::RequestKind, state::AppState};

#[derive(serde::Serialize)]
pub struct MuteSetting {
    pub mute: bool,
}

pub async fn get_mute_setting(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<MuteSetting>) {
    get_mute_setting_internal(uuid, state).await
}

pub async fn get_mute_setting_internal(
    uuid: Uuid,
    state: Arc<AppState>,
) -> (StatusCode, Json<MuteSetting>) {
    let (resp_tx, resp_rx) = oneshot::channel();

    let tx = match state.get_request_sender(&uuid) {
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
