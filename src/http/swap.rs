use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRoute {
    pub source: String,
    pub in_amount: Option<String>,
    pub out_amount: Option<String>,
    pub fee_mint: Option<String>,
    pub pool_fee_tier: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapFees {
    pub platform_fee_amount: Option<String>,
    pub platform_fee_bps: Option<u32>,
    pub dex_fee_in_native: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
    pub quote_id: String,
    pub chain: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub slippage_bps: u32,
    pub expires_at: String,
    pub price_impact_pct: Option<String>,
    pub fees: SwapFees,
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapStatus {
    pub task_id: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSummary {
    pub input_amount: String,
    pub output_amount: String,
    pub min_output_amount: String,
    pub slippage_bps: u32,
    pub price_impact_pct: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmTransactionData {
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas_limit: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionResponse {
    pub quote_id: String,
    pub chain: String,
    pub chain_id: Option<u64>,
    pub transaction: serde_json::Value,
    pub approval_required: Option<bool>,
    pub approval_transaction: Option<EvmTransactionData>,
    pub expires_at: String,
    pub quote_summary: QuoteSummary,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionResponse {
    pub task_id: String,
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveSpenderResponse {
    pub chain_id: u64,
    pub spender_address: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAllowanceResponse {
    pub chain_id: u64,
    pub token_address: String,
    pub wallet_address: String,
    pub allowance: String,
}

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GetSwapQuoteParams {
    pub chain: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSwapParams {
    pub chain: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub wallet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionParams {
    pub chain: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionParams {
    pub chain: String,
    pub signed_transaction: String,
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetApproveAllowanceParams {
    pub chain_id: u64,
    pub token_address: String,
    pub wallet_address: String,
}

// ── API methods ─────────────────────────────────────────────────────────────

pub struct SwapApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl SwapApi<'_> {
    pub async fn get_quote(&self, params: &GetSwapQuoteParams) -> Result<SwapQuote, ShurikenError> {
        let mut query = vec![
            ("chain", params.chain.clone()),
            ("inputMint", params.input_mint.clone()),
            ("outputMint", params.output_mint.clone()),
            ("amount", params.amount.clone()),
        ];
        if let Some(slippage) = params.slippage_bps {
            query.push(("slippageBps", slippage.to_string()));
        }
        self.0.get_with_query("/api/v2/swap/quote", &query).await
    }

    pub async fn execute(&self, params: &ExecuteSwapParams) -> Result<SwapStatus, ShurikenError> {
        self.0.post("/api/v2/swap/execute", params).await
    }

    pub async fn build_transaction(
        &self,
        params: &BuildTransactionParams,
    ) -> Result<BuildTransactionResponse, ShurikenError> {
        self.0.post("/api/v2/swap/transaction", params).await
    }

    pub async fn submit_transaction(
        &self,
        params: &SubmitTransactionParams,
    ) -> Result<SubmitTransactionResponse, ShurikenError> {
        self.0.post("/api/v2/swap/submit", params).await
    }

    pub async fn get_status(&self, task_id: &str) -> Result<SwapStatus, ShurikenError> {
        self.0.get(&format!("/api/v2/swap/status/{task_id}")).await
    }

    pub async fn get_approve_spender(
        &self,
        chain_id: u64,
    ) -> Result<ApproveSpenderResponse, ShurikenError> {
        self.0
            .get_with_query(
                "/api/v2/swap/approve/spender",
                &[("chainId", chain_id.to_string())],
            )
            .await
    }

    pub async fn get_approve_allowance(
        &self,
        params: &GetApproveAllowanceParams,
    ) -> Result<ApproveAllowanceResponse, ShurikenError> {
        self.0
            .get_with_query(
                "/api/v2/swap/approve/allowance",
                &[
                    ("chainId", params.chain_id.to_string()),
                    ("tokenAddress", params.token_address.clone()),
                    ("walletAddress", params.wallet_address.clone()),
                ],
            )
            .await
    }
}
