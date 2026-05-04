//! Live smoke test for the wallets archive lifecycle.
//!
//! Usage:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!   SHURIKEN_WALLET_ID=cmoxxx... \
//!     cargo run --example wallets_smoke
//!
//! Archives the wallet, then restores it. Requires `write:wallets` scope.

use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY required");
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());
    let wallet_id = std::env::var("SHURIKEN_WALLET_ID").expect("SHURIKEN_WALLET_ID required");

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    println!("Archiving wallet {wallet_id}...");
    let archived = client.wallets().archive(&wallet_id).await?;
    println!(
        "  state={}  cleared_default={}",
        archived.wallet.state, archived.cleared_default
    );

    println!("Unarchiving wallet {wallet_id}...");
    let restored = client.wallets().unarchive(&wallet_id).await?;
    println!("  state={}", restored.wallet.state);

    Ok(())
}
