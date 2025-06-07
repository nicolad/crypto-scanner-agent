pub mod util {
    /// Returns the number of logical CPU cores available on the system.
    pub fn cpu_core_count() -> usize {
        num_cpus::get()
    }

    /// Returns the maximum number of threads that can run in parallel.
    pub fn max_parallel_threads() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

/// The version of the `crypto-scanner-agent` library. This is populated at
/// compile time using the `CARGO_PKG_VERSION` environment variable.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod solana;

mod stream;
mod ws;

use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use serde::Serialize;
use tokio::sync::{watch, Mutex};
use tower_http::services::ServeDir;
use shuttle_axum::{
    axum::{
        extract::ws::Message,
        routing::get,
        Extension, Json, Router, response::IntoResponse,
    },
    ShuttleAxum,
};

use stream::spawn_raydium_feed;
use ws::{websocket_handler, State};

#[derive(Serialize)]
struct VersionResponse<'a> {
    version: &'a str,
}

async fn version_handler() -> impl IntoResponse {
    Json(VersionResponse { version: VERSION })
}

#[shuttle_runtime::main]
pub async fn main() -> ShuttleAxum {
    let file_appender = tracing_appender::rolling::daily("logs", "server.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    Box::leak(Box::new(guard));

    let registry = tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_writer(std::io::stdout))
        .with(fmt::layer().with_target(false).with_writer(file_writer));

    let _ = registry.try_init();

    let (tx, rx) = watch::channel(Message::Text("{}".into()));
    tokio::spawn(spawn_raydium_feed(tx));

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

