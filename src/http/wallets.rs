//! Wallet archive lifecycle against `/api/v2/wallets/*`.
//!
//! Soft-archive and restore individual or bulk sets of user wallets. All
//! endpoints require the `write:wallets` scope. Archived wallets are excluded
//! from default-wallet pickers and live-balance polling but remain on file —
//! `unarchive` restores them.
//!
//! Naming: this module exports `WalletRecord` rather than `Wallet` to avoid
//! collision with potential `Wallet`-named types elsewhere in the SDK and to
//! mirror the convention used by `crate::http::wallet_groups::WalletGroupRecord`.

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

/// A user wallet as returned by the archive lifecycle endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletRecord {
    /// Shuriken wallet ID.
    pub wallet_id: String,
    /// On-chain address.
    pub address: String,
    /// Chain (`solana` | `base` | `bsc` | `sui` | `ton` | `aptos`), or `None`
    /// for multi-chain wallets.
    pub chain: Option<String>,
    /// User-assigned label, or `None` if not set.
    pub label: Option<String>,
    /// Lifecycle state — `"ACTIVE"` or `"ARCHIVED"`.
    pub state: String,
    /// ISO 8601 timestamp the wallet was archived. `None` while `state == "ACTIVE"`.
    pub archived_at: Option<String>,
}

/// Response from [`WalletsApi::archive`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResponse {
    /// The wallet in its post-archive state.
    pub wallet: WalletRecord,
    /// `true` when the archived wallet was the user's default wallet on its
    /// chain and the default has been cleared as part of archiving.
    pub cleared_default: bool,
}

/// Response from [`WalletsApi::unarchive`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnarchiveResponse {
    /// The wallet in its post-unarchive state.
    pub wallet: WalletRecord,
}

/// Per-wallet outcome from [`WalletsApi::bulk_archive`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkArchiveEntry {
    /// Shuriken wallet ID.
    pub wallet_id: String,
    /// Outcome — `"archived"` or `"already_archived"`.
    pub status: String,
    /// `Some(true)` only when the wallet was cleared as the user's default;
    /// `None` otherwise.
    pub cleared_default: Option<bool>,
}

/// Response from [`WalletsApi::bulk_archive`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkArchiveResponse {
    /// One entry per wallet ID submitted, in submission order.
    pub results: Vec<BulkArchiveEntry>,
}

// ── Request body types ──────────────────────────────────────────────────────

/// Body for [`WalletsApi::bulk_archive`]. Maximum 100 IDs per request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkArchiveRequest {
    /// Wallet IDs to archive.
    pub wallet_ids: Vec<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

/// Wallet archive lifecycle endpoints — `client.wallets()`.
pub struct WalletsApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl WalletsApi<'_> {
    /// Soft-archive a single wallet.
    ///
    /// Returns the wallet in its post-archive state plus `cleared_default`
    /// flagging whether the user's default-wallet pointer was cleared.
    /// `404 WALLET_NOT_FOUND` if the ID doesn't belong to the user.
    pub async fn archive(&self, wallet_id: &str) -> Result<ArchiveResponse, ShurikenError> {
        self.0
            .post(
                &format!("/api/v2/wallets/{wallet_id}/archive"),
                &serde_json::json!({}),
            )
            .await
    }

    /// Restore a previously-archived wallet.
    ///
    /// `404 WALLET_NOT_FOUND` if the ID doesn't belong to the user.
    pub async fn unarchive(&self, wallet_id: &str) -> Result<UnarchiveResponse, ShurikenError> {
        self.0
            .post(
                &format!("/api/v2/wallets/{wallet_id}/unarchive"),
                &serde_json::json!({}),
            )
            .await
    }

    /// Archive up to 100 wallets in one request. Each wallet is processed
    /// independently; the response carries one entry per submitted ID with
    /// per-wallet status (`archived` / `already_archived`).
    pub async fn bulk_archive(
        &self,
        body: &BulkArchiveRequest,
    ) -> Result<BulkArchiveResponse, ShurikenError> {
        self.0.post("/api/v2/wallets/bulk-archive", body).await
    }
}
