use shuriken_api_types::error::ApiErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShurikenError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("api error (status {status}, {code}): {message}", code = .response.error.code, message = .response.error.message)]
    Api {
        status: u16,
        response: ApiErrorResponse,
    },

    #[error("session error: {0}")]
    Session(String),

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("deserialization failed: {0}")]
    Decode(#[from] serde_json::Error),
}
