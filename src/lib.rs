mod error;
pub mod http;

#[cfg(feature = "ws")]
pub mod ws;

pub use error::ShurikenError;
pub use http::ShurikenHttpClient;

#[cfg(feature = "ws")]
pub use ws::streams;
#[cfg(feature = "ws")]
pub use ws::subscription::Subscription;
#[cfg(feature = "ws")]
pub use ws::ShurikenWsClient;
#[cfg(feature = "ws")]
pub use ws::{ConnectionState, ConnectionStateEvent};

pub use shuriken_api_types as types;

pub use http::account;
pub use http::alpha;
pub use http::perps;
pub use http::portfolio;
pub use http::splits;
pub use http::suggestions;
pub use http::swap;
pub use http::tasks;
pub use http::tokens;
pub use http::transfers;
pub use http::trigger;
pub use http::wallet_groups;
pub use http::wallets;
