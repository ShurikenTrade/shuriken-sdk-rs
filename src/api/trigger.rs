use serde::{Deserialize, Serialize};

use crate::client::ShurikenClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCondition {
    pub metric: String,
    pub direction: String,
    pub value: Option<String>,
    pub trailing_percentage: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOrder {
    pub order_id: String,
    pub status: String,
    pub chain: String,
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub created_at: String,
    pub trigger: TriggerCondition,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOrderView {
    pub order_id: String,
    pub status: String,
    pub chain: Option<String>,
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub created_at: String,
    pub updated_at: String,
    pub trigger: Option<TriggerCondition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelledTriggerOrder {
    pub order_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOrdersResponse {
    pub orders: Vec<TriggerOrderView>,
    pub next_cursor: Option<String>,
}

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTriggerOrderParams {
    pub chain: String,
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub wallet_id: String,
    pub trigger_metric: String,
    pub trigger_direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_hours: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct ListTriggerOrdersParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

impl ShurikenClient {
    pub async fn create_trigger_order(
        &self,
        params: &CreateTriggerOrderParams,
    ) -> Result<TriggerOrder, ShurikenError> {
        self.post("/api/v2/trigger/order", params).await
    }

    pub async fn get_trigger_order(
        &self,
        order_id: &str,
    ) -> Result<TriggerOrderView, ShurikenError> {
        self.get(&format!("/api/v2/trigger/order/{order_id}")).await
    }

    pub async fn list_trigger_orders(
        &self,
        params: &ListTriggerOrdersParams,
    ) -> Result<TriggerOrdersResponse, ShurikenError> {
        let mut query = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = &params.cursor {
            query.push(("cursor", cursor.clone()));
        }
        self.get_with_query("/api/v2/trigger/orders", &query).await
    }

    pub async fn cancel_trigger_order(
        &self,
        order_id: &str,
    ) -> Result<CancelledTriggerOrder, ShurikenError> {
        self.delete(&format!("/api/v2/trigger/order/{order_id}"))
            .await
    }
}
