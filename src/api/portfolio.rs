use serde::Deserialize;

use crate::client::ShurikenClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub chain: String,
    pub wallet_address: String,
    pub native_balance: String,
    pub native_balance_usd: f64,
    pub native_symbol: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioTrade {
    pub chain: String,
    pub tx_hash: String,
    pub timestamp: u64,
    pub wallet_address: String,
    pub input_token: String,
    pub input_amount: Option<String>,
    pub output_token: String,
    pub output_amount: Option<String>,
    pub token: String,
    pub size_usd: String,
    pub price_usd: String,
    pub is_buy: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioHistoryPoint {
    pub timestamp: u64,
    pub value_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPnl {
    pub total_value_usd: f64,
    pub total_bought_usd: f64,
    pub total_sold_usd: f64,
    pub total_pnl_usd: f64,
    pub total_realized_pnl_usd: f64,
    pub total_unrealized_pnl_usd: f64,
    pub position_count: u32,
    pub portfolio_history: Vec<PortfolioHistoryPoint>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    pub wallet_address: String,
    pub token_address: String,
    pub latest_balance_raw: String,
    pub latest_token_usd_price: f64,
    pub token_decimal: u32,
    pub bought_usd: f64,
    pub sold_usd: f64,
    pub bought_native: f64,
    pub sold_native: f64,
    pub buy_count: u32,
    pub sell_count: u32,
    pub balance_usd: f64,
    pub balance_native: f64,
    pub realised_pnl_pct: f64,
    pub total_pnl_pct: f64,
    pub network: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionsResponse {
    pub positions: Vec<PositionInfo>,
    pub total_value_usd: f64,
    pub position_count: u32,
}

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct GetBalancesParams {
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetHistoryParams {
    pub chain: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPnlParams {
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetPositionsParams {
    pub chain: Option<String>,
    pub status: Option<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

impl ShurikenClient {
    pub async fn get_balances(
        &self,
        params: &GetBalancesParams,
    ) -> Result<Vec<WalletBalance>, ShurikenError> {
        let mut query = Vec::new();
        if let Some(chain) = &params.chain {
            query.push(("chain", chain.clone()));
        }
        self.get_with_query("/api/v2/portfolio/balances", &query)
            .await
    }

    pub async fn get_history(
        &self,
        params: &GetHistoryParams,
    ) -> Result<Vec<PortfolioTrade>, ShurikenError> {
        let mut query = Vec::new();
        if let Some(chain) = &params.chain {
            query.push(("chain", chain.clone()));
        }
        if let Some(page) = params.page {
            query.push(("page", page.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        self.get_with_query("/api/v2/portfolio/history", &query)
            .await
    }

    pub async fn get_pnl(&self, params: &GetPnlParams) -> Result<PortfolioPnl, ShurikenError> {
        let mut query = Vec::new();
        if let Some(timeframe) = &params.timeframe {
            query.push(("timeframe", timeframe.clone()));
        }
        self.get_with_query("/api/v2/portfolio/pnl", &query).await
    }

    pub async fn get_positions(
        &self,
        params: &GetPositionsParams,
    ) -> Result<PositionsResponse, ShurikenError> {
        let mut query = Vec::new();
        if let Some(chain) = &params.chain {
            query.push(("chain", chain.clone()));
        }
        if let Some(status) = &params.status {
            query.push(("status", status.clone()));
        }
        self.get_with_query("/api/v2/portfolio/positions", &query)
            .await
    }
}
