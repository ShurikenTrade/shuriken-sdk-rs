use serde::de::DeserializeOwned;

fn api_key() -> String {
    dotenvy::from_path("/Users/nik/Projects/shuriken/shuriken-quickstart-rs/.env").ok();
    std::env::var("SHURIKEN_API_KEY").expect("SHURIKEN_API_KEY not set")
}

const BASE: &str = "https://api.shuriken.trade";

async fn fetch_get(client: &reqwest::Client, path: &str) -> Option<serde_json::Value> {
    let resp = client
        .get(format!("{BASE}{path}"))
        .send()
        .await
        .unwrap_or_else(|e| panic!("{path}: request failed: {e}"));
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        println!("SKIP {path}: HTTP {status}");
        return None;
    }
    let body: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: not JSON: {e}\n{text}"));
    Some(match body.get("data") {
        Some(d) => d.clone(),
        None => body,
    })
}

async fn fetch_post(
    client: &reqwest::Client,
    path: &str,
    body: &serde_json::Value,
) -> Option<serde_json::Value> {
    let resp = client
        .post(format!("{BASE}{path}"))
        .json(body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("{path}: request failed: {e}"));
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        println!("SKIP {path}: HTTP {status}");
        return None;
    }
    let body: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: not JSON: {e}\n{text}"));
    Some(match body.get("data") {
        Some(d) => d.clone(),
        None => body,
    })
}

fn check<T: DeserializeOwned>(label: &str, val: &serde_json::Value, errors: &mut Vec<String>) {
    println!("\n--- {label} ---");
    let pretty = serde_json::to_string_pretty(val).unwrap();
    // Print at most 2000 chars
    if pretty.len() > 2000 {
        println!("{}... (truncated)", &pretty[..2000]);
    } else {
        println!("{pretty}");
    }
    if let Err(e) = serde_json::from_value::<T>(val.clone()) {
        let msg = format!("{label}: {e}");
        println!("FAIL: {msg}");
        errors.push(msg);
    } else {
        println!("OK");
    }
}

const TOKEN_ID: &str = "solana:JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

#[tokio::test]
#[ignore]
async fn live_api_deserialization() {
    let key = api_key();
    let client = reqwest::Client::builder()
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {key}")).unwrap(),
            );
            h
        })
        .build()
        .unwrap();

    let mut errors: Vec<String> = Vec::new();

    // ── Account ───────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(&client, "/api/v2/account/me").await {
        check::<shuriken_sdk::account::AccountInfo>("account/me", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/account/settings").await {
        check::<shuriken_sdk::account::AccountSettings>("account/settings", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/account/usage").await {
        check::<shuriken_sdk::account::AccountUsage>("account/usage", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/account/wallets").await {
        check::<Vec<shuriken_sdk::account::AccountWallet>>("account/wallets", &d, &mut errors);
    }

    // ── Tokens ────────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(&client, &format!("/api/v2/tokens/{TOKEN_ID}")).await {
        check::<shuriken_sdk::tokens::TokenInfo>("tokens/get", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/tokens/search?q=bonk&limit=3").await {
        #[derive(serde::Deserialize)]
        struct W {
            #[allow(dead_code)]
            tokens: Vec<shuriken_sdk::tokens::TokenInfo>,
        }
        check::<W>("tokens/search", &d, &mut errors);
    }
    if let Some(d) = fetch_post(
        &client,
        "/api/v2/tokens/batch",
        &serde_json::json!({"tokens": [TOKEN_ID]}),
    )
    .await
    {
        check::<shuriken_sdk::tokens::BatchTokensResponse>("tokens/batch", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, &format!("/api/v2/tokens/{TOKEN_ID}/price")).await {
        check::<shuriken_sdk::tokens::TokenPrice>("tokens/price", &d, &mut errors);
    }
    if let Some(d) = fetch_get(
        &client,
        &format!("/api/v2/tokens/{TOKEN_ID}/price/chart?resolution=1h&count=5"),
    )
    .await
    {
        check::<shuriken_sdk::tokens::TokenChart>("tokens/chart", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, &format!("/api/v2/tokens/{TOKEN_ID}/stats")).await {
        check::<shuriken_sdk::tokens::TokenStats>("tokens/stats", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, &format!("/api/v2/tokens/{TOKEN_ID}/pools")).await {
        check::<shuriken_sdk::tokens::TokenPools>("tokens/pools", &d, &mut errors);
    }

    // ── Swap ──────────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(
        &client,
        "/api/v2/swap/quote?chain=solana&inputMint=So11111111111111111111111111111111111111112&outputMint=JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN&amount=1000000",
    )
    .await
    {
        check::<shuriken_sdk::swap::SwapQuote>("swap/quote", &d, &mut errors);
    }

    // ── Portfolio ─────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(&client, "/api/v2/portfolio/balances").await {
        #[derive(serde::Deserialize)]
        struct W {
            #[allow(dead_code)]
            wallets: Vec<shuriken_sdk::portfolio::WalletBalance>,
        }
        check::<W>("portfolio/balances", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/portfolio/history?limit=3").await {
        #[derive(serde::Deserialize)]
        struct W {
            #[allow(dead_code)]
            trades: Vec<shuriken_sdk::portfolio::PortfolioTrade>,
        }
        check::<W>("portfolio/history", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/portfolio/pnl?timeframe=30d").await {
        check::<shuriken_sdk::portfolio::PortfolioPnl>("portfolio/pnl", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/portfolio/positions").await {
        check::<shuriken_sdk::portfolio::PositionsResponse>("portfolio/positions", &d, &mut errors);
    }

    // ── Trigger ───────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(&client, "/api/v2/trigger/orders").await {
        check::<shuriken_sdk::trigger::TriggerOrdersResponse>("trigger/orders", &d, &mut errors);
    }

    // ── Perps ─────────────────────────────────────────────────────────────
    if let Some(d) = fetch_get(&client, "/api/v2/perp/account").await {
        check::<shuriken_sdk::perps::PerpAccountState>("perp/account", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/perp/fees").await {
        check::<shuriken_sdk::perps::UserFees>("perp/fees", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/perp/markets").await {
        check::<Vec<shuriken_sdk::perps::PerpMarket>>("perp/markets", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/perp/markets/BTC").await {
        check::<shuriken_sdk::perps::PerpMarket>("perp/markets/BTC", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/perp/orders").await {
        check::<Vec<shuriken_sdk::perps::OpenOrder>>("perp/orders", &d, &mut errors);
    }
    if let Some(d) = fetch_get(&client, "/api/v2/perp/positions").await {
        check::<shuriken_sdk::perps::PerpPositionsResponse>("perp/positions", &d, &mut errors);
    }

    // ── Summary ───────────────────────────────────────────────────────────
    println!("\n\n========================================");
    if errors.is_empty() {
        println!("All tested endpoints deserialized OK!");
    } else {
        println!("DESERIALIZATION ERRORS ({}):", errors.len());
        for e in &errors {
            println!("  - {e}");
        }
        panic!("{} endpoint(s) failed deserialization", errors.len());
    }
}
