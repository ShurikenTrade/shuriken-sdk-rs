//! SplitNOW cross-chain splits against `/api/v2/splits/*`.
//!
//! Two-step flow mirroring trade plan/execute:
//!
//! 1. [`SplitsApi::plan`] — fetch a SplitNOW quote, validate destinations,
//!    and reserve a 60-second plan slot. Returns a `plan_id`.
//! 2. [`SplitsApi::execute`] — execute the previously-planned split. Returns
//!    a `task_id`; poll `client.tasks().get(...)` for lifecycle updates.
//!
//! Scope split: `split:plan` for `plan` (no funds move); `split:execute` for
//! `execute` (funds route through SplitNOW). Deployments without
//! `agent_kit` / `api_v2` enabled return `503 SPLIT_DISABLED`.

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Request body types ──────────────────────────────────────────────────────

/// One destination in a [`PlanSplitBody`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSplitDestination {
    /// Destination wallet ID owned by the caller.
    pub wallet_id: String,
    /// Percent of the split, in basis points. The sum across all destinations
    /// in one plan must equal exactly `10_000`.
    pub pct_bips: u32,
}

/// Body for [`SplitsApi::plan`].
///
/// Specify exactly one of `destination_group_id` (saved wallet group) or
/// `destinations` (inline list).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSplitBody {
    /// Source wallet ID — the wallet whose funds will be split.
    pub source_wallet_id: String,
    /// Saved wallet group ID. Mutually exclusive with `destinations`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_group_id: Option<String>,
    /// Inline destinations. Mutually exclusive with `destination_group_id`.
    /// `pct_bips` across entries must sum to `10_000`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destinations: Option<Vec<PlanSplitDestination>>,
    /// Amount of `from_asset` to split, decimal string. Example: `"0.16"`.
    pub from_amount: String,
    /// Source asset symbol (lowercase): `"sol"` | `"eth"` | `"bnb"`.
    pub from_asset: String,
    /// Optional brief reasoning for the action — shown in activity feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_comment: Option<String>,
}

/// Body for [`SplitsApi::execute`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSplitBody {
    /// Plan ID from [`PlanSplitResult::plan_id`]. Single-use; expires 60s
    /// after creation.
    pub plan_id: String,
    /// Optional brief reasoning for executing — shown in activity feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_comment: Option<String>,
}

// ── Response types ──────────────────────────────────────────────────────────

/// One quote rate returned by SplitNOW at plan time.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSplitRate {
    /// SplitNOW exchanger ID, e.g. `"binance"`, `"bybit"`.
    pub exchanger_id: String,
    /// Quoted exchange rate as a decimal string. `"0"` means the exchanger
    /// can't currently fulfil this pair.
    pub exchange_rate: String,
    /// Destination asset ID.
    pub to_asset_id: String,
    /// Destination network ID.
    pub to_network_id: String,
}

/// Response from [`SplitsApi::plan`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSplitResult {
    /// Plan ID — pass to [`SplitsApi::execute`]. Valid for 60 seconds.
    pub plan_id: String,
    /// Resolved destination count (matches the on-chain fan-out).
    pub destination_count: u32,
    /// Human-readable summary the agent should read before executing.
    pub summary: String,
    /// Quoted exchanger rates from SplitNOW (informational).
    pub rates: Vec<PlanSplitRate>,
    /// Soft warnings the agent should weigh — empty when no warnings.
    pub warnings: Vec<String>,
    /// Plan expiry timestamp (ISO 8601).
    pub expires_at: String,
    /// Seconds until expiry (60).
    pub expires_in_seconds: i32,
}

/// Response from [`SplitsApi::execute`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSplitResult {
    /// Task ID. Poll `client.tasks().get(...)` for lifecycle updates
    /// (`Created` → `QuoteFetched` → `DepositTxBroadcast` → ... →
    /// `Completed` / `Failed` / `Refunding`).
    pub task_id: String,
    /// Upstream SplitNOW order ID (6-character code; visible in SplitNOW's
    /// dashboard).
    pub splitnow_order_id: String,
}

// ── API methods ─────────────────────────────────────────────────────────────

/// SplitNOW cross-chain split endpoints — `client.splits()`.
pub struct SplitsApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl SplitsApi<'_> {
    /// Step 1: get a SplitNOW quote and reserve a 60-second plan slot.
    /// No funds move. Requires `split:plan` scope.
    pub async fn plan(&self, body: &PlanSplitBody) -> Result<PlanSplitResult, ShurikenError> {
        self.0.post("/api/v2/splits/plan", body).await
    }

    /// Step 2: execute a previously-planned split. Returns a `task_id`;
    /// poll `client.tasks().get(...)` for status. Requires `split:execute`
    /// scope.
    pub async fn execute(
        &self,
        body: &ExecuteSplitBody,
    ) -> Result<ExecuteSplitResult, ShurikenError> {
        self.0.post("/api/v2/splits/execute", body).await
    }
}
