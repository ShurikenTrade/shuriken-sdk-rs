//! Alpha signal endpoints against `/api/v2/alpha/*`.
//!
//! Exposes your connected alpha sources (Discord, Telegram, X/Twitter) and
//! the token-level call history they generate. Use these endpoints to build
//! signal-aware trading agents.
//!
//! Scope: `read:alpha` (or the legacy `read:all`) on the agent key.

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Response DTOs ────────────────────────────────────────────────────────────

/// A single alpha source connection (Discord server, Telegram group, X list,
/// or on-chain wallet tracker).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaSourceItem {
    /// Stable ID for the connection.
    pub connection_id: String,
    /// Wire type: `"discord"` | `"telegram"` | `"twitter"` | `"onchain"`.
    pub connection_type: String,
    /// Whether the connection is currently active.
    pub enabled: bool,
    /// ISO 8601 creation timestamp. May be absent on legacy rows.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,
    /// Platform string, if different from `connection_type`.
    #[serde(default)]
    pub platform: Option<String>,
    /// Source identifier (e.g. Discord server ID).
    #[serde(default)]
    pub source: Option<String>,
}

/// Response from [`AlphaApi::get_sources`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaSourcesResult {
    pub sources: Vec<AlphaSourceItem>,
}

/// The most recent source that mentioned a token in a recent call.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentCallSourceItem {
    pub platform: String,
    #[serde(default)]
    pub channel_name: Option<String>,
    #[serde(default)]
    pub author_username: Option<String>,
    #[serde(default)]
    pub message_preview: Option<String>,
    #[serde(default)]
    pub source_name: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
}

/// A token call entry returned by `GET /api/v2/alpha/recent-calls`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentCallItem {
    pub token_address: String,
    #[serde(default)]
    pub token_symbol: Option<String>,
    #[serde(default)]
    pub token_name: Option<String>,
    pub chain: String,
    /// Unix timestamp (seconds) of the first mention.
    pub first_seen_at: i64,
    /// Unix timestamp (seconds) of the most recent mention.
    pub last_seen_at: i64,
    pub mention_count: u32,
    #[serde(default)]
    pub price_usd_at_call: Option<String>,
    #[serde(default)]
    pub current_price_usd: Option<String>,
    #[serde(default)]
    pub market_cap_usd_at_call: Option<String>,
    #[serde(default)]
    pub liquidity_usd_at_call: Option<String>,
    #[serde(default)]
    pub last_source: Option<RecentCallSourceItem>,
}

/// Response from [`AlphaApi::get_recent_calls`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentCallsResult {
    pub total_count: u32,
    pub calls: Vec<RecentCallItem>,
}

/// A token call entry returned by `GET /api/v2/alpha/global-calls`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalCallItem {
    pub token_address: String,
    #[serde(default)]
    pub token_symbol: Option<String>,
    #[serde(default)]
    pub token_name: Option<String>,
    pub chain: String,
    /// Unix timestamp (seconds) of the first mention.
    pub first_seen_at: i64,
    /// Unix timestamp (seconds) of the most recent mention.
    pub last_seen_at: i64,
    pub mention_count: u32,
    #[serde(default)]
    pub current_price_usd: Option<String>,
    #[serde(default)]
    pub price_change_since_call_pct: Option<String>,
    #[serde(default)]
    pub last_tweet_author: Option<String>,
    #[serde(default)]
    pub last_tweet_preview: Option<String>,
}

/// Response from [`AlphaApi::get_global_calls`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalCallsResult {
    pub platform: String,
    pub total_count: u32,
    pub calls: Vec<GlobalCallItem>,
}

/// The caller identity on a signal (X/Discord/Telegram).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaCallCaller {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub verified: Option<bool>,
}

/// Source-level detail on a signal (which server / channel / tweet).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaCallSourceDetail {
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub channel_name: Option<String>,
    #[serde(default)]
    pub topic_id: Option<i32>,
    #[serde(default)]
    pub topic_title: Option<String>,
    #[serde(default)]
    pub tweet_id: Option<String>,
    #[serde(default)]
    pub message_id: Option<String>,
}

/// On-chain trade data attached to wallet-tracker signals.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaTradeData {
    pub is_buy: bool,
    pub amount_usd: String,
    pub amount_native: String,
    pub wallet_address: String,
    pub tx_signature: String,
}

/// A single surrounding message included when `include_message_context` is set.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMessage {
    #[serde(default)]
    pub author: Option<String>,
    pub text: String,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: i64,
    /// Position relative to the signal message (negative = before, positive = after).
    pub offset: i32,
}

/// A single alpha call signal for a token.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaCallSignalItem {
    pub signal_id: String,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: i64,
    /// `"discord"` | `"telegram"` | `"twitter"` | `"onchain"`.
    pub platform: String,
    pub is_bot: bool,
    #[serde(default)]
    pub price_usd: Option<f64>,
    #[serde(default)]
    pub market_cap_usd: Option<f64>,
    #[serde(default)]
    pub liquidity_usd: Option<f64>,
    #[serde(default)]
    pub caller: Option<AlphaCallCaller>,
    #[serde(default)]
    pub source: Option<AlphaCallSourceDetail>,
    #[serde(default)]
    pub trade_data: Option<AlphaTradeData>,
    /// Absent for X / on-chain signals.
    #[serde(default)]
    pub message_preview: Option<String>,
    /// Absent unless `include_message_context` was set.
    #[serde(default)]
    pub context_messages: Option<Vec<ContextMessage>>,
}

/// Response from [`AlphaApi::get_call_context`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallContextResult {
    pub token_address: String,
    pub total_signals: i32,
    pub has_more: bool,
    #[serde(default)]
    pub next_cursor: Option<i64>,
    pub signals: Vec<AlphaCallSignalItem>,
}

/// A single mention record for a token.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMentionItem {
    pub message_id: String,
    pub platform: String,
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub author_username: Option<String>,
    #[serde(default)]
    pub price_usd_at_mention: Option<String>,
    #[serde(default)]
    pub market_cap_usd_at_mention: Option<String>,
}

/// Response from [`AlphaApi::get_token_mentions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMentionsResult {
    pub token_address: String,
    #[serde(default)]
    pub token_symbol: Option<String>,
    pub chain: String,
    pub total_mentions: u32,
    #[serde(default)]
    pub first_seen_at: Option<i64>,
    #[serde(default)]
    pub last_seen_at: Option<i64>,
    pub mentions: Vec<TokenMentionItem>,
}

// ── Request param structs ────────────────────────────────────────────────────

/// Query parameters for [`AlphaApi::get_recent_calls`].
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecentCallsParams {
    /// Max results to return. Server default applies when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Filter to calls from a specific source name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    /// Filter to calls from a specific connection ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

/// Query parameters for [`AlphaApi::get_global_calls`].
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGlobalCallsParams {
    /// Platform to query (`"twitter"` | `"discord"` | …). Server applies a default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Max results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Query parameters for [`AlphaApi::get_call_context`].
///
/// `source_filter` is a list of platform strings (e.g. `["discord", "telegram"]`)
/// that is serialised on the wire as the comma-separated query param
/// `sourceFilter=discord,telegram`. Pass an empty `Vec` or `None` to omit it.
#[derive(Debug, Default, Clone)]
pub struct GetCallContextParams {
    /// Max signals to return.
    pub limit: Option<u32>,
    /// Opaque cursor for pagination (value of `next_cursor` from a previous call).
    pub cursor: Option<i64>,
    /// Filter to specific platforms. Joined with commas on the wire.
    pub source_filter: Option<Vec<String>>,
    /// Include bot / on-chain signals. Defaults to `false` server-side.
    pub include_bot_signals: Option<bool>,
    /// Include surrounding context messages. Defaults to `false` server-side.
    pub include_message_context: Option<bool>,
}

/// Query parameters for [`AlphaApi::get_token_mentions`].
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenMentionsParams {
    /// Max mentions to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// ── API methods ──────────────────────────────────────────────────────────────

/// Alpha signal endpoints — `client.alpha()`.
pub struct AlphaApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl AlphaApi<'_> {
    /// List all alpha source connections for the authenticated user.
    ///
    /// Returns a flat list; connection types vary (`discord`, `telegram`,
    /// `twitter`, `onchain`). Use `enabled` to filter active connections.
    pub async fn get_sources(&self) -> Result<AlphaSourcesResult, ShurikenError> {
        self.0.get("/api/v2/alpha/sources").await
    }

    /// List token calls from your personal alpha connections, most-recent first.
    pub async fn get_recent_calls(
        &self,
        params: GetRecentCallsParams,
    ) -> Result<RecentCallsResult, ShurikenError> {
        let mut q: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(source_name) = &params.source_name {
            q.push(("sourceName", source_name.clone()));
        }
        if let Some(connection_id) = &params.connection_id {
            q.push(("connectionId", connection_id.clone()));
        }
        self.0
            .get_with_query("/api/v2/alpha/recent-calls", &q)
            .await
    }

    /// List token calls aggregated across the global platform feed.
    pub async fn get_global_calls(
        &self,
        params: GetGlobalCallsParams,
    ) -> Result<GlobalCallsResult, ShurikenError> {
        let mut q: Vec<(&str, String)> = Vec::new();
        if let Some(platform) = &params.platform {
            q.push(("platform", platform.clone()));
        }
        if let Some(limit) = params.limit {
            q.push(("limit", limit.to_string()));
        }
        self.0
            .get_with_query("/api/v2/alpha/global-calls", &q)
            .await
    }

    /// Get the individual call signals for a specific token.
    ///
    /// Supports cursor-based pagination via `params.cursor` /
    /// `result.next_cursor`. Set `include_bot_signals` to also return
    /// on-chain wallet-tracker signals. Set `include_message_context` to
    /// attach the surrounding chat messages to each signal.
    pub async fn get_call_context(
        &self,
        token_address: &str,
        params: GetCallContextParams,
    ) -> Result<CallContextResult, ShurikenError> {
        let mut q: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = params.cursor {
            q.push(("cursor", cursor.to_string()));
        }
        if let Some(source_filter) = &params.source_filter {
            if !source_filter.is_empty() {
                q.push(("sourceFilter", source_filter.join(",")));
            }
        }
        if let Some(include_bot_signals) = params.include_bot_signals {
            q.push(("includeBotSignals", include_bot_signals.to_string()));
        }
        if let Some(include_message_context) = params.include_message_context {
            q.push(("includeMessageContext", include_message_context.to_string()));
        }
        let path = format!("/api/v2/alpha/tokens/{token_address}/call-context");
        self.0.get_with_query(&path, &q).await
    }

    /// Get all individual mention records for a specific token.
    pub async fn get_token_mentions(
        &self,
        token_address: &str,
        params: GetTokenMentionsParams,
    ) -> Result<TokenMentionsResult, ShurikenError> {
        let mut q: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            q.push(("limit", limit.to_string()));
        }
        let path = format!("/api/v2/alpha/tokens/{token_address}/mentions");
        self.0.get_with_query(&path, &q).await
    }
}
