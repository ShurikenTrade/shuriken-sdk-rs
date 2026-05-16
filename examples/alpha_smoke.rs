//! Live smoke test for the alpha signal endpoints.
//!
//! Usage:
//!   SHURIKEN_API_KEY=sk_... \
//!   SHURIKEN_API_BASE_URL=https://shuriken-api-staging.tsw.infraweninfra.xyz \
//!     cargo run --example alpha_smoke
//!
//! Exercises all five alpha endpoints. Requires the `read:alpha` scope (or
//! equivalent) on the agent key.

use shuriken_sdk::alpha::{GetCallContextParams, GetGlobalCallsParams, GetRecentCallsParams};
use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY required");
    let base_url = std::env::var("SHURIKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.shuriken.trade".to_string());

    let client = ShurikenHttpClient::with_base_url(&api_key, &base_url)?;

    // ── Sources ──────────────────────────────────────────────────────────────
    println!("Fetching alpha sources from {base_url}...");
    let sources = client.alpha().get_sources().await?;
    println!("  {} source(s)", sources.sources.len());
    for s in &sources.sources {
        println!(
            "  - [{type}] {name}  enabled={enabled}",
            type = s.connection_type,
            name = s.name.as_deref().unwrap_or("<unnamed>"),
            enabled = s.enabled,
        );
    }

    // ── Recent calls ─────────────────────────────────────────────────────────
    println!("\nFetching recent calls (limit=5)...");
    let recent = client
        .alpha()
        .get_recent_calls(GetRecentCallsParams {
            limit: Some(5),
            ..Default::default()
        })
        .await?;
    println!(
        "  total={total}  returned={n}",
        total = recent.total_count,
        n = recent.calls.len()
    );
    for call in &recent.calls {
        println!(
            "  - {sym} ({addr}) chain={chain} mentions={cnt}",
            sym = call.token_symbol.as_deref().unwrap_or("?"),
            addr = &call.token_address[..call.token_address.len().min(12)],
            chain = call.chain,
            cnt = call.mention_count,
        );
    }

    // ── Global calls ─────────────────────────────────────────────────────────
    println!("\nFetching global calls (limit=3)...");
    let global = client
        .alpha()
        .get_global_calls(GetGlobalCallsParams {
            limit: Some(3),
            ..Default::default()
        })
        .await?;
    println!(
        "  platform={platform}  total={total}  returned={n}",
        platform = global.platform,
        total = global.total_count,
        n = global.calls.len()
    );

    // ── Call context for the first recent-call token ─────────────────────────
    if let Some(first) = recent.calls.first() {
        let addr = &first.token_address;
        println!("\nFetching call context for {addr} (limit=2)...");
        let ctx = client
            .alpha()
            .get_call_context(
                addr,
                GetCallContextParams {
                    limit: Some(2),
                    ..Default::default()
                },
            )
            .await?;
        println!(
            "  total_signals={total}  has_more={more}  returned={n}",
            total = ctx.total_signals,
            more = ctx.has_more,
            n = ctx.signals.len(),
        );

        // ── Token mentions ───────────────────────────────────────────────────
        println!("\nFetching token mentions for {addr} (limit=3)...");
        let mentions = client
            .alpha()
            .get_token_mentions(
                addr,
                shuriken_sdk::alpha::GetTokenMentionsParams { limit: Some(3) },
            )
            .await?;
        println!(
            "  total_mentions={total}  returned={n}",
            total = mentions.total_mentions,
            n = mentions.mentions.len(),
        );
    } else {
        println!("\nNo recent calls found — skipping call-context and mention checks.");
    }

    println!("\nAll alpha endpoints OK.");
    Ok(())
}
