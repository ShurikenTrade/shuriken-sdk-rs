use serde::{Deserialize, Serialize};

use crate::client::ShurikenClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotBalance {
    pub coin: String,
    pub total: String,
    pub hold: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpAccountState {
    pub account_value: String,
    pub withdrawable: String,
    pub spot_balances: Vec<SpotBalance>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFees {
    pub daily_volume: String,
    pub maker_rate: String,
    pub taker_rate: String,
    pub referral_discount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpFill {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub fee: String,
    pub closed_pnl: String,
    pub time: u64,
    pub oid: u64,
    pub start_position: String,
    pub direction: String,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingPayment {
    pub coin: String,
    pub usdc: String,
    pub funding_rate: String,
    pub szi: String,
    pub time: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMeta {
    pub name: String,
    pub asset_index: u32,
    pub sz_decimals: u32,
    pub max_leverage: u32,
    pub only_isolated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCtx {
    pub mid_px: String,
    pub mark_px: String,
    pub oracle_px: String,
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub funding: String,
    pub open_interest: String,
    pub premium: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookLevel {
    pub price: String,
    pub size: String,
    pub num_orders: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpMarket {
    pub meta: MarketMeta,
    pub ctx: MarketCtx,
    pub asks: Vec<BookLevel>,
    pub bids: Vec<BookLevel>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResult {
    pub status: String,
    pub oid: Option<u64>,
    pub cloid: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub results: Vec<OrderResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
    pub order_type: String,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpPosition {
    pub coin: String,
    pub szi: String,
    pub entry_px: String,
    pub unrealized_pnl: String,
    pub return_on_equity: String,
    pub liquidation_px: String,
    pub leverage_type: String,
    pub leverage_value: String,
    pub margin_used: String,
    pub position_value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpPositionsResponse {
    pub positions: Vec<PerpPosition>,
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub withdrawable: String,
    pub spot_balances: Option<Vec<SpotBalance>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginResponse {
    pub success: bool,
    pub error: Option<String>,
}

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TpSlParams {
    pub trigger_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_market: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_px: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPerpAccountParams {
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPerpFeesParams {
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetPerpFillsParams {
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub coin: Option<String>,
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetPerpFundingParams {
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub coin: Option<String>,
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPerpOrdersParams {
    pub coin: Option<String>,
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPerpPositionsParams {
    pub wallet_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderParams {
    pub wallet_id: String,
    pub coin: String,
    pub is_buy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_px: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sz: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_usd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp: Option<TpSlParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sl: Option<TpSlParams>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyOrderParams {
    pub wallet_id: String,
    pub coin: String,
    pub is_buy: bool,
    pub sz: String,
    pub limit_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_cloid: Option<String>,
    pub order_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderParams {
    pub wallet_id: String,
    pub coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_all: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchModifyEntry {
    pub coin: String,
    pub is_buy: bool,
    pub sz: String,
    pub limit_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_cloid: Option<String>,
    pub order_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchModifyParams {
    pub wallet_id: String,
    pub modifications: Vec<BatchModifyEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePositionParams {
    pub wallet_id: String,
    pub coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverageParams {
    pub wallet_id: String,
    pub coin: String,
    pub leverage: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cross: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarginParams {
    pub wallet_id: String,
    pub coin: String,
    pub amount: String,
}

// ── API methods ─────────────────────────────────────────────────────────────

fn wallet_query(wallet_id: &Option<String>) -> Vec<(&'static str, String)> {
    wallet_id
        .as_ref()
        .map(|w| vec![("wallet_id", w.clone())])
        .unwrap_or_default()
}

impl ShurikenClient {
    pub async fn get_perp_account(
        &self,
        params: &GetPerpAccountParams,
    ) -> Result<PerpAccountState, ShurikenError> {
        self.get_with_query("/api/v2/perp/account", &wallet_query(&params.wallet_id))
            .await
    }

    pub async fn get_perp_fees(
        &self,
        params: &GetPerpFeesParams,
    ) -> Result<UserFees, ShurikenError> {
        self.get_with_query("/api/v2/perp/fees", &wallet_query(&params.wallet_id))
            .await
    }

    pub async fn get_perp_fills(
        &self,
        params: &GetPerpFillsParams,
    ) -> Result<Vec<PerpFill>, ShurikenError> {
        let mut query = vec![("start_time", params.start_time.to_string())];
        if let Some(end) = params.end_time {
            query.push(("end_time", end.to_string()));
        }
        if let Some(coin) = &params.coin {
            query.push(("coin", coin.clone()));
        }
        if let Some(w) = &params.wallet_id {
            query.push(("wallet_id", w.clone()));
        }
        self.get_with_query("/api/v2/perp/fills", &query).await
    }

    pub async fn get_perp_funding(
        &self,
        params: &GetPerpFundingParams,
    ) -> Result<Vec<FundingPayment>, ShurikenError> {
        let mut query = vec![("start_time", params.start_time.to_string())];
        if let Some(end) = params.end_time {
            query.push(("end_time", end.to_string()));
        }
        if let Some(coin) = &params.coin {
            query.push(("coin", coin.clone()));
        }
        if let Some(w) = &params.wallet_id {
            query.push(("wallet_id", w.clone()));
        }
        self.get_with_query("/api/v2/perp/funding", &query).await
    }

    pub async fn get_perp_markets(&self) -> Result<Vec<PerpMarket>, ShurikenError> {
        self.get("/api/v2/perp/markets").await
    }

    pub async fn get_perp_market(&self, coin: &str) -> Result<PerpMarket, ShurikenError> {
        self.get(&format!("/api/v2/perp/markets/{coin}")).await
    }

    pub async fn get_perp_orders(
        &self,
        params: &GetPerpOrdersParams,
    ) -> Result<Vec<OpenOrder>, ShurikenError> {
        let mut query = wallet_query(&params.wallet_id);
        if let Some(coin) = &params.coin {
            query.push(("coin", coin.clone()));
        }
        self.get_with_query("/api/v2/perp/orders", &query).await
    }

    pub async fn get_perp_positions(
        &self,
        params: &GetPerpPositionsParams,
    ) -> Result<PerpPositionsResponse, ShurikenError> {
        self.get_with_query("/api/v2/perp/positions", &wallet_query(&params.wallet_id))
            .await
    }

    pub async fn place_perp_order(
        &self,
        params: &PlaceOrderParams,
    ) -> Result<OrderResponse, ShurikenError> {
        self.post("/api/v2/perp/order", params).await
    }

    pub async fn modify_perp_order(
        &self,
        params: &ModifyOrderParams,
    ) -> Result<OrderResponse, ShurikenError> {
        self.patch("/api/v2/perp/order", params).await
    }

    pub async fn cancel_perp_order(
        &self,
        params: &CancelOrderParams,
    ) -> Result<OrderResponse, ShurikenError> {
        self.delete_with_body("/api/v2/perp/order", params).await
    }

    pub async fn batch_modify_perp_orders(
        &self,
        params: &BatchModifyParams,
    ) -> Result<OrderResponse, ShurikenError> {
        self.patch("/api/v2/perp/orders", params).await
    }

    pub async fn close_perp_position(
        &self,
        params: &ClosePositionParams,
    ) -> Result<OrderResponse, ShurikenError> {
        self.post("/api/v2/perp/position/close", params).await
    }

    pub async fn update_perp_leverage(
        &self,
        params: &UpdateLeverageParams,
    ) -> Result<LeverageResponse, ShurikenError> {
        self.post("/api/v2/perp/leverage", params).await
    }

    pub async fn update_perp_margin(
        &self,
        params: &UpdateMarginParams,
    ) -> Result<MarginResponse, ShurikenError> {
        self.post("/api/v2/perp/position/margin", params).await
    }
}
