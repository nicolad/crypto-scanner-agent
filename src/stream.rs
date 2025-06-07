use std::{error::Error, time::Duration};

use shuttle_axum::axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use futures::{StreamExt, SinkExt};
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

/// Parse incoming JSON text into a list of [`Signal`]s.
///
/// The function filters entries where the 24h percentage gain is below 5% or
/// the quote volume is below $1M. Any valid signals are returned for further
/// processing or broadcasting.
fn extract_signals_from_text(txt: &str) -> Result<Vec<Signal>, Box<dyn Error + Send + Sync>> {
    let parsed: serde_json::Value = serde_json::from_str(txt)?;
    let mut signals = Vec::new();

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
                signals.push(sig);
            }
        }
    }

    Ok(signals)
}

/// Connect to the Raydium WebSocket feed and forward any valid signals to
/// connected WebSocket clients via the provided watch channel.
pub async fn spawn_raydium_feed(tx: watch::Sender<Message>) {
    // Default Raydium public feed. Can be overridden by the RAYDIUM_WS_URL
    // environment variable if needed.
    let url = std::env::var("RAYDIUM_WS_URL")
        .unwrap_or_else(|_| "wss://api.raydium.io/ws".to_string());
    loop {
        match connect_async(url).await {
            Ok((ws, _)) => {
                tracing::info!("\u{1f7e2} Connected to Raydium stream");
                if let Err(e) = handle_socket(ws, &tx).await {
                    tracing::warn!("Raydium WS error: {:?}", e);
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
    let (mut sink, mut stream) = ws.split();
    while let Some(Ok(frame)) = stream.next().await {
        match frame {
            tungstenite::Message::Text(txt) => {
                for sig in extract_signals_from_text(&txt)? {
                    let json = serde_json::to_string(&sig)?;
                    let _ = tx.send(Message::Text(json));
                }
            }
            tungstenite::Message::Ping(payload) => {
                // Echo the ping payload back as recommended by the Raydium docs
                sink.send(tungstenite::Message::Pong(payload)).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_signals_basic_filtering() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "5.5",
                "q": "1500000",
                "c": "30000"
            },
            {
                "s": "ETHUSDT",
                "P": "2.0",
                "q": "900000",
                "c": "2000"
            }
        ]"#;

        let signals = extract_signals_from_text(json).unwrap();
        assert_eq!(signals.len(), 1);
        let sig = &signals[0];
        assert_eq!(sig.symbol, "BTCUSDT");
        assert!((sig.pct_gain_24h - 5.5).abs() < f64::EPSILON);
        assert!((sig.quote_vol_usdt - 1_500_000.0).abs() < f64::EPSILON);
        assert!((sig.last_price - 30000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_signals_invalid_json() {
        let json = "{ invalid json }";
        assert!(extract_signals_from_text(json).is_err());
    }

    #[tokio::test]
    async fn test_extract_signals_multiple_valid_entries() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "8.0",
                "q": "2000000",
                "c": "30000"
            },
            {
                "s": "ETHUSDT",
                "P": "5.0",
                "q": "1500000",
                "c": "2000"
            }
        ]"#;

        let signals = extract_signals_from_text(json).unwrap();
        assert_eq!(signals.len(), 2);
    }

    #[test]
    fn test_extract_signals_non_numeric_fields_error() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "five",
                "q": "1500000",
                "c": "30000"
            }
        ]"#;

        assert!(extract_signals_from_text(json).is_err());
    }

    #[test]
    fn test_extract_signals_numeric_values_not_strings() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": 10,
                "q": 2000000,
                "c": 30000
            }
        ]"#;

        let signals = extract_signals_from_text(json).unwrap();
        assert!(signals.is_empty());
    }

    #[test]
    fn test_extract_signals_empty_array() {
        let json = "[]";
        let signals = extract_signals_from_text(json).unwrap();
        assert!(signals.is_empty());
    }

    #[test]
    fn test_extract_signals_non_array_json_returns_empty() {
        let json = "{}";
        let signals = extract_signals_from_text(json).unwrap();
        assert!(signals.is_empty());
    }

    #[test]
    fn test_extract_signals_exact_threshold() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "5.0",
                "q": "1000000",
                "c": "100"
            }
        ]"#;

        let signals = extract_signals_from_text(json).unwrap();
        assert_eq!(signals.len(), 1);
        let sig = &signals[0];
        assert_eq!(sig.symbol, "BTCUSDT");
        assert!((sig.pct_gain_24h - 5.0).abs() < f64::EPSILON);
        assert!((sig.quote_vol_usdt - 1_000_000.0).abs() < f64::EPSILON);
        assert!((sig.last_price - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_signals_negative_values_filtered() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "-10",
                "q": "2000000",
                "c": "30000"
            }
        ]"#;

        let signals = extract_signals_from_text(json).unwrap();
        assert!(signals.is_empty());
    }

    #[test]
    fn test_extract_signals_malformed_number_error() {
        let json = r#"[
            {
                "s": "BTCUSDT",
                "P": "5.0",
                "q": "1_000_000",
                "c": "30000"
            }
        ]"#;

        assert!(extract_signals_from_text(json).is_err());
    }
}
