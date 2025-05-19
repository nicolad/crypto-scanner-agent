use std::{error::Error, time::Duration};

use axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::watch;
use tokio_tungstenite::{connect_async, tungstenite};

#[derive(Serialize, Clone)]
pub struct Signal {
    pub symbol: String,
    pub pct_gain_24h: f64,
    pub quote_vol_usdt: f64,
    pub last_price: f64,
    pub ts: DateTime<Utc>,
}

pub async fn spawn_binance_feed(tx: watch::Sender<Message>) {
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

