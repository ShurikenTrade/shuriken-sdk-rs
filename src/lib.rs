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
pub use http::perps;
pub use http::portfolio;
pub use http::swap;
pub use http::tokens;
pub use http::trigger;
