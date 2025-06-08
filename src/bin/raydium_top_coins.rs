//! Query Raydium V3 pools, keep the N highest-volume ones,
//! print a table **and** save them to a local JSON file.
//
//! Build:  cargo run --bin raydium_top_coins --release
//! Logs :  RUST_LOG=raydium_cli=debug cargo run …

use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::File, io::Write, path::Path, time::Instant};
use tracing::{debug, error, info, instrument};

/* ─────────────────────────── Types ─────────────────────────── */

/// Outer status wrapper used by every Raydium V3 call.
#[derive(Debug, Deserialize)]
struct ApiWrapper {
    success: bool,
    #[serde(default)]
    msg: Option<String>,
    data: Value, // shape varies → handle at runtime
}

/// Pool row – keep only the bits we care about.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RaydiumPool {
    /// Pair name, e.g. `"SOL/USDC"`.  
    /// Some rows sadly omit it, so we supply an empty string instead of
    /// aborting the whole deserialisation.
    #[serde(default)]
    name: String,

    price: Option<f64>, // mid-price
    volume24h: Option<f64>,
}

/* ─────────────────────────── Constants ─────────────────────── */

const ENDPOINT: &str = "https://api-v3.raydium.io/pools/info/list";
const LIMIT: usize = 50; // top-N in table / JSON
const JSON_OUT: &str = "raydium_top_pools.json";

/* ─────────────────────────── Main ──────────────────────────── */

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let t0 = Instant::now();
    info!("Querying Raydium V3 pools…");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("building HTTP client")?;

    let raw = fetch_raw(&client)?;
    let mut pools = parse_json(&raw)?;

    // sort & trim
    pools.sort_by(|a, b| {
        b.volume24h
            .partial_cmp(&a.volume24h)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pools.truncate(LIMIT);

    save_json(&pools)?;
    print_table(&pools);
    info!("Done in {:.2?}  →  {}", t0.elapsed(), JSON_OUT);
    Ok(())
}

/* ───────────────────────── HTTP ────────────────────────────── */

#[instrument(skip(client))]
fn fetch_raw(client: &Client) -> Result<String> {
    // required query params – leaving them out returns 500
    let qs = [
        ("poolType", "all"),
        ("poolSortField", "volume24h"),
        ("sortType", "desc"),
        ("pageSize", &LIMIT.to_string()),
        ("page", "1"),
    ];

    let body = client
        .get(ENDPOINT)
        .query(&qs)
        .header("accept", "application/json")
        .send()
        .context("sending GET")?
        .error_for_status()
        .context("HTTP error")?
        .text()
        .context("reading body")?;

    debug!(bytes = body.len(), "downloaded body");
    Ok(body)
}

/* ─────────────────────── JSON parsing ──────────────────────── */

#[instrument(level = "debug", skip(raw))]
fn parse_json(raw: &str) -> Result<Vec<RaydiumPool>> {
    let wrapper: ApiWrapper =
        serde_json::from_str(raw).map_err(|e| slice_err(raw, &e, "wrapper parse failed"))?;

    if !wrapper.success {
        bail!(wrapper
            .msg
            .unwrap_or_else(|| "Raydium signalled failure".into()));
    }

    // data = […] | { list:[…] } | { count:n , data:[…] }
    let arr = if let Some(a) = wrapper.data.as_array() {
        a.clone()
    } else if wrapper.data.get("list").is_some() {
        wrapper.data["list"]
            .as_array()
            .ok_or_else(|| anyhow!("‘list’ is not an array"))?
            .clone()
    } else if wrapper.data.get("data").is_some() {
        wrapper.data["data"]
            .as_array()
            .ok_or_else(|| anyhow!("‘data’ is not an array"))?
            .clone()
    } else {
        bail!("unrecognised payload shape: {}", wrapper.data);
    };

    serde_json::from_value::<Vec<RaydiumPool>>(Value::Array(arr))
        .map_err(|e| slice_err(raw, &e, "pool array parse failed"))
}

/* ──────────────────── JSON file output ─────────────────────── */

fn save_json(pools: &[RaydiumPool]) -> Result<()> {
    let path = Path::new(JSON_OUT);
    let mut file = File::create(path).context("creating JSON output file")?;
    serde_json::to_writer_pretty(&mut file, pools).context("serialising pretty JSON")?;
    file.write_all(b"\n").ok(); // final newline – cosmetics
    Ok(())
}

/* ───────────────────────── Helpers ─────────────────────────── */

fn slice_err(raw: &str, err: &impl std::fmt::Display, ctx: &str) -> anyhow::Error {
    // Safe 200-byte snippet around the byte offset (serde_json ¹⁰² → .column()).
    let pos = err
        .to_string()
        .split(" at line ")
        .last()
        .and_then(|s| s.split(" column ").nth(1)?.parse::<usize>().ok())
        .unwrap_or(0);

    let start = pos.saturating_sub(100);
    let end = (pos + 100).min(raw.len());
    error!(%err, snippet = &raw[start..end], ctx);
    // `.context()` needs a `'static` str; own the string first.
    anyhow!(err.to_string()).context(ctx.to_owned())
}

fn print_table(pools: &[RaydiumPool]) {
    println!("{:<22} | {:>13} | {}", "POOL", "PRICE", "VOL 24H");
    println!("{}", "-".repeat(60));
    for p in pools {
        println!(
            "{:<22} | {:>13.6} | {}",
            p.name,
            p.price.unwrap_or_default(),
            p.volume24h
                .map(|v| format!("{:.0}", v))
                .unwrap_or_else(|| "-".into())
        );
    }
}
