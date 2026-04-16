pub mod api;
mod client;
mod error;

#[cfg(feature = "ws")]
pub mod ws;

pub use client::ShurikenClient;
pub use error::ShurikenError;

pub use shuriken_api_types as types;

pub use api::account;
pub use api::perps;
pub use api::portfolio;
pub use api::swap;
pub use api::tokens;
pub use api::trigger;
