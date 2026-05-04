//! Live smoke test for wallet-to-wallet transfers.
//!
//! Usage:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!   SHURIKEN_FROM_WALLET=cmoxxx... \
//!   SHURIKEN_TO_WALLET=cmoxxx... \
//!     cargo run --example transfers_smoke
//!
//! Sends a tiny SVM amount with `await_result: false` so the call returns
//! quickly with a task_id. Requires `transfer:write` scope.

use shuriken_sdk::transfers::SendBody;
use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY required");
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());
    let from_wallet = std::env::var("SHURIKEN_FROM_WALLET").expect("SHURIKEN_FROM_WALLET required");
    let to_wallet = std::env::var("SHURIKEN_TO_WALLET").expect("SHURIKEN_TO_WALLET required");

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    let body = SendBody {
        from_wallet_id: from_wallet,
        to_wallet_id: to_wallet,
        token: "native".to_string(),
        amount: "1000".to_string(), // 1000 lamports — tiny test amount
        chain: "SVM".to_string(),
        chain_id: None,
        await_result: Some(false),
        correlation_id: None,
        agent_comment: Some("transfers_smoke example".to_string()),
    };

    println!("Submitting transfer (await_result: false)...");
    let result = client.transfers().send(&body).await?;
    println!(
        "  task_id={}  status={}  will_archive_on_success={}",
        result.task_id, result.status, result.will_archive_on_success
    );

    Ok(())
}
