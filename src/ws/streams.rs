use std::collections::HashMap;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use shuriken_api_types::{alpha, analytics, automation, evm, notification, svm, wallet};

// ---------------------------------------------------------------------------
// IntoFilterMap trait
// ---------------------------------------------------------------------------

pub trait IntoFilterMap {
    fn into_filter_map(self) -> HashMap<String, String>;
}

// ---------------------------------------------------------------------------
// StreamDef<P, F>
// ---------------------------------------------------------------------------

pub struct StreamDef<P: DeserializeOwned, F: IntoFilterMap> {
    pub name: &'static str,
    _phantom: PhantomData<fn() -> (P, F)>,
}

impl<P: DeserializeOwned, F: IntoFilterMap> StreamDef<P, F> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }
}

impl<P: DeserializeOwned, F: IntoFilterMap> Clone for StreamDef<P, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: DeserializeOwned, F: IntoFilterMap> Copy for StreamDef<P, F> {}

// ---------------------------------------------------------------------------
// Filter types
// ---------------------------------------------------------------------------

pub struct NoFilter;

impl IntoFilterMap for NoFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub struct SvmTokenFilter {
    pub token_address: String,
}

impl IntoFilterMap for SvmTokenFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("tokenAddress".into(), self.token_address);
        m
    }
}

pub struct SvmWalletFilter {
    pub wallet_address: String,
}

impl IntoFilterMap for SvmWalletFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("walletAddress".into(), self.wallet_address);
        m
    }
}

pub struct EvmTokenFilter {
    pub chain_id: String,
    pub token_address: String,
}

impl IntoFilterMap for EvmTokenFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("chainId".into(), self.chain_id);
        m.insert("tokenAddress".into(), self.token_address);
        m
    }
}

pub struct EvmWalletFilter {
    pub wallet_address: String,
}

impl IntoFilterMap for EvmWalletFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("walletAddress".into(), self.wallet_address);
        m
    }
}

pub struct AlphaProfileFilter {
    pub profile_id: String,
}

impl IntoFilterMap for AlphaProfileFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("profileId".into(), self.profile_id);
        m
    }
}

pub struct AlphaNamedFeedFilter {
    pub feed_id: String,
}

impl IntoFilterMap for AlphaNamedFeedFilter {
    fn into_filter_map(self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("feedId".into(), self.feed_id);
        m
    }
}

// ---------------------------------------------------------------------------
// SVM streams (9)
// ---------------------------------------------------------------------------

pub const SVM_TOKEN_SWAPS: StreamDef<svm::SwapEvent, SvmTokenFilter> =
    StreamDef::new("svm.token.swaps");

pub const SVM_TOKEN_POOL_INFO: StreamDef<svm::TokenPoolEvent, SvmTokenFilter> =
    StreamDef::new("svm.token.poolInfo");

pub const SVM_TOKEN_BALANCES: StreamDef<svm::TokenBalanceEvent, SvmTokenFilter> =
    StreamDef::new("svm.token.balances");

pub const SVM_TOKEN_DISTRIBUTION_STATS: StreamDef<
    analytics::TokenDistributionStatsEvent,
    SvmTokenFilter,
> = StreamDef::new("svm.token.distributionStats");

pub const SVM_TOKEN_HOLDER_STATS: StreamDef<analytics::HolderStatsEvent, SvmTokenFilter> =
    StreamDef::new("svm.token.holderStats");

pub const SVM_WALLET_NATIVE_BALANCE: StreamDef<wallet::SvmNativeBalanceEvent, SvmWalletFilter> =
    StreamDef::new("svm.wallet.nativeBalance");

pub const SVM_WALLET_TOKEN_BALANCES: StreamDef<wallet::SvmTokenBalanceEvent, SvmWalletFilter> =
    StreamDef::new("svm.wallet.tokenBalances");

pub const SVM_BONDING_CURVE_CREATIONS: StreamDef<svm::BondingCurveCreationEvent, NoFilter> =
    StreamDef::new("svm.bondingCurve.creations");

pub const SVM_BONDING_CURVE_GRADUATIONS: StreamDef<svm::BondingCurveGraduationEvent, NoFilter> =
    StreamDef::new("svm.bondingCurve.graduations");

// ---------------------------------------------------------------------------
// EVM streams (5)
// ---------------------------------------------------------------------------

pub const EVM_TOKEN_SWAPS: StreamDef<evm::SwapEvent, EvmTokenFilter> =
    StreamDef::new("evm.token.swaps");

pub const EVM_TOKEN_POOL_INFO: StreamDef<evm::TokenPoolEvent, EvmTokenFilter> =
    StreamDef::new("evm.token.poolInfo");

pub const EVM_TOKEN_BALANCES: StreamDef<evm::TokenBalanceEvent, EvmTokenFilter> =
    StreamDef::new("evm.token.balances");

pub const EVM_WALLET_NATIVE_BALANCE: StreamDef<wallet::EvmNativeBalanceEvent, EvmWalletFilter> =
    StreamDef::new("evm.wallet.nativeBalance");

pub const EVM_WALLET_TOKEN_BALANCES: StreamDef<evm::TokenBalanceEvent, EvmWalletFilter> =
    StreamDef::new("evm.wallet.tokenBalances");

// ---------------------------------------------------------------------------
// Alpha streams (5)
// ---------------------------------------------------------------------------

pub const ALPHA_SIGNAL_FEED_GLOBAL: StreamDef<alpha::SignalFeedUpdateEvent, NoFilter> =
    StreamDef::new("alpha.signalFeedGlobal");

pub const ALPHA_SIGNAL_FEED_PERSONAL: StreamDef<alpha::SignalFeedUpdateEvent, NoFilter> =
    StreamDef::new("alpha.signalFeedPersonal");

pub const ALPHA_SIGNAL_FEED_PROFILE: StreamDef<alpha::SignalFeedUpdateEvent, AlphaProfileFilter> =
    StreamDef::new("alpha.signalFeedProfile");

pub const ALPHA_SIGNAL_FEED_NAMED: StreamDef<alpha::SignalFeedUpdateEvent, AlphaNamedFeedFilter> =
    StreamDef::new("alpha.signalFeedNamed");

pub const ALPHA_PERSONAL: StreamDef<alpha::MessageEvent, NoFilter> =
    StreamDef::new("alpha.personal");

// ---------------------------------------------------------------------------
// Portfolio / Automation streams (2)
// ---------------------------------------------------------------------------

pub const PORTFOLIO_NOTIFICATIONS: StreamDef<notification::NotificationEvent, NoFilter> =
    StreamDef::new("portfolio.notifications");

pub const AUTOMATION_UPDATES: StreamDef<automation::AutomationEvent, NoFilter> =
    StreamDef::new("automation.updates");
