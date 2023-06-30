use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use super::api;

pub fn router() -> axum::Router {
    let client_service = ServeDir::new("client")
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new("client/404.html"));

    Router::new()
        .nest("/api", api::routes())
        .fallback_service(client_service)
}
