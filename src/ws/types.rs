use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Session bootstrap types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionFilter {
    pub stream: String,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub provider: String,
    pub app_key: String,
    pub ws_host: String,
    pub ws_port: u16,
    pub force_tls: bool,
    pub auth_endpoint: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub recommended_reconnect_backoff_ms: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSubscription {
    pub stream: String,
    pub channel: String,
    pub event: String,
    pub visibility: String,
    pub payload_format: String,
    pub payload_schema_id: String,
    pub payload_schema_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub connection: ConnectionInfo,
    pub session: SessionInfo,
    pub subscriptions: Vec<ResolvedSubscription>,
}

// ── Connection state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ConnectionStateEvent {
    pub state: ConnectionState,
    pub reason: Option<String>,
}

// ── Pusher protocol messages ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct PusherMessage {
    pub event: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PusherSubscribe {
    pub event: &'static str,
    pub data: PusherSubscribeData,
}

#[derive(Debug, Serialize)]
pub(crate) struct PusherSubscribeData {
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PusherConnectionEstablished {
    pub socket_id: String,
}
