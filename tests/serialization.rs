use serde_json::json;
use shuriken_sdk::types::Network;

// ── API types deserialization ───────────────────────────────────────────────

#[test]
fn deserialize_token_info() {
    let data = json!({
        "tokenId": "solana:So111",
        "chain": "solana",
        "address": "So111",
        "name": "Wrapped SOL",
        "symbol": "SOL",
        "decimals": 9
    });
    let token: shuriken_sdk::tokens::TokenInfo = serde_json::from_value(data).unwrap();
    assert_eq!(token.symbol, "SOL");
    assert_eq!(token.decimals, 9);
}

#[test]
fn deserialize_token_price_with_null_price() {
    let data = json!({
        "tokenId": "solana:unknown",
        "decimals": 6,
        "priceUsd": null
    });
    let price: shuriken_sdk::tokens::TokenPrice = serde_json::from_value(data).unwrap();
    assert!(price.price_usd.is_none());
}

#[test]
fn deserialize_token_price_change_stats() {
    let data = json!({
        "5m": 1.5,
        "1h": -0.3,
        "6h": null,
        "24h": 12.0
    });
    let stats: shuriken_sdk::tokens::TokenPriceChangeStats = serde_json::from_value(data).unwrap();
    assert_eq!(stats.m5, Some(1.5));
    assert_eq!(stats.h1, Some(-0.3));
    assert!(stats.h6.is_none());
    assert_eq!(stats.h24, Some(12.0));
}

#[test]
fn deserialize_swap_quote() {
    let data = json!({
        "quoteId": "q_123",
        "chain": "solana",
        "inputMint": "So111",
        "outputMint": "EPjF",
        "inAmount": "1000000000",
        "outAmount": "50000000",
        "slippageBps": 100,
        "expiresAt": "2026-01-01T00:00:00Z",
        "priceImpactPct": "0.5",
        "fees": {
            "platformFeeAmount": "100000",
            "platformFeeBps": 30,
            "dexFeeInNative": null
        },
        "routes": [{
            "source": "Jupiter",
            "inAmount": "1000000000",
            "outAmount": "50000000",
            "feeMint": null,
            "poolFeeTier": null
        }]
    });
    let quote: shuriken_sdk::swap::SwapQuote = serde_json::from_value(data).unwrap();
    assert_eq!(quote.quote_id, "q_123");
    assert_eq!(quote.slippage_bps, 100);
    assert_eq!(quote.routes.len(), 1);
    assert_eq!(quote.routes[0].source, "Jupiter");
}

#[test]
fn deserialize_swap_status() {
    let data = json!({
        "taskId": "t_456",
        "status": "success",
        "txHash": "abc123",
        "errorCode": null,
        "errorMessage": null
    });
    let status: shuriken_sdk::swap::SwapStatus = serde_json::from_value(data).unwrap();
    assert_eq!(status.status, "success");
    assert_eq!(status.tx_hash, Some("abc123".into()));
}

#[test]
fn deserialize_perp_market() {
    let data = json!({
        "meta": {
            "name": "BTC",
            "assetIndex": 0,
            "szDecimals": 4,
            "maxLeverage": 50,
            "onlyIsolated": false
        },
        "ctx": {
            "midPx": "60000.0",
            "markPx": "60001.0",
            "oraclePx": "59999.0",
            "prevDayPx": "59000.0",
            "dayNtlVlm": "1000000",
            "funding": "0.0001",
            "openInterest": "500000",
            "premium": "0.001"
        },
        "asks": [{"price": "60002.0", "size": "1.5", "numOrders": 3}],
        "bids": [{"price": "59998.0", "size": "2.0", "numOrders": 5}]
    });
    let market: shuriken_sdk::perps::PerpMarket = serde_json::from_value(data).unwrap();
    assert_eq!(market.meta.name, "BTC");
    assert_eq!(market.meta.max_leverage, 50);
    assert_eq!(market.asks.len(), 1);
    assert_eq!(market.bids[0].num_orders, 5);
}

#[test]
fn deserialize_account_info() {
    let data = json!({
        "userId": "u_789",
        "displayName": "TestUser"
    });
    let info: shuriken_sdk::account::AccountInfo = serde_json::from_value(data).unwrap();
    assert_eq!(info.user_id, "u_789");
    assert_eq!(info.display_name, Some("TestUser".into()));
}

#[test]
fn deserialize_account_info_null_display_name() {
    let data = json!({
        "userId": "u_789",
        "displayName": null
    });
    let info: shuriken_sdk::account::AccountInfo = serde_json::from_value(data).unwrap();
    assert!(info.display_name.is_none());
}

#[test]
fn deserialize_trigger_order() {
    let data = json!({
        "orderId": "o_1",
        "status": "active",
        "chain": "solana",
        "inputToken": "So111",
        "outputToken": "EPjF",
        "amount": "1000000",
        "createdAt": "2026-01-01T00:00:00Z",
        "trigger": {
            "metric": "price_usd",
            "direction": "above",
            "value": "0.001",
            "trailingPercentage": null
        }
    });
    let order: shuriken_sdk::trigger::TriggerOrder = serde_json::from_value(data).unwrap();
    assert_eq!(order.order_id, "o_1");
    assert_eq!(order.trigger.metric, "price_usd");
    assert!(order.trigger.trailing_percentage.is_none());
}

#[test]
fn deserialize_portfolio_pnl() {
    let data = json!({
        "totalValueUsd": 1234.56,
        "totalBoughtUsd": 1000.0,
        "totalSoldUsd": 500.0,
        "totalPnlUsd": 234.56,
        "totalRealizedPnlUsd": 100.0,
        "totalUnrealizedPnlUsd": 134.56,
        "positionCount": 3,
        "portfolioHistory": [
            {"timestamp": 1700000000, "valueUsd": 1000.0},
            {"timestamp": 1700003600, "valueUsd": 1234.56}
        ]
    });
    let pnl: shuriken_sdk::portfolio::PortfolioPnl = serde_json::from_value(data).unwrap();
    assert_eq!(pnl.position_count, 3);
    assert_eq!(pnl.portfolio_history.len(), 2);
}

// ── shuriken-api-types deserialization ──────────────────────────────────────

#[test]
fn deserialize_svm_swap_event() {
    let data = json!({
        "tokenMint": "So111",
        "signature": "sig123",
        "slot": 300000000,
        "blockTime": 1700000000,
        "isBuy": true,
        "sizeSol": "1.5",
        "sizeUsd": "150.0",
        "priceUsd": "100.0",
        "priceSol": "1.0",
        "network": "sol"
    });
    let event: shuriken_sdk::types::svm::SwapEvent = serde_json::from_value(data).unwrap();
    assert_eq!(event.token_mint, "So111");
    assert!(event.is_buy);
    assert_eq!(event.network, Network::Sol);
}

#[test]
fn deserialize_evm_swap_event() {
    let data = json!({
        "tokenAddress": "0xtoken",
        "txHash": "0xhash",
        "chainId": 8453,
        "blockNumber": 1000000,
        "timestamp": 1700000000,
        "isBuy": false,
        "amountNative": "0.5",
        "amountUsd": "1500.0",
        "priceNative": "3000.0",
        "priceUsd": "3000.0",
        "tokenDecimals": 18,
        "tokenInAddress": "0xin",
        "tokenOutAddress": "0xout",
        "amountIn": "500000000000000000",
        "amountOut": "1500000000",
        "network": "base"
    });
    let event: shuriken_sdk::types::evm::SwapEvent = serde_json::from_value(data).unwrap();
    assert_eq!(event.chain_id, 8453);
    assert!(!event.is_buy);
    assert_eq!(event.network, Network::Base);
}

#[test]
fn deserialize_svm_bonding_curve_creation_event() {
    let data = json!({
        "tokenAddress": "token1",
        "curveAddress": "curve1",
        "curveDexType": "pump",
        "creator": "creator1",
        "signature": "sig1",
        "slot": 300000000u64,
        "blockTime": 1700000000i64,
        "blockHeight": 250000000u64,
        "blockHash": "hash1",
        "network": "sol"
    });
    let event: shuriken_sdk::types::svm::BondingCurveCreationEvent =
        serde_json::from_value(data).unwrap();
    assert_eq!(event.curve_dex_type, "pump");
    assert_eq!(event.curve_address, "curve1");
}

#[test]
fn deserialize_svm_bonding_curve_graduation_event() {
    let data = json!({
        "tokenAddress": "token1",
        "curveAddress": "curve1",
        "curveDexType": "pump",
        "destPoolAddress": "pool1",
        "destPoolDexType": "raydium",
        "signature": "sig1",
        "slot": 300000000u64,
        "blockTime": 1700000000i64,
        "blockHeight": 250000000u64,
        "blockHash": "hash1",
        "network": "sol"
    });
    let event: shuriken_sdk::types::svm::BondingCurveGraduationEvent =
        serde_json::from_value(data).unwrap();
    assert_eq!(event.curve_dex_type, "pump");
    assert_eq!(event.dest_pool_dex_type, "raydium");
}

#[test]
fn deserialize_wallet_balance_events() {
    let svm_data = json!({
        "owner": "wallet1",
        "slot": 300000000,
        "blockTime": 1700000000,
        "preBalance": 1000000000u64,
        "postBalance": 2000000000u64,
        "network": "sol"
    });
    let event: shuriken_sdk::types::wallet::SvmNativeBalanceEvent =
        serde_json::from_value(svm_data).unwrap();
    assert_eq!(event.pre_balance, 1_000_000_000);
    assert_eq!(event.post_balance, 2_000_000_000);

    let evm_data = json!({
        "owner": "0xwallet",
        "chainId": 1,
        "blockNumber": 1000000,
        "blockTime": 1700000000,
        "balance": "5000000000000000000",
        "network": "eth"
    });
    let event: shuriken_sdk::types::wallet::EvmNativeBalanceEvent =
        serde_json::from_value(evm_data).unwrap();
    assert_eq!(event.network, Network::Eth);
}

// ── Request serialization ───────────────────────────────────────────────────

#[test]
fn serialize_execute_swap_params() {
    let params = shuriken_sdk::swap::ExecuteSwapParams {
        chain: "solana".into(),
        input_mint: "So111".into(),
        output_mint: "EPjF".into(),
        amount: "1000000000".into(),
        wallet_id: "w_1".into(),
        slippage_bps: None,
    };
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["chain"], "solana");
    assert_eq!(json["walletId"], "w_1");
    // slippageBps should be omitted when None
    assert!(json.get("slippageBps").is_none());
}

#[test]
fn serialize_place_order_params() {
    let params = shuriken_sdk::perps::PlaceOrderParams {
        wallet_id: "w_1".into(),
        coin: "BTC".into(),
        is_buy: true,
        sz: Some("0.1".into()),
        limit_px: Some("60000".into()),
        ..Default::default()
    };
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["coin"], "BTC");
    assert_eq!(json["isBuy"], true);
    assert!(json.get("orderType").is_none());
}

#[test]
fn serialize_create_trigger_order_params() {
    let params = shuriken_sdk::trigger::CreateTriggerOrderParams {
        chain: "solana".into(),
        input_token: "So111".into(),
        output_token: "EPjF".into(),
        amount: "1000000".into(),
        wallet_id: "w_1".into(),
        trigger_metric: "price_usd".into(),
        trigger_direction: "above".into(),
        trigger_value: Some("0.001".into()),
        ..Default::default()
    };
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["triggerMetric"], "price_usd");
    assert_eq!(json["triggerValue"], "0.001");
    assert!(json.get("trailingPercentage").is_none());
}

// ── Client construction ─────────────────────────────────────────────────────

#[test]
fn http_client_new() {
    let client = shuriken_sdk::ShurikenHttpClient::new("sk_test123");
    assert!(client.is_ok());
}

#[test]
fn http_client_with_base_url() {
    let client =
        shuriken_sdk::ShurikenHttpClient::with_base_url("sk_test", "https://staging.example.com/");
    assert!(client.is_ok());
}

#[test]
fn deserialize_swap_preset_evm() {
    let data = serde_json::json!({
        "type": "evm",
        "slippageBps": 1500,
        "maxPriceImpactPct": 10.0,
        "bribeAmountNative": "0",
        "mevProtectionEnabled": true
    });
    let preset: shuriken_sdk::account::SwapPreset = serde_json::from_value(data).unwrap();
    match preset {
        shuriken_sdk::account::SwapPreset::Evm { slippage_bps, .. } => {
            assert_eq!(slippage_bps, 1500);
        }
        _ => panic!("expected Evm variant"),
    }
}

#[test]
fn deserialize_swap_preset_solana() {
    let data = serde_json::json!({
        "type": "solana",
        "slippageBps": 1500,
        "customPriorityFeeSol": "0.001",
        "bribeAmountSol": "0.001",
        "maxPriceImpactPct": 10.0,
        "mevProtectionEnabled": true
    });
    let preset: shuriken_sdk::account::SwapPreset = serde_json::from_value(data).unwrap();
    match preset {
        shuriken_sdk::account::SwapPreset::Solana { slippage_bps, .. } => {
            assert_eq!(slippage_bps, 1500);
        }
        _ => panic!("expected Solana variant"),
    }
}
