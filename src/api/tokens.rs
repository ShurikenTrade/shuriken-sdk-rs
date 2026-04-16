use serde::{Deserialize, Serialize};

use crate::client::ShurikenClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub token_id: String,
    pub chain: String,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPool {
    pub address: Option<String>,
    pub liquidity_usd: Option<String>,
    pub market_cap_usd: Option<String>,
    pub price_usd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    pub token_id: String,
    pub decimals: u32,
    pub price_usd: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenChartCandle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenChart {
    pub token_id: String,
    pub resolution: String,
    pub candles: Vec<TokenChartCandle>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenVolumeStats {
    pub buy5m: Option<f64>,
    pub buy1h: Option<f64>,
    pub buy6h: Option<f64>,
    pub buy24h: Option<f64>,
    pub sell5m: Option<f64>,
    pub sell1h: Option<f64>,
    pub sell6h: Option<f64>,
    pub sell24h: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTxnStats {
    pub buys5m: Option<u64>,
    pub buys1h: Option<u64>,
    pub buys6h: Option<u64>,
    pub buys24h: Option<u64>,
    pub sells5m: Option<u64>,
    pub sells1h: Option<u64>,
    pub sells6h: Option<u64>,
    pub sells24h: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUniqueTradersStats {
    pub buyers5m: Option<u64>,
    pub buyers1h: Option<u64>,
    pub buyers6h: Option<u64>,
    pub buyers24h: Option<u64>,
    pub sellers5m: Option<u64>,
    pub sellers1h: Option<u64>,
    pub sellers6h: Option<u64>,
    pub sellers24h: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPriceChangeStats {
    #[serde(rename = "5m")]
    pub m5: Option<f64>,
    #[serde(rename = "1h")]
    pub h1: Option<f64>,
    #[serde(rename = "6h")]
    pub h6: Option<f64>,
    #[serde(rename = "24h")]
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    pub token_id: String,
    pub volume: TokenVolumeStats,
    pub txns: TokenTxnStats,
    pub unique_traders: TokenUniqueTradersStats,
    pub price_change: TokenPriceChangeStats,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPools {
    pub token_id: String,
    pub pools: Vec<TokenPool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTokensResponse {
    pub tokens: Vec<TokenInfo>,
    pub not_found: Vec<String>,
    pub invalid: Vec<String>,
    pub errors: Vec<String>,
}

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchTokensParams {
    pub q: String,
    pub chain: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GetTokenChartParams {
    pub token_id: String,
    pub resolution: Option<String>,
    pub count: Option<u32>,
}

// ── API methods ─────────────────────────────────────────────────────────────

impl ShurikenClient {
    pub async fn get_token(&self, token_id: &str) -> Result<TokenInfo, ShurikenError> {
        self.get(&format!("/api/v2/tokens/{token_id}")).await
    }

    pub async fn search_tokens(
        &self,
        params: &SearchTokensParams,
    ) -> Result<Vec<TokenInfo>, ShurikenError> {
        let mut query = vec![("q", params.q.clone())];
        if let Some(chain) = &params.chain {
            query.push(("chain", chain.clone()));
        }
        if let Some(page) = params.page {
            query.push(("page", page.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        self.get_with_query("/api/v2/tokens/search", &query).await
    }

    pub async fn batch_tokens(
        &self,
        tokens: &[String],
    ) -> Result<BatchTokensResponse, ShurikenError> {
        #[derive(Serialize)]
        struct Body<'a> {
            tokens: &'a [String],
        }
        self.post("/api/v2/tokens/batch", &Body { tokens }).await
    }

    pub async fn get_token_price(&self, token_id: &str) -> Result<TokenPrice, ShurikenError> {
        self.get(&format!("/api/v2/tokens/{token_id}/price")).await
    }

    pub async fn get_token_chart(
        &self,
        params: &GetTokenChartParams,
    ) -> Result<TokenChart, ShurikenError> {
        let mut query = Vec::new();
        if let Some(resolution) = &params.resolution {
            query.push(("resolution", resolution.clone()));
        }
        if let Some(count) = params.count {
            query.push(("count", count.to_string()));
        }
        self.get_with_query(
            &format!("/api/v2/tokens/{}/price/chart", params.token_id),
            &query,
        )
        .await
    }

    pub async fn get_token_stats(&self, token_id: &str) -> Result<TokenStats, ShurikenError> {
        self.get(&format!("/api/v2/tokens/{token_id}/stats")).await
    }

    pub async fn get_token_pools(&self, token_id: &str) -> Result<TokenPools, ShurikenError> {
        self.get(&format!("/api/v2/tokens/{token_id}/pools")).await
    }
}
