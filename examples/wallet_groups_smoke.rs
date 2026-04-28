//! Live smoke test for the wallet-groups namespace against a staging deploy.
//!
//! Run with:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!     cargo run --example wallet_groups_smoke

use shuriken_sdk::wallet_groups::{CreateWalletGroupBody, UpdateWalletGroupBody};
use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY")?;
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    println!("=== Rust SDK staging smoke ===\n");

    let groups = client.wallet_groups().list(None).await?;
    println!("list: {} group(s)", groups.len());
    for g in &groups {
        println!(
            "  - {} | {} | chain={:?} | {} wallets",
            g.group_id,
            g.name,
            g.chain,
            g.wallet_ids.len()
        );
    }

    if groups.is_empty() {
        println!("No groups to exercise. Bailing.");
        return Ok(());
    }
    let target = &groups[0];

    let fetched = client.wallet_groups().get(&target.group_id).await?;
    println!(
        "get {}: name=\"{}\", {} wallets",
        fetched.group_id,
        fetched.name,
        fetched.wallet_ids.len()
    );

    let renamed = client
        .wallet_groups()
        .update(
            &target.group_id,
            &UpdateWalletGroupBody {
                name: Some("rs-sdk-smoke-renamed".to_string()),
            },
        )
        .await?;
    println!(
        "update -> name=\"{}\" updatedAt={}",
        renamed.name, renamed.updated_at
    );

    let reverted = client
        .wallet_groups()
        .update(
            &target.group_id,
            &UpdateWalletGroupBody {
                name: Some(fetched.name.clone()),
            },
        )
        .await?;
    println!("revert -> name=\"{}\"", reverted.name);

    let created = client
        .wallet_groups()
        .create(&CreateWalletGroupBody {
            name: "rs-sdk-smoke-empty".to_string(),
            chain: Some("svm".to_string()),
            wallet_ids: None,
        })
        .await?;
    println!("create empty -> {}", created.group_id);

    let del = client.wallet_groups().delete(&created.group_id).await?;
    println!("delete -> groupId={}", del.group_id);

    println!("\n✅ Rust SDK live smoke OK");
    Ok(())
}
