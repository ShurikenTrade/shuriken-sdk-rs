//! Live smoke test for agent-posted trade suggestions.
//!
//! Usage:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!     cargo run --example suggestions_smoke
//!
//! Lists OPEN suggestions for the authenticated agent key. Requires the
//! `read:suggestions` scope.

use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY required");
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    println!("Listing OPEN suggestions at {base_url}...");
    let list = client.suggestions().list(None).await?;
    println!("  {} suggestion(s) returned", list.suggestions.len());
    for s in &list.suggestions {
        println!(
            "  - {id}  {state:?}  {side:?} {symbol} on {net}  rationale: {rationale}",
            id = s.id,
            state = s.state,
            side = s.side,
            symbol = s.asset.symbol,
            net = s.network_id,
            rationale = s.rationale,
        );
    }
    if let Some(cursor) = &list.next_cursor {
        println!("  next_cursor: {cursor}");
    }

    Ok(())
}
