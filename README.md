# shuriken-sdk

[![crates.io](https://img.shields.io/crates/v/shuriken-sdk)](https://crates.io/crates/shuriken-sdk)

Rust SDK for the [Shuriken](https://app.shuriken.trade) API.

> **Status:** Early development — API surface may change.

## Install

```toml
[dependencies]
shuriken-sdk = "0.3"

# Enable WebSocket streams:
# shuriken-sdk = { version = "0.3", features = ["ws"] }
```

## Quick start

```rust
use shuriken_sdk::ShurikenHttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ShurikenHttpClient::new("sk_...")?;

    // REST — works immediately
    let token = client.tokens().get("solana:So11111111111111111111111111111111111111112").await?;
    println!("{} ({})", token.name, token.symbol);

    Ok(())
}
```

## Tokens

```rust
use shuriken_sdk::tokens::{SearchTokensParams, GetTokenChartParams};

// Search
let results = client.tokens().search(&SearchTokensParams {
    q: "bonk".into(),
    chain: Some("solana".into()),
    page: None,
    limit: None,
}).await?;

// Get token metadata
let token = client.tokens().get("solana:So111...").await?;

// Batch lookup (up to 100)
let batch = client.tokens().batch(&["solana:So111...".into()]).await?;

// Price
let price = client.tokens().get_price("solana:So111...").await?;

// OHLCV chart
let chart = client.tokens().get_chart(&GetTokenChartParams {
    token_id: "solana:So111...".into(),
    resolution: Some("1h".into()),
    count: Some(50),
}).await?;

// Trading stats & pools
let stats = client.tokens().get_stats("solana:So111...").await?;
let pools = client.tokens().get_pools("solana:So111...").await?;
```

## Swap

```rust
use shuriken_sdk::swap::{GetSwapQuoteParams, ExecuteSwapParams, BuildTransactionParams};

// Get a quote
let quote = client.swap().get_quote(&GetSwapQuoteParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    slippage_bps: Some(100),
}).await?;

// Managed execution (Shuriken signs & submits)
let status = client.swap().execute(&ExecuteSwapParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    wallet_id: "w_123".into(),
    slippage_bps: None,
}).await?;

// Build unsigned transaction (for self-signing)
let tx = client.swap().build_transaction(&BuildTransactionParams {
    chain: "solana".into(),
    input_mint: "So111...".into(),
    output_mint: "EPjF...".into(),
    amount: "1000000000".into(),
    wallet_address: "7xKX...".into(),
    slippage_bps: None,
}).await?;

// Poll execution status
let result = client.tasks().get_status("task_id").await?;

// EVM approval helpers
let spender = client.swap().get_approve_spender(8453).await?;
```

## Portfolio

```rust
use shuriken_sdk::portfolio::*;

let balances = client.portfolio().get_balances(&GetBalancesParams { chain: Some("solana".into()) }).await?;
let trades = client.portfolio().get_history(&GetHistoryParams { limit: Some(50), ..Default::default() }).await?;
let pnl = client.portfolio().get_pnl(&GetPnlParams { timeframe: Some("7d".into()) }).await?;
let positions = client.portfolio().get_positions(&GetPositionsParams::default()).await?;
```

## Account

```rust
let me = client.account().get_me().await?;
let wallets = client.account().get_wallets().await?;
let usage = client.account().get_usage().await?;
let settings = client.account().get_settings().await?;

// Enable multisend (durable nonce) on a Solana wallet
let resp = client.account().enable_multisend("wallet-id").await?;
println!("Task ID: {}", resp.task_id);
```

## Trigger orders

```rust
use shuriken_sdk::trigger::*;

let order = client.trigger().create(&CreateTriggerOrderParams {
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

let orders = client.trigger().list(&ListTriggerOrdersParams { limit: Some(50), ..Default::default() }).await?;
let detail = client.trigger().get("order_id").await?;
let cancelled = client.trigger().cancel("order_id").await?;
```

## Perps

```rust
use shuriken_sdk::perps::*;

// Markets
let markets = client.perps().get_markets().await?;
let btc = client.perps().get_market("BTC").await?;

// Account
let account = client.perps().get_account(&GetPerpAccountParams::default()).await?;

// Place an order
let result = client.perps().place_order(&PlaceOrderParams {
    wallet_id: "w_123".into(),
    coin: "BTC".into(),
    is_buy: true,
    sz: Some("0.1".into()),
    limit_px: Some("60000".into()),
    order_type: Some("limit".into()),
    ..Default::default()
}).await?;

// Close position
client.perps().close_position(&ClosePositionParams {
    wallet_id: "w_123".into(),
    coin: "BTC".into(),
    percentage: Some(100.0),
}).await?;
```

## WebSocket streams

Requires the `ws` feature.

```rust
use shuriken_sdk::{ShurikenWsClient, streams};
use shuriken_sdk::streams::SvmTokenFilter;
use futures_util::StreamExt;

let mut ws = ShurikenWsClient::new("sk_...")?;
ws.connect().await?;

// Subscribe to a typed stream
let mut sub = ws.subscribe(
    streams::SVM_TOKEN_SWAPS,
    SvmTokenFilter { token_address: "So111...".into() },
).await?;

// Each event is already deserialized to the correct type
while let Some(swap) = sub.next().await {
    println!("Swap: {} USD @ {}", swap.size_usd, swap.price_usd);
}

// Disconnect when done
ws.disconnect().await;
```

### Connection state

```rust
use shuriken_sdk::{ShurikenWsClient, ConnectionState};
use futures_util::StreamExt;

let mut ws = ShurikenWsClient::new("sk_...")?;
let mut state_sub = ws.on_state_change().await;

// Listen for state changes in a separate task
tokio::spawn(async move {
    while let Some(event) = state_sub.next().await {
        println!("Connection: {:?}", event.state);
    }
});

ws.connect().await?;
```

### Available streams

| Stream constant | Wire name | Filter | Payload type |
|----------------|-----------|--------|-------------|
| `SVM_TOKEN_SWAPS` | `svm.token.swaps` | `SvmTokenFilter` | `types::svm::SwapEvent` |
| `SVM_TOKEN_POOL_INFO` | `svm.token.poolInfo` | `SvmTokenFilter` | `types::svm::TokenPoolEvent` |
| `SVM_TOKEN_BALANCES` | `svm.token.balances` | `SvmTokenFilter` | `types::svm::TokenBalanceEvent` |
| `SVM_TOKEN_DISTRIBUTION_STATS` | `svm.token.distributionStats` | `SvmTokenFilter` | `types::analytics::TokenDistributionStatsEvent` |
| `SVM_TOKEN_HOLDER_STATS` | `svm.token.holderStats` | `SvmTokenFilter` | `types::analytics::HolderStatsEvent` |
| `SVM_WALLET_NATIVE_BALANCE` | `svm.wallet.nativeBalance` | `SvmWalletFilter` | `types::wallet::SvmNativeBalanceEvent` |
| `SVM_WALLET_TOKEN_BALANCES` | `svm.wallet.tokenBalances` | `SvmWalletFilter` | `types::wallet::SvmTokenBalanceEvent` |
| `SVM_BONDING_CURVE_CREATIONS` | `svm.bondingCurve.creations` | `NoFilter` | `types::svm::BondingCurveCreationEvent` |
| `SVM_BONDING_CURVE_GRADUATIONS` | `svm.bondingCurve.graduations` | `NoFilter` | `types::svm::BondingCurveGraduationEvent` |
| `EVM_TOKEN_SWAPS` | `evm.token.swaps` | `EvmTokenFilter` | `types::evm::SwapEvent` |
| `EVM_TOKEN_POOL_INFO` | `evm.token.poolInfo` | `EvmTokenFilter` | `types::evm::TokenPoolEvent` |
| `EVM_TOKEN_BALANCES` | `evm.token.balances` | `EvmTokenFilter` | `types::evm::TokenBalanceEvent` |
| `EVM_WALLET_NATIVE_BALANCE` | `evm.wallet.nativeBalance` | `EvmWalletFilter` | `types::wallet::EvmNativeBalanceEvent` |
| `EVM_WALLET_TOKEN_BALANCES` | `evm.wallet.tokenBalances` | `EvmWalletFilter` | `types::evm::TokenBalanceEvent` |
| `ALPHA_SIGNAL_FEED_GLOBAL` | `alpha.signalFeed.global` | `NoFilter` | `types::alpha::SignalFeedUpdateEvent` |
| `ALPHA_SIGNAL_FEED_PERSONAL` | `alpha.signalFeed.personal` | `NoFilter` | `types::alpha::SignalFeedUpdateEvent` |
| `ALPHA_SIGNAL_FEED_PROFILE` | `alpha.signalFeed.profile` | `AlphaProfileFilter` | `types::alpha::SignalFeedUpdateEvent` |
| `ALPHA_SIGNAL_FEED_NAMED` | `alpha.signalFeed.named` | `AlphaNamedFeedFilter` | `types::alpha::SignalFeedUpdateEvent` |
| `ALPHA_PERSONAL` | `alpha.personal` | `NoFilter` | `types::alpha::MessageEvent` |
| `PORTFOLIO_NOTIFICATIONS` | `portfolio.notifications` | `NoFilter` | `types::notification::NotificationEvent` |
| `AUTOMATION_UPDATES` | `automation.updates` | `NoFilter` | `types::automation::AutomationEvent` |

## Authentication

Get an API key from the [Shuriken Agents dashboard](https://app.shuriken.trade/agents). The SDK uses agent keys (`sk_...`) for authentication.

## License

MIT
