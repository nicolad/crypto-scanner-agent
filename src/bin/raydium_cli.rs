use anyhow::{anyhow, Result};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

const INFO_URL: &str = "https://api-v3.raydium.io/main/info";
const PRICE_URL: &str = "https://api-v3.raydium.io/mint/price";
const MINT_LIST_URL: &str = "https://api-v3.raydium.io/mint/list";
const POOLS_URL: &str = "https://api-v3.raydium.io/pools/info/list?poolType=all&poolSortField=default&sortType=desc&pageSize=10&page=1";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

enum Command {
    ListPools,
    Balances { owner: String, rpc: String },
    Info,
    Price { mint: String },
    Mints,
}

fn parse_args() -> Result<Command> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err(anyhow!("no command provided"));
    }
    match args.remove(0).as_str() {
        "list-pools" => Ok(Command::ListPools),
        "balances" => {
            if args.is_empty() {
                return Err(anyhow!("balances requires owner"));
            }
            let owner = args.remove(0);
            let mut rpc = "https://api.mainnet-beta.solana.com".to_string();
            if !args.is_empty() && args[0].starts_with("--rpc=") {
                rpc = args.remove(0)[6..].to_string();
            }
            Ok(Command::Balances { owner, rpc })
        }
        "info" => Ok(Command::Info),
        "price" => {
            if args.is_empty() {
                return Err(anyhow!("price requires mint"));
            }
            Ok(Command::Price { mint: args.remove(0) })
        }
        "mints" => Ok(Command::Mints),
        _ => Err(anyhow!("unknown command")),
    }
}

#[derive(Deserialize)]
struct MainInfoOuter {
    success: bool,
    data: MainInfoData,
}

#[derive(Deserialize)]
struct MainInfoData {
    tvl: f64,
    #[serde(alias = "volume24")]
    volume_24: f64,
}

async fn fetch_main_info(client: &Client) -> Result<MainInfoData> {
    let outer: MainInfoOuter = client.get(INFO_URL).send().await?.json().await?;
    if !outer.success {
        Err(anyhow!("Raydium API returned success=false for /main/info"))
    } else {
        Ok(outer.data)
    }
}

#[derive(Deserialize)]
struct PriceOuter {
    success: bool,
    data: HashMap<String, f64>,
}

async fn fetch_price(client: &Client, ids: &[&str]) -> Result<HashMap<String, f64>> {
    let url = Url::parse_with_params(PRICE_URL, &[("ids", ids.join(","))])?;
    let outer: PriceOuter = client.get(url).send().await?.json().await?;
    if !outer.success {
        Err(anyhow!("Raydium API returned success=false for /mint/price"))
    } else {
        Ok(outer.data)
    }
}

#[derive(Deserialize)]
struct MintListOuter {
    success: bool,
    data: MintListData,
}

#[derive(Deserialize)]
struct MintListData {
    #[serde(rename = "mintList")]
    mint_list: Vec<MintItem>,
}

#[derive(Deserialize)]
struct MintItem {
    #[serde(rename = "address")]
    mint: String,
    symbol: String,
    name: String,
    decimals: u8,
}

async fn fetch_mints(client: &Client) -> Result<Vec<MintItem>> {
    let outer: MintListOuter = client.get(MINT_LIST_URL).send().await?.json().await?;
    if !outer.success {
        Err(anyhow!("Raydium API returned success=false for /mint/list"))
    } else {
        Ok(outer.data.mint_list)
    }
}

#[derive(Deserialize, Clone)]
struct TokenSide {
    #[serde(alias = "mint", alias = "address")]
    mint: Option<String>,
}

#[derive(Deserialize, Clone)]
struct RawPool {
    id: Option<String>,
    #[serde(alias = "base_mint", alias = "baseMint")]
    base_mint: Option<String>,
    #[serde(alias = "quote_mint", alias = "quoteMint")]
    quote_mint: Option<String>,
    base: Option<TokenSide>,
    quote: Option<TokenSide>,
    #[serde(alias = "mintA")]
    mint_a: Option<TokenSide>,
    #[serde(alias = "mintB")]
    mint_b: Option<TokenSide>,
    #[serde(alias = "fee_bps", alias = "feeBps")]
    fee_bps: Option<u32>,
    #[serde(alias = "feeRate")]
    fee_rate: Option<f64>,
}

#[derive(Debug)]
struct Pool {
    id: String,
    token0: String,
    token1: String,
    fee_bps: u32,
}

fn raw_to_pool(raw: &RawPool) -> Option<Pool> {
    let id = raw.id.as_ref()?;

    let mint0 = raw
        .base_mint
        .clone()
        .or_else(|| raw.base.as_ref().and_then(|t| t.mint.clone()))
        .or_else(|| raw.mint_a.as_ref().and_then(|t| t.mint.clone()))?;
    let mint1 = raw
        .quote_mint
        .clone()
        .or_else(|| raw.quote.as_ref().and_then(|t| t.mint.clone()))
        .or_else(|| raw.mint_b.as_ref().and_then(|t| t.mint.clone()))?;
    let fee_bps = raw
        .fee_bps
        .or_else(|| raw.fee_rate.map(|r| (r * 10_000.0).round() as u32))?;

    Some(Pool {
        id: id.to_owned(),
        token0: mint0,
        token1: mint1,
        fee_bps,
    })
}

async fn fetch_pools(client: &Client) -> Result<Vec<Pool>> {
    let url = Url::parse(POOLS_URL)?;
    let body: Value = client.get(url).send().await?.json().await?;

    fn extract_lists(v: &Value) -> Vec<Value> {
        if v.is_array() {
            v.as_array().cloned().unwrap_or_default()
        } else if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
            arr.to_vec()
        } else if let Some(obj) = v.get("data") {
            if let Some(arr) = obj.get("data").and_then(|d| d.as_array()) {
                arr.to_vec()
            } else if let Some(arr) = obj.get("lists").and_then(|l| l.as_array()) {
                arr.to_vec()
            } else {
                let mut out = vec![];
                if let Some(arr) = obj.get("official").and_then(|l| l.as_array()) {
                    out.extend(arr.to_owned());
                }
                if let Some(arr) = obj.get("unOfficial").and_then(|l| l.as_array()) {
                    out.extend(arr.to_owned());
                }
                out
            }
        } else {
            vec![]
        }
    }

    let pools_json = extract_lists(&body);
    if pools_json.is_empty() {
        return Err(anyhow!("Raydium API: no pool list found in response"));
    }
    let mut out = Vec::with_capacity(pools_json.len());
    for item in pools_json {
        if let Ok(raw) = serde_json::from_value::<RawPool>(item) {
            if let Some(pool) = raw_to_pool(&raw) {
                out.push(pool);
            }
        }
    }
    Ok(out)
}

async fn fetch_balances(owner: &str, rpc_url: &str) -> Result<Vec<(String, u64)>> {
    let client = Client::new();

    let sol_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [owner],
    });
    let sol_resp: Value = client.post(rpc_url).json(&sol_req).send().await?.json().await?;
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
    let tok_resp: Value = client.post(rpc_url).json(&tok_req).send().await?.json().await?;
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

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = parse_args()?;
    let http = Client::new();

    match cmd {
        Command::ListPools => {
            for pool in fetch_pools(&http).await? {
                println!("{:<20} {}→{} (fee {} bps)", pool.id, pool.token0, pool.token1, pool.fee_bps);
            }
        }
        Command::Balances { owner, rpc } => {
            for (mint, amount) in fetch_balances(&owner, &rpc).await? {
                println!("{mint}: {amount}");
            }
        }
        Command::Info => {
            let info = fetch_main_info(&http).await?;
            println!(
                "Raydium TVL  : ${:.2} M\nRaydium 24 h : ${:.2} M",
                info.tvl / 1_000_000.0,
                info.volume_24 / 1_000_000.0
            );
        }
        Command::Price { mint } => {
            let ids: Vec<&str> = mint.split(',').collect();
            let prices = fetch_price(&http, &ids).await?;
            for id in ids {
                match prices.get(id) {
                    Some(p) => println!("{id}  ${:.6}", p),
                    None => println!("{id}  (price unavailable)"),
                }
            }
        }
        Command::Mints => {
            let tokens = fetch_mints(&http).await?;
            if tokens.is_empty() {
                println!("(no mints found)");
            } else {
                for t in tokens {
                    println!("{:<44} {:<10} {:<3} {}", t.mint, t.symbol, t.decimals, t.name);
                }
            }
        }
    }

    Ok(())
}
