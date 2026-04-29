//! Wallet group CRUD against `/api/v2/wallet-groups/*`.
//!
//! Each endpoint accepts EITHER the legacy `read:wallets` / `write:wallets`
//! scope OR the focused `manage:wallet-groups` scope. Treasury-management
//! keys can be issued with `manage:wallet-groups` only — full group CRUD
//! without granting the broader wallet-address read surface.
//!
//! Naming: this module exports `WalletGroupRecord` rather than `WalletGroup`
//! because `crate::http::account::WalletGroup` already names a legacy
//! in-settings reference shape (carried inside `TradeSettings.walletGroups`).
//! `WalletGroupRecord` is the canonical entity returned by the v2 REST surface.

use serde::{Deserialize, Serialize};

use super::ShurikenHttpClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

/// A user-defined named group of wallets on a single chain.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletGroupRecord {
    /// Group ID (cuid).
    pub group_id: String,
    /// Group name (1-32 chars).
    pub name: String,
    /// Chain string. `svm` | `base` | `bsc` for the generate pathway; arbitrary for others.
    pub chain: Option<String>,
    /// Member wallet IDs in display order.
    pub wallet_ids: Vec<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last-update timestamp.
    pub updated_at: String,
}

/// Response from [`WalletGroupsApi::delete`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWalletGroupResponse {
    /// ID of the group that was deleted. Idempotent — also returned if the
    /// group was already gone.
    pub group_id: String,
}

// ── Request body types ──────────────────────────────────────────────────────

/// Body for [`WalletGroupsApi::create`].
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletGroupBody {
    /// Group name (1-32 chars).
    pub name: String,
    /// Chain. Required when `wallet_ids` is non-empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Optional initial members. Wallets must already exist on `chain`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_ids: Option<Vec<String>>,
}

/// Body for [`WalletGroupsApi::create_with_wallets`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletGroupWithWalletsBody {
    /// Group name (1-32 chars). Wallets are auto-named `<name> 1..N`.
    pub name: String,
    /// Chain. `svm` | `base` | `bsc` only.
    pub chain: String,
    /// Number of fresh wallets to create. 1..=16.
    pub wallet_count: u32,
}

/// Body for [`WalletGroupsApi::update`].
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWalletGroupBody {
    /// New name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Body for [`WalletGroupsApi::add_wallets`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWalletsToGroupBody {
    /// Wallet IDs to add. Must exist on the group's chain.
    pub wallet_ids: Vec<String>,
    /// Insert at this position; appends if absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

/// Body for [`WalletGroupsApi::remove_wallets`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveWalletsFromGroupBody {
    /// Wallet IDs to remove. Wallets are not deleted.
    pub wallet_ids: Vec<String>,
}

/// Body for [`WalletGroupsApi::reorder_wallets`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderWalletsInGroupBody {
    /// Complete ordered list — must match current membership exactly.
    pub wallet_ids: Vec<String>,
}

/// Body for [`WalletGroupsApi::move_wallet`].
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MoveWalletBody {
    /// Source group, or `None`/`null` to detach.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_group_id: Option<String>,
    /// Destination group, or `None`/`null` to detach.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_group_id: Option<String>,
}

// ── API methods ─────────────────────────────────────────────────────────────

/// Wallet-group management endpoints — `client.wallet_groups()`.
pub struct WalletGroupsApi<'a>(pub(crate) &'a ShurikenHttpClient);

impl WalletGroupsApi<'_> {
    /// List wallet groups for the authenticated user. Optional chain filter.
    pub async fn list(&self, chain: Option<&str>) -> Result<Vec<WalletGroupRecord>, ShurikenError> {
        #[derive(Deserialize)]
        struct Wrapper {
            groups: Vec<WalletGroupRecord>,
        }
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(c) = chain {
            query.push(("chain", c.to_string()));
        }
        let wrapper: Wrapper = self
            .0
            .get_with_query("/api/v2/wallet-groups", &query)
            .await?;
        Ok(wrapper.groups)
    }

    /// Get a single group by ID.
    pub async fn get(&self, group_id: &str) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .get(&format!("/api/v2/wallet-groups/{group_id}"))
            .await
    }

    /// Create an empty (or pre-populated) group.
    pub async fn create(
        &self,
        body: &CreateWalletGroupBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0.post("/api/v2/wallet-groups", body).await
    }

    /// Atomically create N fresh wallets and a new group containing them.
    /// Single transaction — no orphan wallets on partial failure.
    pub async fn create_with_wallets(
        &self,
        body: &CreateWalletGroupWithWalletsBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .post("/api/v2/wallet-groups/with-wallets", body)
            .await
    }

    /// Rename an existing group.
    pub async fn update(
        &self,
        group_id: &str,
        body: &UpdateWalletGroupBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .patch(&format!("/api/v2/wallet-groups/{group_id}"), body)
            .await
    }

    /// Delete a group. Idempotent — member wallets are not affected.
    pub async fn delete(&self, group_id: &str) -> Result<DeleteWalletGroupResponse, ShurikenError> {
        self.0
            .delete(&format!("/api/v2/wallet-groups/{group_id}"))
            .await
    }

    /// Add existing wallets to a group.
    pub async fn add_wallets(
        &self,
        group_id: &str,
        body: &AddWalletsToGroupBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .post(&format!("/api/v2/wallet-groups/{group_id}/wallets"), body)
            .await
    }

    /// Remove wallets from a group. Wallets themselves stay; only membership
    /// is removed.
    pub async fn remove_wallets(
        &self,
        group_id: &str,
        body: &RemoveWalletsFromGroupBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .delete_with_body(&format!("/api/v2/wallet-groups/{group_id}/wallets"), body)
            .await
    }

    /// Reorder a group's wallets. Provided list must match current membership
    /// exactly.
    pub async fn reorder_wallets(
        &self,
        group_id: &str,
        body: &ReorderWalletsInGroupBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .put(
                &format!("/api/v2/wallet-groups/{group_id}/wallets/order"),
                body,
            )
            .await
    }

    /// Move a wallet between groups (or detach from a group). Returns the
    /// destination group's new state.
    pub async fn move_wallet(
        &self,
        wallet_id: &str,
        body: &MoveWalletBody,
    ) -> Result<WalletGroupRecord, ShurikenError> {
        self.0
            .post(&format!("/api/v2/wallets/{wallet_id}/move"), body)
            .await
    }
}
