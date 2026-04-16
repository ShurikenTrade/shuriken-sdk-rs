use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShurikenError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("api error (status {status}): {message}")]
    Api {
        status: u16,
        message: String,
        request_id: Option<String>,
    },

    #[error("session error: {0}")]
    Session(String),

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("deserialization failed: {0}")]
    Decode(#[from] serde_json::Error),
}
