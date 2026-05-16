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
fn deserialize_task_status() {
    let data = json!({
        "taskId": "t_789",
        "taskType": "swap",
        "status": "success",
        "txHash": "def456",
        "errorCode": null,
        "errorMessage": null
    });
    let task: shuriken_sdk::tasks::TaskStatus = serde_json::from_value(data).unwrap();
    assert_eq!(task.task_id, "t_789");
    assert_eq!(task.task_type, "swap");
    assert_eq!(task.status, "success");
    assert_eq!(task.tx_hash, Some("def456".into()));
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

// ── Wallet groups ──────────────────────────────────────────────────────────

#[test]
fn deserialize_wallet_group_record() {
    let data = json!({
        "groupId": "cmoige9wn0006glkr8bdr123d",
        "name": "treasury",
        "chain": "svm",
        "walletIds": ["w1", "w2"],
        "createdAt": "2026-04-28T09:58:37.224Z",
        "updatedAt": "2026-04-28T10:51:37.162Z"
    });
    let group: shuriken_sdk::wallet_groups::WalletGroupRecord =
        serde_json::from_value(data).unwrap();
    assert_eq!(group.group_id, "cmoige9wn0006glkr8bdr123d");
    assert_eq!(group.name, "treasury");
    assert_eq!(group.chain.as_deref(), Some("svm"));
    assert_eq!(group.wallet_ids.len(), 2);
}

#[test]
fn deserialize_wallet_group_record_with_null_chain() {
    let data = json!({
        "groupId": "g1",
        "name": "n",
        "chain": null,
        "walletIds": [],
        "createdAt": "2026-04-28T09:58:37.224Z",
        "updatedAt": "2026-04-28T09:58:37.224Z"
    });
    let group: shuriken_sdk::wallet_groups::WalletGroupRecord =
        serde_json::from_value(data).unwrap();
    assert!(group.chain.is_none());
    assert!(group.wallet_ids.is_empty());
}

#[test]
fn deserialize_delete_wallet_group_response() {
    let data = json!({ "groupId": "cmoige9wn0006glkr8bdr123d" });
    let resp: shuriken_sdk::wallet_groups::DeleteWalletGroupResponse =
        serde_json::from_value(data).unwrap();
    assert_eq!(resp.group_id, "cmoige9wn0006glkr8bdr123d");
}

#[test]
fn create_wallet_group_with_wallets_body_serializes_camel_case() {
    let body = shuriken_sdk::wallet_groups::CreateWalletGroupWithWalletsBody {
        name: "treasury".to_string(),
        chain: "svm".to_string(),
        wallet_count: 4,
    };
    let json_str = serde_json::to_string(&body).unwrap();
    assert!(json_str.contains("\"walletCount\":4"));
    assert!(json_str.contains("\"chain\":\"svm\""));
}

#[test]
fn create_wallet_group_body_omits_none_fields() {
    let body = shuriken_sdk::wallet_groups::CreateWalletGroupBody {
        name: "n".to_string(),
        chain: None,
        wallet_ids: None,
    };
    let json_str = serde_json::to_string(&body).unwrap();
    assert!(!json_str.contains("chain"));
    assert!(!json_str.contains("walletIds"));
}

#[test]
fn add_wallets_body_camel_case_with_position() {
    let body = shuriken_sdk::wallet_groups::AddWalletsToGroupBody {
        wallet_ids: vec!["w1".to_string()],
        position: Some(0),
    };
    let json_str = serde_json::to_string(&body).unwrap();
    assert!(json_str.contains("\"walletIds\":[\"w1\"]"));
    assert!(json_str.contains("\"position\":0"));
}

#[test]
fn move_wallet_body_omits_none_fields() {
    let body = shuriken_sdk::wallet_groups::MoveWalletBody {
        from_group_id: None,
        to_group_id: Some("g2".to_string()),
    };
    let json_str = serde_json::to_string(&body).unwrap();
    assert!(!json_str.contains("fromGroupId"));
    assert!(json_str.contains("\"toGroupId\":\"g2\""));
}

// ─── wallets (archive lifecycle) ───────────────────────────────────────────

#[test]
fn wallet_record_round_trips_camel_case() {
    let json = serde_json::json!({
        "walletId": "w1",
        "address": "0xabc",
        "chain": "base",
        "label": "treasury",
        "state": "ARCHIVED",
        "archivedAt": "2026-05-04T12:00:00Z"
    });
    let w: shuriken_sdk::wallets::WalletRecord = serde_json::from_value(json).unwrap();
    assert_eq!(w.wallet_id, "w1");
    assert_eq!(w.state, "ARCHIVED");
    assert_eq!(w.archived_at.as_deref(), Some("2026-05-04T12:00:00Z"));
}

#[test]
fn wallet_record_active_omits_archived_at() {
    let json = serde_json::json!({
        "walletId": "w1",
        "address": "0xabc",
        "state": "ACTIVE"
    });
    let w: shuriken_sdk::wallets::WalletRecord = serde_json::from_value(json).unwrap();
    assert_eq!(w.state, "ACTIVE");
    assert!(w.archived_at.is_none());
    assert!(w.chain.is_none());
    assert!(w.label.is_none());
}

#[test]
fn archive_response_carries_cleared_default() {
    let json = serde_json::json!({
        "wallet": {
            "walletId": "w1",
            "address": "abc",
            "state": "ARCHIVED",
            "archivedAt": "2026-05-04T12:00:00Z"
        },
        "clearedDefault": true
    });
    let r: shuriken_sdk::wallets::ArchiveResponse = serde_json::from_value(json).unwrap();
    assert_eq!(r.wallet.wallet_id, "w1");
    assert!(r.cleared_default);
}

#[test]
fn bulk_archive_request_serializes_camel_case() {
    let body = shuriken_sdk::wallets::BulkArchiveRequest {
        wallet_ids: vec!["w1".into(), "w2".into()],
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"walletIds\":[\"w1\",\"w2\"]"));
}

#[test]
fn bulk_archive_entry_omits_cleared_default_when_none() {
    let json = serde_json::json!({
        "walletId": "w1",
        "status": "already_archived"
    });
    let e: shuriken_sdk::wallets::BulkArchiveEntry = serde_json::from_value(json).unwrap();
    assert_eq!(e.status, "already_archived");
    assert!(e.cleared_default.is_none());
}

// ─── transfers ─────────────────────────────────────────────────────────────

#[test]
fn send_body_full_serializes_camel_case() {
    let body = shuriken_sdk::transfers::SendBody {
        from_wallet_id: "wa".into(),
        to_wallet_id: "wb".into(),
        token: "USDC".into(),
        amount: "1000000".into(),
        chain: "EVM".into(),
        chain_id: Some(8453),
        await_result: Some(false),
        correlation_id: Some("corr-123".into()),
        agent_comment: Some("topup".into()),
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"fromWalletId\":\"wa\""));
    assert!(s.contains("\"toWalletId\":\"wb\""));
    assert!(s.contains("\"chainId\":8453"));
    assert!(s.contains("\"awaitResult\":false"));
    assert!(s.contains("\"correlationId\":\"corr-123\""));
    assert!(s.contains("\"agentComment\":\"topup\""));
}

#[test]
fn send_body_minimal_omits_optional_fields() {
    let body = shuriken_sdk::transfers::SendBody {
        from_wallet_id: "wa".into(),
        to_wallet_id: "wb".into(),
        token: "SOL".into(),
        amount: "1000000000".into(),
        chain: "SVM".into(),
        chain_id: None,
        await_result: None,
        correlation_id: None,
        agent_comment: None,
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(!s.contains("chainId"));
    assert!(!s.contains("awaitResult"));
    assert!(!s.contains("correlationId"));
    assert!(!s.contains("agentComment"));
}

#[test]
fn retire_wallet_body_minimal_omits_optional_fields() {
    let body = shuriken_sdk::transfers::RetireWalletBody {
        from_wallet_id: "wa".into(),
        to_wallet_id: "wb".into(),
        token: "native".into(),
        chain: "SVM".into(),
        chain_id: None,
        await_result: None,
        correlation_id: None,
        agent_comment: None,
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(!s.contains("chainId"));
    assert!(!s.contains("awaitResult"));
}

#[test]
fn transfer_result_success_carries_transaction() {
    let json = serde_json::json!({
        "taskId": "t-1",
        "status": "SUCCESS",
        "willArchiveOnSuccess": false,
        "transaction": {"hash": "0xabc", "explorerUrl": "https://basescan.org/tx/0xabc"}
    });
    let r: shuriken_sdk::transfers::TransferResult = serde_json::from_value(json).unwrap();
    assert_eq!(r.task_id, "t-1");
    assert_eq!(r.status, "SUCCESS");
    let txn = r.transaction.expect("transaction present on success");
    assert_eq!(txn.hash, "0xabc");
    assert_eq!(
        txn.explorer_url.as_deref(),
        Some("https://basescan.org/tx/0xabc")
    );
    assert!(r.error.is_none());
}

#[test]
fn transfer_result_failed_carries_error() {
    let json = serde_json::json!({
        "taskId": "t-2",
        "status": "FAILED",
        "willArchiveOnSuccess": true,
        "error": {"code": "INSUFFICIENT_BALANCE_FOR_GAS", "message": "Not enough gas"}
    });
    let r: shuriken_sdk::transfers::TransferResult = serde_json::from_value(json).unwrap();
    let err = r.error.expect("error present on failure");
    assert_eq!(err.code, "INSUFFICIENT_BALANCE_FOR_GAS");
    assert!(r.transaction.is_none());
    assert!(r.will_archive_on_success);
}

#[test]
fn transfer_result_pending_no_settlement() {
    let json = serde_json::json!({
        "taskId": "t-3",
        "status": "PENDING",
        "willArchiveOnSuccess": false
    });
    let r: shuriken_sdk::transfers::TransferResult = serde_json::from_value(json).unwrap();
    assert_eq!(r.status, "PENDING");
    assert!(r.transaction.is_none());
    assert!(r.error.is_none());
}

// ─── splits ────────────────────────────────────────────────────────────────

#[test]
fn plan_split_body_with_inline_destinations_serializes_camel_case() {
    let body = shuriken_sdk::splits::PlanSplitBody {
        source_wallet_id: "src".into(),
        destination_group_id: None,
        destinations: Some(vec![
            shuriken_sdk::splits::PlanSplitDestination {
                wallet_id: "d1".into(),
                pct_bips: 5_000,
            },
            shuriken_sdk::splits::PlanSplitDestination {
                wallet_id: "d2".into(),
                pct_bips: 5_000,
            },
        ]),
        from_amount: "0.16".into(),
        from_asset: "sol".into(),
        agent_comment: None,
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"sourceWalletId\":\"src\""));
    assert!(s.contains("\"fromAmount\":\"0.16\""));
    assert!(s.contains("\"fromAsset\":\"sol\""));
    assert!(s.contains("\"pctBips\":5000"));
    assert!(s.contains("\"walletId\":\"d1\""));
    assert!(!s.contains("destinationGroupId"));
    assert!(!s.contains("agentComment"));
}

#[test]
fn plan_split_body_with_group_omits_destinations() {
    let body = shuriken_sdk::splits::PlanSplitBody {
        source_wallet_id: "src".into(),
        destination_group_id: Some("g1".into()),
        destinations: None,
        from_amount: "0.5".into(),
        from_asset: "sol".into(),
        agent_comment: Some("daily fan-out".into()),
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"destinationGroupId\":\"g1\""));
    assert!(!s.contains("\"destinations\""));
    assert!(s.contains("\"agentComment\":\"daily fan-out\""));
}

#[test]
fn plan_split_result_round_trips() {
    let json = serde_json::json!({
        "planId": "p-1",
        "destinationCount": 2,
        "summary": "Split 0.16 SOL from src across 2 destinations",
        "rates": [
            {"exchangerId": "binance", "exchangeRate": "1.0", "toAssetId": "sol", "toNetworkId": "solana"},
            {"exchangerId": "bybit", "exchangeRate": "0", "toAssetId": "sol", "toNetworkId": "solana"}
        ],
        "warnings": [],
        "expiresAt": "2026-05-04T13:00:00Z",
        "expiresInSeconds": 60
    });
    let r: shuriken_sdk::splits::PlanSplitResult = serde_json::from_value(json).unwrap();
    assert_eq!(r.plan_id, "p-1");
    assert_eq!(r.destination_count, 2);
    assert_eq!(r.expires_in_seconds, 60);
    assert_eq!(r.rates.len(), 2);
    assert_eq!(r.rates[0].exchanger_id, "binance");
}

#[test]
fn execute_split_body_omits_optional_agent_comment() {
    let body = shuriken_sdk::splits::ExecuteSplitBody {
        plan_id: "p-1".into(),
        agent_comment: None,
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"planId\":\"p-1\""));
    assert!(!s.contains("agentComment"));
}

#[test]
fn execute_split_result_round_trips() {
    let json = serde_json::json!({
        "taskId": "t-99",
        "splitnowOrderId": "abc123"
    });
    let r: shuriken_sdk::splits::ExecuteSplitResult = serde_json::from_value(json).unwrap();
    assert_eq!(r.task_id, "t-99");
    assert_eq!(r.splitnow_order_id, "abc123");
}

// ─── suggestions ───────────────────────────────────────────────────────────

#[test]
fn create_suggestion_request_round_trips_with_camel_case() {
    let body = shuriken_sdk::suggestions::CreateSuggestionRequest {
        side: shuriken_sdk::suggestions::SuggestionSide::Buy,
        network_id: "SOL".into(),
        asset: "So11111111111111111111111111111111111111112".into(),
        rationale: "Funding flipped positive after a flush.".into(),
        amount_in_usd: Some(250.0),
        confidence: Some(shuriken_sdk::suggestions::SuggestionConfidence::Medium),
    };
    let v = serde_json::to_value(&body).unwrap();
    assert_eq!(v["side"], "BUY");
    assert_eq!(v["networkId"], "SOL");
    assert_eq!(v["asset"], "So11111111111111111111111111111111111111112");
    assert_eq!(v["rationale"], "Funding flipped positive after a flush.");
    assert_eq!(v["amountInUsd"], 250.0);
    assert_eq!(v["confidence"], "MEDIUM");

    // Round-trip back through deserialise.
    let back: shuriken_sdk::suggestions::CreateSuggestionRequest =
        serde_json::from_value(v).unwrap();
    assert_eq!(back.network_id, "SOL");
    assert_eq!(back.side, shuriken_sdk::suggestions::SuggestionSide::Buy);
    assert_eq!(
        back.confidence,
        Some(shuriken_sdk::suggestions::SuggestionConfidence::Medium)
    );
}

#[test]
fn create_suggestion_request_omits_none_optionals() {
    let body = shuriken_sdk::suggestions::CreateSuggestionRequest {
        side: shuriken_sdk::suggestions::SuggestionSide::Sell,
        network_id: "BASE".into(),
        asset: "0xabc".into(),
        rationale: "Take profit.".into(),
        amount_in_usd: None,
        confidence: None,
    };
    let s = serde_json::to_string(&body).unwrap();
    assert!(s.contains("\"side\":\"SELL\""));
    assert!(s.contains("\"networkId\":\"BASE\""));
    assert!(!s.contains("amountInUsd"));
    assert!(!s.contains("confidence"));
}

#[test]
fn trade_suggestion_deserialises_full_shape() {
    let data = json!({
        "id": "sug_01h8mxnkv1qz3sb0r5e0f7n4ck",
        "state": "OPEN",
        "createdAt": "2026-05-09T12:00:00.000Z",
        "expiresAt": "2026-05-09T18:00:00.000Z",
        "actedAt": null,
        "dismissedAt": null,
        "dismissReason": null,
        "linkedTaskId": null,
        "side": "BUY",
        "networkId": "SOL",
        "asset": {
            "address": "So11111111111111111111111111111111111111112",
            "symbol": "SOL",
            "name": "Solana",
            "priceUsd": 162.34
        },
        "rationale": "Funding flipped positive after a flush.",
        "amountInUsd": 250.0,
        "confidence": "MEDIUM",
        "agentKey": { "id": "ak_123", "name": "alpha-scout" }
    });
    let s: shuriken_sdk::suggestions::TradeSuggestion = serde_json::from_value(data).unwrap();
    assert_eq!(s.id, "sug_01h8mxnkv1qz3sb0r5e0f7n4ck");
    assert_eq!(s.state, shuriken_sdk::suggestions::SuggestionState::Open);
    assert_eq!(s.side, shuriken_sdk::suggestions::SuggestionSide::Buy);
    assert_eq!(s.network_id, "SOL");
    assert_eq!(s.asset.symbol, "SOL");
    assert_eq!(s.asset.price_usd, Some(162.34));
    assert_eq!(s.amount_in_usd, Some(250.0));
    assert_eq!(
        s.confidence,
        Some(shuriken_sdk::suggestions::SuggestionConfidence::Medium)
    );
    assert_eq!(s.agent_key.id, "ak_123");
    assert_eq!(s.agent_key.name.as_deref(), Some("alpha-scout"));
    assert!(s.acted_at.is_none());
    assert!(s.dismissed_at.is_none());
    assert!(s.dismiss_reason.is_none());
    assert!(s.linked_task_id.is_none());
}

#[test]
fn trade_suggestion_deserialises_with_null_nullables() {
    let data = json!({
        "id": "sug_1",
        "state": "DISMISSED",
        "createdAt": "2026-05-09T12:00:00.000Z",
        "expiresAt": "2026-05-09T18:00:00.000Z",
        "actedAt": null,
        "dismissedAt": "2026-05-09T12:30:00.000Z",
        "dismissReason": "too risky",
        "linkedTaskId": null,
        "side": "SELL",
        "networkId": "BASE",
        "asset": {
            "address": "0xabc",
            "symbol": "WETH",
            "name": "Wrapped Ether",
            "priceUsd": null
        },
        "rationale": "Resistance reclaim failed.",
        "amountInUsd": null,
        "confidence": null,
        "agentKey": { "id": "ak_42", "name": null }
    });
    let s: shuriken_sdk::suggestions::TradeSuggestion = serde_json::from_value(data).unwrap();
    assert_eq!(
        s.state,
        shuriken_sdk::suggestions::SuggestionState::Dismissed
    );
    assert_eq!(s.dismiss_reason.as_deref(), Some("too risky"));
    assert!(s.amount_in_usd.is_none());
    assert!(s.confidence.is_none());
    assert!(s.asset.price_usd.is_none());
    assert!(s.agent_key.name.is_none());
}

#[test]
fn list_suggestions_query_omits_none_params() {
    let q = shuriken_sdk::suggestions::ListSuggestionsQuery::default();
    let s = serde_json::to_string(&q).unwrap();
    assert_eq!(s, "{}");

    let q2 = shuriken_sdk::suggestions::ListSuggestionsQuery {
        state: Some("ALL".into()),
        limit: None,
        cursor: None,
    };
    let s2 = serde_json::to_string(&q2).unwrap();
    assert!(s2.contains("\"state\":\"ALL\""));
    assert!(!s2.contains("limit"));
    assert!(!s2.contains("cursor"));
}

#[test]
fn list_suggestions_response_round_trips() {
    let data = json!({
        "suggestions": [],
        "nextCursor": "cursor_abc"
    });
    let r: shuriken_sdk::suggestions::ListSuggestionsResponse =
        serde_json::from_value(data).unwrap();
    assert!(r.suggestions.is_empty());
    assert_eq!(r.next_cursor.as_deref(), Some("cursor_abc"));

    // Missing nextCursor is treated as None.
    let data2 = json!({ "suggestions": [] });
    let r2: shuriken_sdk::suggestions::ListSuggestionsResponse =
        serde_json::from_value(data2).unwrap();
    assert!(r2.next_cursor.is_none());
}

#[test]
fn suggestion_state_serialises_uppercase() {
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionState::Open).unwrap(),
        json!("OPEN")
    );
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionState::Acted).unwrap(),
        json!("ACTED")
    );
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionState::Dismissed).unwrap(),
        json!("DISMISSED")
    );
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionState::Expired).unwrap(),
        json!("EXPIRED")
    );
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionSide::Buy).unwrap(),
        json!("BUY")
    );
    assert_eq!(
        serde_json::to_value(shuriken_sdk::suggestions::SuggestionConfidence::High).unwrap(),
        json!("HIGH")
    );
}

#[test]
fn dismiss_body_omits_reason_when_none() {
    // Mirror the server contract: empty body when reason is absent.
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DismissBodyMirror {
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    }
    let empty = serde_json::to_string(&DismissBodyMirror { reason: None }).unwrap();
    assert_eq!(empty, "{}");

    let with = serde_json::to_string(&DismissBodyMirror {
        reason: Some("too risky".into()),
    })
    .unwrap();
    assert_eq!(with, "{\"reason\":\"too risky\"}");
}

#[test]
fn ack_body_omits_linked_task_id_when_none() {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct AckBodyMirror {
        #[serde(skip_serializing_if = "Option::is_none")]
        linked_task_id: Option<String>,
    }
    let empty = serde_json::to_string(&AckBodyMirror {
        linked_task_id: None,
    })
    .unwrap();
    assert_eq!(empty, "{}");

    let with = serde_json::to_string(&AckBodyMirror {
        linked_task_id: Some("task_999".into()),
    })
    .unwrap();
    assert_eq!(with, "{\"linkedTaskId\":\"task_999\"}");
}

// ── Alpha ────────────────────────────────────────────────────────────────────

#[test]
fn deserialize_alpha_source_item_full() {
    let data = json!({
        "connectionId": "conn_abc123",
        "connectionType": "discord",
        "enabled": true,
        "createdAt": "2026-01-15T10:00:00Z",
        "name": "Alpha Calls Server",
        "platform": "discord",
        "source": "1234567890"
    });
    let item: shuriken_sdk::alpha::AlphaSourceItem = serde_json::from_value(data).unwrap();
    assert_eq!(item.connection_id, "conn_abc123");
    assert_eq!(item.connection_type, "discord");
    assert!(item.enabled);
    assert_eq!(item.created_at.as_deref(), Some("2026-01-15T10:00:00Z"));
    assert_eq!(item.name.as_deref(), Some("Alpha Calls Server"));
}

#[test]
fn deserialize_alpha_source_item_minimal() {
    // Optional fields absent — should not error.
    let data = json!({
        "connectionId": "conn_xyz",
        "connectionType": "telegram",
        "enabled": false
    });
    let item: shuriken_sdk::alpha::AlphaSourceItem = serde_json::from_value(data).unwrap();
    assert_eq!(item.connection_id, "conn_xyz");
    assert!(!item.enabled);
    assert!(item.created_at.is_none());
    assert!(item.name.is_none());
    assert!(item.platform.is_none());
    assert!(item.source.is_none());
}

#[test]
fn deserialize_alpha_sources_result() {
    let data = json!({
        "sources": [
            {
                "connectionId": "conn_1",
                "connectionType": "discord",
                "enabled": true
            },
            {
                "connectionId": "conn_2",
                "connectionType": "twitter",
                "enabled": false
            }
        ]
    });
    let result: shuriken_sdk::alpha::AlphaSourcesResult = serde_json::from_value(data).unwrap();
    assert_eq!(result.sources.len(), 2);
    assert_eq!(result.sources[0].connection_id, "conn_1");
    assert_eq!(result.sources[1].connection_type, "twitter");
}

#[test]
fn deserialize_recent_calls_result() {
    let data = json!({
        "totalCount": 42,
        "calls": [
            {
                "tokenAddress": "So111",
                "tokenSymbol": "SOL",
                "tokenName": "Wrapped SOL",
                "chain": "svm",
                "firstSeenAt": 1700000000_i64,
                "lastSeenAt": 1700001000_i64,
                "mentionCount": 7,
                "priceUsdAtCall": "23.45",
                "currentPriceUsd": "24.10",
                "marketCapUsdAtCall": "10000000",
                "liquidityUsdAtCall": "500000",
                "lastSource": {
                    "platform": "discord",
                    "channelName": "#alpha",
                    "authorUsername": "whale_caller",
                    "messagePreview": "SOL looking good",
                    "sourceName": "Alpha Server",
                    "connectionId": "conn_1"
                }
            }
        ]
    });
    let result: shuriken_sdk::alpha::RecentCallsResult = serde_json::from_value(data).unwrap();
    assert_eq!(result.total_count, 42);
    assert_eq!(result.calls.len(), 1);
    let call = &result.calls[0];
    assert_eq!(call.token_address, "So111");
    assert_eq!(call.mention_count, 7);
    assert_eq!(call.price_usd_at_call.as_deref(), Some("23.45"));
    let src = call.last_source.as_ref().unwrap();
    assert_eq!(src.platform, "discord");
    assert_eq!(src.author_username.as_deref(), Some("whale_caller"));
}

#[test]
fn deserialize_global_calls_result() {
    let data = json!({
        "platform": "twitter",
        "totalCount": 100,
        "calls": [
            {
                "tokenAddress": "EPjF",
                "chain": "svm",
                "firstSeenAt": 1700000000_i64,
                "lastSeenAt": 1700002000_i64,
                "mentionCount": 15,
                "currentPriceUsd": "1.001",
                "priceChangeSinceCallPct": "2.5",
                "lastTweetAuthor": "@cryptoKOL",
                "lastTweetPreview": "USDC stable and ready"
            }
        ]
    });
    let result: shuriken_sdk::alpha::GlobalCallsResult = serde_json::from_value(data).unwrap();
    assert_eq!(result.platform, "twitter");
    assert_eq!(result.total_count, 100);
    assert_eq!(result.calls.len(), 1);
    let call = &result.calls[0];
    assert_eq!(call.last_tweet_author.as_deref(), Some("@cryptoKOL"));
    assert_eq!(call.price_change_since_call_pct.as_deref(), Some("2.5"));
}

#[test]
fn deserialize_call_context_result_with_full_signal() {
    let data = json!({
        "tokenAddress": "So111",
        "totalSignals": 3,
        "hasMore": true,
        "nextCursor": 1700001000_i64,
        "signals": [
            {
                "signalId": "sig_abc",
                "timestampMs": 1700000000000_i64,
                "platform": "discord",
                "isBot": false,
                "priceUsd": 23.45,
                "marketCapUsd": 10000000.0,
                "liquidityUsd": 500000.0,
                "caller": {
                    "username": "whale_user",
                    "displayName": "Whale",
                    "avatarUrl": "https://example.com/avatar.png",
                    "verified": true
                },
                "source": {
                    "guildId": "guild_1",
                    "serverName": "Alpha Server",
                    "channelId": "chan_1",
                    "channelName": "#calls",
                    "topicId": 42,
                    "topicTitle": "Alpha Calls",
                    "tweetId": null,
                    "messageId": "msg_1"
                },
                "messagePreview": "SOL is pumping!",
                "contextMessages": [
                    {
                        "author": "user1",
                        "text": "GM everyone",
                        "timestampMs": 1699999900000_i64,
                        "offset": -1
                    }
                ]
            }
        ]
    });
    let result: shuriken_sdk::alpha::CallContextResult = serde_json::from_value(data).unwrap();
    assert_eq!(result.token_address, "So111");
    assert_eq!(result.total_signals, 3);
    assert!(result.has_more);
    assert_eq!(result.next_cursor, Some(1700001000));
    assert_eq!(result.signals.len(), 1);
    let sig = &result.signals[0];
    assert_eq!(sig.signal_id, "sig_abc");
    assert!(!sig.is_bot);
    assert_eq!(sig.price_usd, Some(23.45));
    let caller = sig.caller.as_ref().unwrap();
    assert_eq!(caller.username.as_deref(), Some("whale_user"));
    assert_eq!(caller.verified, Some(true));
    let ctx = sig.context_messages.as_ref().unwrap();
    assert_eq!(ctx.len(), 1);
    assert_eq!(ctx[0].offset, -1);
    assert_eq!(ctx[0].text, "GM everyone");
}

#[test]
fn deserialize_call_context_signal_minimal() {
    // On-chain / X signals have no messagePreview or contextMessages.
    let data = json!({
        "tokenAddress": "So111",
        "totalSignals": 1,
        "hasMore": false,
        "signals": [
            {
                "signalId": "sig_onchain",
                "timestampMs": 1700000000000_i64,
                "platform": "onchain",
                "isBot": true,
                "tradeData": {
                    "isBuy": true,
                    "amountUsd": "5000.00",
                    "amountNative": "200000000",
                    "walletAddress": "7xKX...",
                    "txSignature": "5abc..."
                }
            }
        ]
    });
    let result: shuriken_sdk::alpha::CallContextResult = serde_json::from_value(data).unwrap();
    let sig = &result.signals[0];
    assert!(sig.is_bot);
    assert!(sig.message_preview.is_none());
    assert!(sig.context_messages.is_none());
    let td = sig.trade_data.as_ref().unwrap();
    assert!(td.is_buy);
    assert_eq!(td.amount_usd, "5000.00");
}

#[test]
fn deserialize_token_mentions_result() {
    let data = json!({
        "tokenAddress": "So111",
        "tokenSymbol": "SOL",
        "chain": "svm",
        "totalMentions": 25,
        "firstSeenAt": 1699000000_i64,
        "lastSeenAt": 1700000000_i64,
        "mentions": [
            {
                "messageId": "msg_abc",
                "platform": "discord",
                "timestamp": 1700000000_i64,
                "channelId": "chan_1",
                "guildId": "guild_1",
                "authorUsername": "caller_1",
                "priceUsdAtMention": "23.00",
                "marketCapUsdAtMention": "9800000"
            }
        ]
    });
    let result: shuriken_sdk::alpha::TokenMentionsResult = serde_json::from_value(data).unwrap();
    assert_eq!(result.token_address, "So111");
    assert_eq!(result.token_symbol.as_deref(), Some("SOL"));
    assert_eq!(result.total_mentions, 25);
    assert_eq!(result.first_seen_at, Some(1699000000));
    assert_eq!(result.mentions.len(), 1);
    let m = &result.mentions[0];
    assert_eq!(m.message_id, "msg_abc");
    assert_eq!(m.price_usd_at_mention.as_deref(), Some("23.00"));
}

#[test]
fn get_recent_calls_params_omits_none_fields() {
    let params = shuriken_sdk::alpha::GetRecentCallsParams::default();
    let s = serde_json::to_string(&params).unwrap();
    assert_eq!(s, "{}");
}

#[test]
fn get_recent_calls_params_serialises_camel_case() {
    let params = shuriken_sdk::alpha::GetRecentCallsParams {
        limit: Some(10),
        source_name: Some("Alpha Server".into()),
        connection_id: Some("conn_1".into()),
    };
    let s = serde_json::to_string(&params).unwrap();
    assert!(s.contains("\"limit\":10"));
    assert!(s.contains("\"sourceName\":\"Alpha Server\""));
    assert!(s.contains("\"connectionId\":\"conn_1\""));
    // No snake_case keys
    assert!(!s.contains("source_name"));
    assert!(!s.contains("connection_id"));
}

#[test]
fn get_global_calls_params_omits_none_fields() {
    let params = shuriken_sdk::alpha::GetGlobalCallsParams::default();
    let s = serde_json::to_string(&params).unwrap();
    assert_eq!(s, "{}");
}

#[test]
fn get_token_mentions_params_omits_none_fields() {
    let params = shuriken_sdk::alpha::GetTokenMentionsParams::default();
    let s = serde_json::to_string(&params).unwrap();
    assert_eq!(s, "{}");
}
