mod api;
mod routes;
mod utils;

use crate::routes::root::router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

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

    let app = router().layer(CorsLayer::permissive()).layer(
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
