use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::utils::{kind::RequestKind, state::AppState};

pub async fn send_mute_request(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    let tx = match state.get_request_sender(&uuid) {
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

pub async fn send_unmute_request(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    let tx = match state.get_request_sender(&uuid) {
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
