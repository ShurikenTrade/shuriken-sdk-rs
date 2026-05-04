//! Wallet-to-wallet transfers against `/api/v2/transfers/*`.
//!
//! Two endpoints sharing one wire shape (`TransferResult`):
//!
//! - `send`: wallet-to-wallet transfer of native or token amount.
//! - `retire_wallet`: drain the source's full balance of one token to the
//!   destination, then archive the source on terminal success.
//!
//! Both endpoints accept SVM (Solana) and EVM (Base 8453, BSC 56) chains.
//! Idempotency is auto-derived from `from + to + token + amount + chain` in
//! 5-minute buckets unless the caller supplies `correlation_id`. Setting
//! `await_result: false` returns immediately with a `task_id`; otherwise
//! the call blocks until the task reaches a terminal state.
//!
//! All endpoints require the `transfer:write` scope.

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Request body types ──────────────────────────────────────────────────────

/// Body for [`TransfersApi::send`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBody {
    /// Source wallet ID.
    pub from_wallet_id: String,
    /// Destination wallet ID.
    pub to_wallet_id: String,
    /// Token symbol (`SOL`, `ETH`, `BNB`, `USDC`), raw mint/contract address,
    /// or `"native"` for the chain's native asset.
    pub token: String,
    /// Raw base units as a decimal string, e.g. `"1000000"` for 1 USDC at
    /// 6 decimals or `"1000000000"` for 1 SOL.
    pub amount: String,
    /// `"SVM"` or `"EVM"`.
    pub chain: String,
    /// Required when `chain == "EVM"`. `8453` = Base, `56` = BSC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    /// Block until the task reaches a terminal state. Defaults to `true`
    /// server-side when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub await_result: Option<bool>,
    /// Caller-supplied idempotency key. Omit to auto-derive (5-minute bucket).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Optional free-form note attached to the activity feed entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_comment: Option<String>,
}

/// Body for [`TransfersApi::retire_wallet`].
///
/// Drains `token` from `from_wallet_id` to `to_wallet_id` and archives the
/// source on terminal success. For native amounts the executor reserves a
/// gas/rent buffer; an `INSUFFICIENT_BALANCE_FOR_GAS` error fires when the
/// balance net of buffer is non-positive.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetireWalletBody {
    /// Source wallet ID — drained, then archived on terminal success.
    pub from_wallet_id: String,
    /// Destination wallet ID.
    pub to_wallet_id: String,
    /// Token symbol, contract/mint address, or `"native"`.
    pub token: String,
    /// `"SVM"` or `"EVM"`.
    pub chain: String,
    /// Required when `chain == "EVM"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    /// Block until terminal state. Defaults to `true` server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub await_result: Option<bool>,
    /// Caller-supplied idempotency key. Omit to auto-derive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Optional free-form note for the activity feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_comment: Option<String>,
}

// ── Response types ──────────────────────────────────────────────────────────

/// Settled transaction info, populated only when the call succeeded under
/// `await_result: true`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferTransactionInfo {
    /// Transaction signature (SVM) or hash (EVM).
    pub hash: String,
    /// Explorer URL, or `None` on chains without a configured explorer.
    pub explorer_url: Option<String>,
}

/// Settled error info, populated when the call reached a terminal failure.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferErrorInfo {
    /// Error code, e.g. `"INSUFFICIENT_BALANCE_FOR_GAS"`.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

/// Wire-format response shared by [`TransfersApi::send`] and
/// [`TransfersApi::retire_wallet`].
///
/// `transaction` is populated when `await_result: true` and the task
/// succeeded; `error` is populated when it failed. Non-blocking calls
/// (`await_result: false`) return neither — poll
/// `client.tasks().get(&task_id)` for status.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    /// Server-issued task ID. Use with `client.tasks().get(...)` for polling.
    pub task_id: String,
    /// `"PENDING"` | `"SUCCESS"` | `"FAILED"`.
    pub status: String,
    /// `true` when the source wallet will be archived on terminal success
    /// (always `true` for `retire_wallet`, `false` for `send`).
    pub will_archive_on_success: bool,
    /// Settled transaction details. Only present on `status == "SUCCESS"`
    /// under `await_result: true`.
    pub transaction: Option<TransferTransactionInfo>,
    /// Settled failure details. Only present on `status == "FAILED"`.
    pub error: Option<TransferErrorInfo>,
}

// ── API methods ─────────────────────────────────────────────────────────────

/// Wallet-to-wallet transfer endpoints — `client.transfers()`.
pub struct TransfersApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl TransfersApi<'_> {
    /// Send a fixed amount of `token` from one wallet to another.
    pub async fn send(&self, body: &SendBody) -> Result<TransferResult, ShurikenError> {
        self.0.post("/api/v2/transfers/send", body).await
    }

    /// Drain the source wallet's full balance of `token` to the destination,
    /// then archive the source on terminal success.
    pub async fn retire_wallet(
        &self,
        body: &RetireWalletBody,
    ) -> Result<TransferResult, ShurikenError> {
        self.0.post("/api/v2/transfers/retire-wallet", body).await
    }
}
