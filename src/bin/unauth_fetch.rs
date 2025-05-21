use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use reqwest::{Client, Method};
use std::sync::Arc;
use tokio_stream::wrappers::LinesStream;

/// How many requests we want in-flight at once.
const CONCURRENCY: usize = 16;

#[tokio::main]
async fn main() -> Result<()> {
    // ---------------------------------------------------------------------
    // 1.  Shared, clonable state ------------------------------------------------
    // ---------------------------------------------------------------------
    let base = Arc::new("https://api.exchange.example".to_owned());
    let client = Arc::new(Client::builder().build()?);

    // ---------------------------------------------------------------------
    // 2.  Produce a stream of `(method, path)` pairs ----------------------------
    // ---------------------------------------------------------------------
    // (**replace this with your real CSV / file parsing**)
    let lines = std::fs::read_to_string("unauth_requests.csv")?;
    let stream = LinesStream::new(lines.lines().map(str::to_owned).collect::<Vec<_>>().into_iter());

    // ---------------------------------------------------------------------
    // 3.  Fan-out the work  ------------------------------------------------
    // ---------------------------------------------------------------------
    stream
        .map(|line| {
            // Each invocation of this closure happens sequentially,
            // so we grab cheap clones *outside* the async block …
            let base = Arc::clone(&base);
            let client = Arc::clone(&client);

            async move {
                // … and move them *into* the future.
                let (method_raw, path) = parse_line(&line)?;
                let url = format!("{}{}", base, path);
                let method = method_raw.parse::<Method>()?;

                let t0 = std::time::Instant::now();
                let status = client.request(method, &url).send().await?.status();
                println!("{:>4} – {} ({:?})", status.as_u16(), url, t0.elapsed());

                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(CONCURRENCY)
        .try_collect::<()>()
        .await?;

    Ok(())
}

/// Very small helper – adapt to your CSV format.
fn parse_line(line: &str) -> Result<(String, String)> {
    let mut parts = line.splitn(2, ',');
    let method = parts.next().unwrap_or("").trim().to_owned();
    let path = parts.next().unwrap_or("").trim().to_owned();
    Ok((method, path))
}
