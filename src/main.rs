mod stream;
mod ws;

use std::sync::Arc;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use shuttle_axum::{
    axum::{
        extract::ws::Message,
        routing::get,
        Extension, Json, Router, response::IntoResponse,
    },
    ShuttleAxum,
};
use serde::Serialize;
use tokio::sync::{watch, Mutex};
use tower_http::services::ServeDir;

use stream::spawn_binance_feed;
use ws::{websocket_handler, State};
use crate::VERSION;

#[derive(Serialize)]
struct VersionResponse<'a> {
    version: &'a str,
}

async fn version_handler() -> impl IntoResponse {
    Json(VersionResponse { version: VERSION })
}

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    // Initialize tracing only if a global subscriber hasn't already been set.
    // Shuttle's runtime installs one when running locally, so we use
    // `try_init` to avoid panicking with `SetGlobalDefaultError`.
    // Log to both stdout and a rolling log file under `logs/server.log`.
    let file_appender = tracing_appender::rolling::daily("logs", "server.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Prevent the writer from being dropped prematurely.
    Box::leak(Box::new(guard));

    let registry = tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_writer(std::io::stdout))
        .with(fmt::layer().with_target(false).with_writer(file_writer));

    let _ = registry
        .try_init();

    let (tx, rx) = watch::channel(Message::Text("{}".into()));
    tokio::spawn(spawn_binance_feed(tx));

    let state = Arc::new(Mutex::new(State {
        clients_count: 0,
        rx,
    }));

    let router = Router::new()
        .route("/version", get(version_handler))
        .route("/websocket", get(websocket_handler))
        .nest_service("/", ServeDir::new("static"))
        .layer(Extension(state));

    Ok(router.into())
}
