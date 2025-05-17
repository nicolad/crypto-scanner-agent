use std::{sync::Arc, time::Duration};

use axum::{
    extract::{ws::{Message, WebSocket}, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json;
use shuttle_axum::ShuttleAxum;
use tokio::sync::{watch, Mutex};
use tokio_tungstenite::{connect_async, tungstenite};
use tower_http::services::ServeDir;
use std::error::Error;
use tracing_subscriber;

#[derive(Serialize, Clone)]
struct Signal {
    symbol: String,
    pct_gain_24h: f64,
    quote_vol_usdt: f64,
    last_price: f64,
    ts: DateTime<Utc>,
}

struct State {
    clients_count: usize,
    rx: watch::Receiver<Message>,
}

async fn spawn_binance_feed(tx: watch::Sender<Message>) {
    let url = "wss://stream.binance.com:9443/ws/!ticker@arr";
    loop {
        match connect_async(url).await {
            Ok((ws, _)) => {
                tracing::info!("\u{1f7e2} Connected to Binance stream");
                if let Err(e) = handle_socket(ws, &tx).await {
                    tracing::warn!("Binance WS error: {:?}", e);
                }
            }
            Err(e) => tracing::error!("WS connect failed: {:?}", e),
        }
        for delay in [2u64, 4, 8, 16] {
            tracing::info!("Reconnect in {}s", delay);
            tokio::time::sleep(Duration::from_secs(delay)).await;
            if connect_async(url).await.is_ok() {
                break;
            }
        }
    }
}

async fn handle_socket<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    tx: &watch::Sender<Message>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (_sink, mut stream) = ws.split();
    while let Some(Ok(frame)) = stream.next().await {
        if let tungstenite::Message::Text(txt) = frame {
            let parsed: serde_json::Value = serde_json::from_str(&txt)?;
            if let Some(arr) = parsed.as_array() {
                for obj in arr {
                    let pct: f64 = obj["P"].as_str().unwrap_or("0").parse()?;
                    let vol: f64 = obj["q"].as_str().unwrap_or("0").parse()?;
                    if pct >= 5.0 && vol >= 1_000_000.0 {
                        let sig = Signal {
                            symbol: obj["s"].as_str().unwrap().to_owned(),
                            pct_gain_24h: pct,
                            quote_vol_usdt: vol,
                            last_price: obj["c"].as_str().unwrap_or("0").parse()?,
                            ts: Utc::now(),
                        };
                        let json = serde_json::to_string(&sig)?;
                        let _ = tx.send(Message::Text(json));
                    }
                }
            }
        }
    }
    Ok(())
}

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let (tx, rx) = watch::channel(Message::Text("{}".into()));
    tokio::spawn(spawn_binance_feed(tx.clone()));

    let state = Arc::new(Mutex::new(State { clients_count: 0, rx }));

    let router = Router::new()
        .route("/websocket", get(websocket_handler))
        .nest_service("/", ServeDir::new("static"))
        .layer(Extension(state));

    Ok(router.into())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<Mutex<State>>) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = {
        let mut state = state.lock().await;
        state.clients_count += 1;
        state.rx.clone()
    };

    let mut send_task = tokio::spawn(async move {
        while let Ok(()) = rx.changed().await {
            let msg = rx.borrow().clone();

            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {}
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    state.lock().await.clients_count -= 1;
}
