//! Pull Raydium liquidity JSON and print the top-N coins by 24 h volume.
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const LIQUIDITY_URL: &str = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
const TOKEN_URL: &str = "https://api.raydium.io/v2/sdk/token/solana.mainnet.json";

#[derive(Debug, Deserialize)]
struct LiquidityFile {
    official: Vec<PoolInfo>,
    #[serde(rename = "unOfficial")]
    unofficial: Vec<PoolInfo>,
}

#[derive(Debug, Deserialize)]
struct PoolInfo {
    #[serde(rename = "baseSymbol")]
    base_symbol: String,
    #[serde(rename = "quoteSymbol")]
    quote_symbol: String,
    #[serde(rename = "volume24h", default)]
    volume24h: f64,
}

#[derive(Debug, Deserialize)]
struct TokenFile {
    official: Vec<TokenInfo>,
    #[serde(rename = "unOfficial")]
    unofficial: Vec<TokenInfo>,
}

#[derive(Debug, Deserialize)]
struct TokenInfo {
    #[serde(rename = "symbol")]
    symbol: String,
    #[serde(rename = "name")]
    _name: Option<String>,
    #[serde(rename = "decimals")]
    _decimals: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .user_agent("raydium-top-coins/0.1 (github.com/crypto-scanner-agent)")
        .build()?;

    let (liq_res, token_res) = tokio::try_join!(
        client.get(LIQUIDITY_URL).send(),
        client.get(TOKEN_URL).send()
    )?;

    let liquidity: LiquidityFile = liq_res.json().await?;
    let token_file: TokenFile = token_res.json().await?;

    let listed_tokens: HashSet<String> = token_file
        .official
        .into_iter()
        .chain(token_file.unofficial)
        .map(|t| t.symbol)
        .collect();

    let mut volume: HashMap<String, f64> = HashMap::new();
    let all_pools = liquidity.official.into_iter().chain(liquidity.unofficial);
    for pool in all_pools {
        if !listed_tokens.contains(&pool.base_symbol) || !listed_tokens.contains(&pool.quote_symbol) {
            continue;
        }
        *volume.entry(pool.base_symbol).or_default() += pool.volume24h;
        *volume.entry(pool.quote_symbol).or_default() += pool.volume24h;
    }

    let mut ranking: Vec<_> = volume.into_iter().collect();
    ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 10 Raydium coins by 24 h volume:");
    for (rank, (sym, vol)) in ranking.into_iter().take(10).enumerate() {
        println!("{:>2}. {:<10} ${:.2} M", rank + 1, sym, vol / 1_000_000.0);
    }

    Ok(())
}

