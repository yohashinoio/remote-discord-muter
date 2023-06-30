use std::sync::Arc;

use axum::{http::StatusCode, routing::get, routing::post, Router};

use crate::{
    api::{
        mute::{send_mute_request, send_unmute_request},
        setting::get_mute_setting,
        watch::{get_watchers, watch_mute_request, watch_mute_setting},
    },
    utils::state::AppState,
};

async fn ok() -> StatusCode {
    StatusCode::OK
}

pub fn routes() -> Router {
    Router::new()
        .route("/mute/:uuid", post(send_mute_request))
        .route("/unmute/:uuid", post(send_unmute_request))
        .route("/setting/mute/:uuid", get(get_mute_setting))
        .route("/watchers", get(get_watchers))
        .route("/watch/setting/mute/:uuid", get(watch_mute_setting))
        .route(
            "/watch/:username/:user_id/:avatar_id",
            get(watch_mute_request),
        )
        .route("/ok", get(ok))
        .with_state(Arc::new(AppState::new()))
}
