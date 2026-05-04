//! Live smoke test for SplitNOW cross-chain splits.
//!
//! Usage:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!   SHURIKEN_SOURCE_WALLET=cmoxxx... \
//!   SHURIKEN_DEST_WALLET_1=cmoxxx... \
//!   SHURIKEN_DEST_WALLET_2=cmoxxx... \
//!     cargo run --example splits_smoke
//!
//! Plans a 50/50 SOL split across two destinations and prints the plan_id +
//! top quoted exchanger. Does not execute. Requires `split:plan` scope; add
//! `split:execute` to drive the second step.

use shuriken_sdk::splits::{PlanSplitBody, PlanSplitDestination};
use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY required");
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());
    let source = std::env::var("SHURIKEN_SOURCE_WALLET").expect("SHURIKEN_SOURCE_WALLET required");
    let dest_1 = std::env::var("SHURIKEN_DEST_WALLET_1").expect("SHURIKEN_DEST_WALLET_1 required");
    let dest_2 = std::env::var("SHURIKEN_DEST_WALLET_2").expect("SHURIKEN_DEST_WALLET_2 required");

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    let body = PlanSplitBody {
        source_wallet_id: source,
        destination_group_id: None,
        destinations: Some(vec![
            PlanSplitDestination {
                wallet_id: dest_1,
                pct_bips: 5_000,
            },
            PlanSplitDestination {
                wallet_id: dest_2,
                pct_bips: 5_000,
            },
        ]),
        from_amount: "0.01".to_string(),
        from_asset: "sol".to_string(),
        agent_comment: Some("splits_smoke example".to_string()),
    };

    println!("Planning split...");
    let plan = client.splits().plan(&body).await?;
    println!(
        "  plan_id={}  destinations={}  expires_in={}s",
        plan.plan_id, plan.destination_count, plan.expires_in_seconds
    );

    let top = plan
        .rates
        .iter()
        .filter(|r| r.exchange_rate.parse::<f64>().unwrap_or(0.0) > 0.0)
        .max_by(|a, b| {
            a.exchange_rate
                .parse::<f64>()
                .unwrap_or(0.0)
                .partial_cmp(&b.exchange_rate.parse::<f64>().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    if let Some(top) = top {
        println!(
            "  top rate: {} @ {} {}/{}",
            top.exchanger_id, top.exchange_rate, top.to_asset_id, top.to_network_id
        );
    }

    println!("(execute_split skipped — re-run with split:execute scope and the plan_id above to drive step 2)");

    Ok(())
}
