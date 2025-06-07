use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Fetch balances for a Solana account.
///
/// The returned vector includes the SOL balance (lamports) and any SPL token
/// balances with a positive amount. Zero-balance tokens are filtered out except
/// for SOL.
pub async fn fetch_balances(owner: &str, rpc_url: &str) -> Result<Vec<(String, u64)>> {
    let client = Client::new();

    let sol_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [owner],
    });
    let sol_resp: Value = client
        .post(rpc_url)
        .json(&sol_req)
        .send()
        .await?
        .json()
        .await?;
    let sol_lamports = sol_resp
        .get("result")
        .and_then(|r| r.get("value"))
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("invalid getBalance response"))?;
    let mut balances = vec![("SOL".to_owned(), sol_lamports)];

    let tok_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            owner,
            { "programId": TOKEN_PROGRAM_ID },
            { "encoding": "jsonParsed" }
        ]
    });
    let tok_resp: Value = client
        .post(rpc_url)
        .json(&tok_req)
        .send()
        .await?
        .json()
        .await?;
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
                        balances.push((mint.to_owned(), amount));
                    }
                }
            }
        }
    }

    balances.retain(|(mint, amt)| *amt > 0 || mint == "SOL");
    Ok(balances)
}
