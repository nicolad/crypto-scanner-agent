use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{debug, error, info, instrument};

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Fetch balances for a Solana account.
///
/// * Returns the SOL balance (lamports) **plus** every SPL-token balance > 0.
/// * Zero-balance tokens are filtered out (except SOL, which is always kept).
#[instrument(name = "solana::fetch_balances", skip(rpc_url))]
pub async fn fetch_balances(owner: &str, rpc_url: &str) -> Result<Vec<(String, u64)>> {
    info!(%owner, "Fetching Solana balances");

    let client = Client::new();

    /* ------------------------------------------------------------------ SOL */

    let sol_req = json!({
        "jsonrpc": "2.0",
        "id":      1,
        "method":  "getBalance",
        "params":  [owner],
    });
    debug!("getBalance request  ➜  {sol_req}");
    let sol_resp: Value = client
        .post(rpc_url)
        .json(&sol_req)
        .send()
        .await?
        .json()
        .await?;
    debug!("getBalance response ➜  {sol_resp}");

    let sol_lamports = sol_resp
        .get("result")
        .and_then(|r| r.get("value"))
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            error!("Invalid getBalance response: {sol_resp}");
            anyhow!("invalid getBalance response")
        })?;

    let mut balances = vec![("SOL".to_owned(), sol_lamports)];

    /* ------------------------------------------------------------- SPL tokens */

    let tok_req = json!({
        "jsonrpc": "2.0",
        "id":      1,
        "method":  "getTokenAccountsByOwner",
        "params": [
            owner,
            { "programId": TOKEN_PROGRAM_ID },
            { "encoding": "jsonParsed" }
        ]
    });
    debug!("getTokenAccountsByOwner request  ➜  {tok_req}");
    let tok_resp: Value = client
        .post(rpc_url)
        .json(&tok_req)
        .send()
        .await?
        .json()
        .await?;
    debug!("getTokenAccountsByOwner response ➜  {tok_resp}");

    if let Some(arr) = tok_resp
        .get("result")
        .and_then(|r| r.get("value"))
        .and_then(Value::as_array)
    {
        for acc in arr {
            if let Some(info) = acc
                .get("account")
                .and_then(|a| a.get("data"))
                .and_then(|d| d.get("parsed"))
                .and_then(|p| p.get("info"))
            {
                if let (Some(mint), Some(amount_str)) = (
                    info.get("mint").and_then(Value::as_str),
                    info.get("tokenAmount")
                        .and_then(|ta| ta.get("amount"))
                        .and_then(Value::as_str),
                ) {
                    if let Ok(amount) = amount_str.parse::<u64>() {
                        debug!(%mint, amount, "Parsed SPL-token balance");
                        balances.push((mint.to_owned(), amount));
                    }
                }
            }
        }
    }

    /* ------------------------------------------------------- final filtering */

    let before = balances.len();
    balances.retain(|(mint, amt)| *amt > 0 || mint == "SOL");
    let after = balances.len();

    info!(
        owner,
        total = after,
        filtered = before - after,
        "Balance fetch complete"
    );
    Ok(balances)
}
