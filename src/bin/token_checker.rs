use rig::providers::deepseek::{self, Client};
use futures::{stream, StreamExt};
use anyhow::Result;
use std::env;

/// Response structure describing token status.
#[derive(serde::Deserialize, serde::Serialize)]
struct TokenReview {
    /// Short comment about the token.
    comment: String,
}

/// Check a single token symbol using a DeepSeek agent.
async fn check_token(client: &Client, token: &str) -> Result<String> {
    let agent = client
        .extractor::<TokenReview>("gpt-4")
        .preamble(
            "You are a cryptocurrency expert. For the provided token symbol, \n             state whether it appears legitimate or suspicious in one short sentence.",
        )
        .build();

    let prompt = format!("Token: {token}");
    let review = agent.extract(&prompt).await?;
    Ok(review.comment)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let client = Client::from_env();
    let tokens: Vec<String> = env::args().skip(1).collect();
    if tokens.is_empty() {
        eprintln!("usage: token_checker SYMBOL [SYMBOL...]" );
        std::process::exit(1);
    }

    let results = stream::iter(tokens.iter())
        .map(|t| check_token(&client, t))
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;

    for (token, res) in tokens.iter().zip(results) {
        match res {
            Ok(comment) => println!("{token}: {comment}"),
            Err(e) => eprintln!("{token}: error - {e}"),
        }
    }

    Ok(())
}

