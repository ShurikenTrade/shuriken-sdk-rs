use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::ShurikenClient;
use crate::error::ShurikenError;

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub user_id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWallet {
    pub wallet_id: String,
    pub address: String,
    pub chain: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SwapPreset {
    #[serde(rename = "solana")]
    Solana {
        slippage_bps: u32,
        mev_protection_enabled: bool,
        custom_priority_fee_sol: Option<String>,
        bribe_amount_sol: Option<String>,
        max_price_impact_pct: Option<f64>,
    },
    #[serde(rename = "evm")]
    Evm {
        slippage_bps: u32,
        mev_protection_enabled: bool,
        max_price_impact_pct: Option<f64>,
        max_priority_fee_per_gas_gwei: Option<String>,
        bribe_amount_native: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainPresets {
    pub auto: SwapPreset,
    pub p1: SwapPreset,
    pub p2: SwapPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletGroup {
    pub id: String,
    pub name: String,
    pub wallet_ids: Vec<String>,
    pub network_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneClickModeSettings {
    pub enabled: bool,
    pub buy_presets: Vec<String>,
    pub sell_presets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedWallets {
    pub wallet_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultWallets {
    pub default_wallet_by_network: HashMap<String, String>,
    pub selected_wallet_ids_by_network: HashMap<String, SelectedWallets>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSettings {
    pub auto_enable_multisend: bool,
    pub chain_presets_buy: HashMap<String, ChainPresets>,
    pub chain_presets_sell: HashMap<String, ChainPresets>,
    pub default_wallets: DefaultWallets,
    pub one_click_mode: HashMap<String, OneClickModeSettings>,
    pub wallet_groups: Vec<WalletGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSettings {
    pub trade_settings: TradeSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKeyConstraints {
    pub buys_enabled: bool,
    pub sells_enabled: bool,
    pub max_executions_per_hour: u32,
    pub max_executions_per_day: u32,
    pub max_concurrent_executions: u32,
    pub max_limit_orders_per_day: u32,
    pub allow_custom_gas: bool,
    pub allow_bribes: bool,
    pub allowed_networks: Vec<u32>,
    pub allowed_wallet_ids: Vec<String>,
    pub max_buy_usd_per_trade: Option<f64>,
    pub max_buy_usd_per_day: Option<f64>,
    pub max_sell_usd_per_trade: Option<f64>,
    pub max_sell_usd_per_day: Option<f64>,
    pub max_limit_order_usd_per_order: Option<f64>,
    pub max_slippage_bps: Option<u32>,
    pub max_price_impact_pct: Option<f64>,
    pub max_sell_position_pct: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUsage {
    pub key_id: String,
    pub scopes: Vec<String>,
    pub constraints: AgentKeyConstraints,
}

// ── API methods ─────────────────────────────────────────────────────────────

impl ShurikenClient {
    pub async fn get_me(&self) -> Result<AccountInfo, ShurikenError> {
        self.get("/api/v2/account/me").await
    }

    pub async fn get_settings(&self) -> Result<AccountSettings, ShurikenError> {
        self.get("/api/v2/account/settings").await
    }

    pub async fn update_settings(
        &self,
        settings: &AccountSettings,
    ) -> Result<AccountSettings, ShurikenError> {
        self.put("/api/v2/account/settings", settings).await
    }

    pub async fn get_usage(&self) -> Result<AccountUsage, ShurikenError> {
        self.get("/api/v2/account/usage").await
    }

    pub async fn get_wallets(&self) -> Result<Vec<AccountWallet>, ShurikenError> {
        self.get("/api/v2/account/wallets").await
    }
}
