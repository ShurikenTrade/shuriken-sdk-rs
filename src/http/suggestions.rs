//! Agent-posted trade suggestions against `/api/v2/agents/suggestions/*`.
//!
//! Agents post advisory trade ideas to the user; users ack (execute) or
//! dismiss them from the terminal / tg-bot. The lifecycle states
//! (`OPEN | ACTED | DISMISSED | EXPIRED`) are derived server-side from the
//! three nullable timestamps (`actedAt`, `dismissedAt`, `expiresAt`).
//!
//! Scopes:
//! - `write:suggestions` — `create`
//! - `read:suggestions` — `list`
//! - `manage:suggestions` — `ack` / `dismiss`

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Enums ───────────────────────────────────────────────────────────────────

/// Trade direction. Serialised as the proto enum name (`BUY` / `SELL`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SuggestionSide {
    Buy,
    Sell,
}

/// Derived suggestion lifecycle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SuggestionState {
    Open,
    Acted,
    Dismissed,
    Expired,
}

/// Optional confidence hint posted by the agent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SuggestionConfidence {
    Low,
    Medium,
    High,
}

// ── Response types ──────────────────────────────────────────────────────────

/// Lightweight handle for the agent key that posted the suggestion.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSuggestionAgentKey {
    /// Agent key ID.
    pub id: String,
    /// Display name for the agent key. May be absent on legacy rows.
    #[serde(default)]
    pub name: Option<String>,
}

/// Asset metadata enriched onto the response — consumers don't have to call
/// `tokens().get(...)` separately.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSuggestionAsset {
    /// Token address on the suggestion's network.
    pub address: String,
    /// Token symbol.
    pub symbol: String,
    /// Token name.
    pub name: String,
    /// Most recent USD price, if known.
    #[serde(default)]
    pub price_usd: Option<f64>,
}

/// Outbound suggestion shape returned by `create` / `list` / `dismiss` / `ack`.
///
/// The `state` field is derived; the timestamps remain the source of truth.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSuggestion {
    /// Suggestion ID (cuid).
    pub id: String,
    /// Derived lifecycle.
    pub state: SuggestionState,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 expiry timestamp.
    pub expires_at: String,
    /// ISO 8601 timestamp when the user acked the suggestion, if any.
    #[serde(default)]
    pub acted_at: Option<String>,
    /// ISO 8601 timestamp when the user dismissed the suggestion, if any.
    #[serde(default)]
    pub dismissed_at: Option<String>,
    /// Free-form reason supplied with `dismiss`, if any.
    #[serde(default)]
    pub dismiss_reason: Option<String>,
    /// Task ID linked at `ack` time, if any.
    #[serde(default)]
    pub linked_task_id: Option<String>,
    /// Trade direction.
    pub side: SuggestionSide,
    /// `common.NetworkId` proto enum name (e.g. `SOL`, `BASE`, `BSC`, `MONAD`).
    pub network_id: String,
    /// Asset metadata.
    pub asset: TradeSuggestionAsset,
    /// Why the agent suggested this trade. ≤500 chars.
    pub rationale: String,
    /// Optional sizing hint in USD.
    #[serde(default)]
    pub amount_in_usd: Option<f64>,
    /// Optional confidence hint.
    #[serde(default)]
    pub confidence: Option<SuggestionConfidence>,
    /// Agent key that posted the suggestion.
    pub agent_key: TradeSuggestionAgentKey,
}

/// Response from [`SuggestionsApi::list`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSuggestionsResponse {
    /// Suggestions ordered most-recent first.
    pub suggestions: Vec<TradeSuggestion>,
    /// Cursor for the next page, if any.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

// ── Request body / query types ──────────────────────────────────────────────

/// Body for [`SuggestionsApi::create`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSuggestionRequest {
    /// Trade direction.
    pub side: SuggestionSide,
    /// `common.NetworkId` enum name (`SOL`, `BASE`, `BSC`, `MONAD`, …).
    pub network_id: String,
    /// Token address. Direction is implicit from `side`.
    pub asset: String,
    /// Why you're suggesting this trade. ≤500 chars.
    pub rationale: String,
    /// Optional sizing hint in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_in_usd: Option<f64>,
    /// Optional confidence: `LOW`, `MEDIUM`, or `HIGH`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<SuggestionConfidence>,
}

/// Query parameters for [`SuggestionsApi::list`].
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSuggestionsQuery {
    /// Filter by state, or `ALL` for everything. Defaults to `OPEN` server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Page size, 1..=200. Defaults to 50 server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Opaque next-page cursor returned by a previous call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct DismissBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AckBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    linked_task_id: Option<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

/// Agent-posted trade suggestions — `client.suggestions()`.
pub struct SuggestionsApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl SuggestionsApi<'_> {
    /// Post a new trade suggestion. Requires the `write:suggestions` scope.
    pub async fn create(
        &self,
        req: &CreateSuggestionRequest,
    ) -> Result<TradeSuggestion, ShurikenError> {
        self.0.post("/api/v2/agents/suggestions", req).await
    }

    /// List suggestions for the authenticated user. With no query, the server
    /// defaults to `state=OPEN` and `limit=50`.
    pub async fn list(
        &self,
        query: Option<&ListSuggestionsQuery>,
    ) -> Result<ListSuggestionsResponse, ShurikenError> {
        let mut q: Vec<(&str, String)> = Vec::new();
        if let Some(query) = query {
            if let Some(state) = &query.state {
                q.push(("state", state.clone()));
            }
            if let Some(limit) = query.limit {
                q.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = &query.cursor {
                q.push(("cursor", cursor.clone()));
            }
        }
        self.0
            .get_with_query("/api/v2/agents/suggestions", &q)
            .await
    }

    /// Dismiss an OPEN suggestion with an optional free-form reason.
    ///
    /// Returns `409 SUGGESTION_NOT_OPEN` if the suggestion is already acted /
    /// dismissed / expired.
    pub async fn dismiss(
        &self,
        id: &str,
        reason: Option<String>,
    ) -> Result<TradeSuggestion, ShurikenError> {
        let body = DismissBody { reason };
        self.0
            .post(&format!("/api/v2/agents/suggestions/{id}/dismiss"), &body)
            .await
    }

    /// Ack (mark as acted) an OPEN suggestion. Optionally link a task ID to
    /// record which execution flow honored the suggestion.
    ///
    /// Returns `409 SUGGESTION_NOT_OPEN` if the suggestion is already acted /
    /// dismissed / expired.
    pub async fn ack(
        &self,
        id: &str,
        linked_task_id: Option<String>,
    ) -> Result<TradeSuggestion, ShurikenError> {
        let body = AckBody { linked_task_id };
        self.0
            .post(&format!("/api/v2/agents/suggestions/{id}/ack"), &body)
            .await
    }
}
