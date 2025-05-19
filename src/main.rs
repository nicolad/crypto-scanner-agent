mod stream;
mod ws;

use std::sync::Arc;

use axum::{extract::ws::Message, routing::get, Extension, Router};
use shuttle_axum::ShuttleAxum;
use tokio::sync::{watch, Mutex};
use tower_http::services::ServeDir;

use stream::spawn_binance_feed;
use ws::{websocket_handler, State};

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let (tx, rx) = watch::channel(Message::Text("{}".into()));
    tokio::spawn(spawn_binance_feed(tx));

    let state = Arc::new(Mutex::new(State { clients_count: 0, rx }));

    let router = Router::new()
        .route("/websocket", get(websocket_handler))
        .nest_service("/", ServeDir::new("static"))
        .layer(Extension(state));

    Ok(router.into())
}
