# shuriken-sdk

[![crates.io](https://img.shields.io/crates/v/shuriken-sdk)](https://crates.io/crates/shuriken-sdk)

Rust SDK for the [Shuriken](https://app.shuriken.trade) API.

> **Status:** Early development — API surface may change.

## Install

```toml
[dependencies]
shuriken-sdk = "0.1"

# Enable WebSocket streams:
# shuriken-sdk = { version = "0.1", features = ["ws"] }
```

## Quick start

```rust
use shuriken_sdk::ShurikenClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ShurikenClient::new("sk_...")?;

    // REST — works immediately
    let token = client.get_token("solana:So11111111111111111111111111111111111111112").await?;
    println!("{} ({})", token.name, token.symbol);

    Ok(())
}
```

## Tokens

```rust
use shuriken_sdk::tokens::{SearchTokensParams, GetTokenChartParams};

// Search
let results = client.search_tokens(&SearchTokensParams {
    q: "bonk".into(),
    chain: Some("solana".into()),
    page: None,
    limit: None,
}).await?;

// Get token metadata
let token = client.get_token("solana:So111...").await?;

// Batch lookup (up to 100)
let batch = client.batch_tokens(&["solana:So111...".into()]).await?;

// Price
let price = client.get_token_price("solana:So111...").await?;

// OHLCV chart
let chart = client.get_token_chart(&GetTokenChartParams {
    token_id: "solana:So111...".into(),
    resolution: Some("1h".into()),
    count: Some(50),
}).await?;

// Trading stats & pools
let stats = client.get_token_stats("solana:So111...").await?;
let pools = client.get_token_pools("solana:So111...").await?;
```

## Swap

```rust
use shuriken_sdk::swap::{GetSwapQuoteParams, ExecuteSwapParams, BuildTransactionParams};

// Get a quote
let quote = client.get_swap_quote(&GetSwapQuoteParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    slippage_bps: Some(100),
}).await?;

// Managed execution (Shuriken signs & submits)
let status = client.execute_swap(&ExecuteSwapParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    wallet_id: "w_123".into(),
    slippage_bps: None,
}).await?;

// Build unsigned transaction (for self-signing)
let tx = client.build_transaction(&BuildTransactionParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    wallet_address: "7xKX...".into(),
    slippage_bps: None,
}).await?;

// Poll execution status
let result = client.get_swap_status("task_id").await?;

// EVM approval helpers
let spender = client.get_approve_spender(8453).await?;
```

## Portfolio

```rust
use shuriken_sdk::portfolio::*;

let balances = client.get_balances(&GetBalancesParams { chain: Some("solana".into()) }).await?;
let trades = client.get_history(&GetHistoryParams { limit: Some(50), ..Default::default() }).await?;
let pnl = client.get_pnl(&GetPnlParams { timeframe: Some("7d".into()) }).await?;
let positions = client.get_positions(&GetPositionsParams::default()).await?;
```

## Account

```rust
let me = client.get_me().await?;
let wallets = client.get_wallets().await?;
let usage = client.get_usage().await?;
let settings = client.get_settings().await?;
```

## Trigger orders

```rust
use shuriken_sdk::trigger::*;

let order = client.create_trigger_order(&CreateTriggerOrderParams {
    chain: "solana".into(),
    input_token: "So111...".into(),
    output_token: "EPjF...".into(),
    amount: "1000000000".into(),
    wallet_id: "w_123".into(),
    trigger_metric: "price_usd".into(),
    trigger_direction: "above".into(),
    trigger_value: Some("0.001".into()),
    ..Default::default()
}).await?;

let orders = client.list_trigger_orders(&ListTriggerOrdersParams { limit: Some(50), ..Default::default() }).await?;
let detail = client.get_trigger_order("order_id").await?;
let cancelled = client.cancel_trigger_order("order_id").await?;
```

## Perps

```rust
use shuriken_sdk::perps::*;

// Markets
let markets = client.get_perp_markets().await?;
let btc = client.get_perp_market("BTC").await?;

// Account
let account = client.get_perp_account(&GetPerpAccountParams::default()).await?;

// Place an order
let result = client.place_perp_order(&PlaceOrderParams {
    wallet_id: "w_123".into(),
    coin: "BTC".into(),
    is_buy: true,
    sz: Some("0.1".into()),
    limit_px: Some("60000".into()),
    order_type: Some("limit".into()),
    ..Default::default()
}).await?;

// Close position
client.close_perp_position(&ClosePositionParams {
    wallet_id: "w_123".into(),
    coin: "BTC".into(),
    percentage: Some(100.0),
}).await?;
```

## WebSocket streams

Requires the `ws` feature.

```rust
use std::collections::HashMap;

let client = ShurikenClient::new("sk_...")?;
client.ws.connect().await?;

let mut filter = HashMap::new();
filter.insert("tokenAddress".into(), "So111...".into());

client.ws.subscribe("svm.token.swaps", filter, |event| {
    // Deserialize into the shared types from shuriken-api-types
    if let Ok(swap) = serde_json::from_value::<shuriken_sdk::types::svm::SwapEvent>(event) {
        println!("Swap: {} USD @ {}", swap.size_usd, swap.price_usd);
    }
}).await?;

// Listen for state changes
client.ws.on_state_change(|event| {
    println!("Connection: {:?}", event.state);
}).await;

// Disconnect when done
client.ws.disconnect().await;
```

### Available streams

| Stream | Filter | Payload type |
|--------|--------|-------------|
| `svm.token.swaps` | `tokenAddress` | `types::svm::SwapEvent` |
| `svm.token.poolInfo` | `tokenAddress` | `types::svm::TokenPoolEvent` |
| `svm.token.balances` | `tokenAddress` | `types::svm::TokenBalanceEvent` |
| `svm.wallet.nativeBalance` | `walletAddress` | `types::wallet::SvmNativeBalanceEvent` |
| `svm.wallet.tokenBalances` | `walletAddress` | `types::wallet::SvmTokenBalanceEvent` |
| `svm.bondingCurve.creations` | — | `types::svm::BondingCurveCreationEvent` |
| `svm.bondingCurve.graduations` | — | `types::svm::BondingCurveGraduationEvent` |
| `evm.token.swaps` | `chainId`, `tokenAddress` | `types::evm::SwapEvent` |
| `evm.token.poolInfo` | `chainId`, `tokenAddress` | `types::evm::TokenPoolEvent` |
| `evm.token.balances` | `tokenAddress` | `types::evm::TokenBalanceEvent` |
| `evm.wallet.nativeBalance` | `walletAddress` | `types::wallet::EvmNativeBalanceEvent` |
| `evm.wallet.tokenBalances` | `walletAddress` | `types::wallet::EvmNativeBalanceEvent` |

## Authentication

Get an API key from the [Shuriken Agents dashboard](https://app.shuriken.trade/agents). The SDK uses agent keys (`sk_...`) for authentication.

## License

MIT
