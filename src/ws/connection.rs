use std::sync::Arc;

use futures_util::SinkExt;
use reqwest::Client;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use crate::error::ShurikenError;

use super::types::*;

pub(crate) type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

pub(crate) struct ActiveSubscription {
    pub id: usize,
    pub channel: String,
    pub event: String,
    pub tx: mpsc::UnboundedSender<serde_json::Value>,
    pub filter: SubscriptionFilter,
    pub resolved: Option<ResolvedSubscription>,
}

// ── HTTP helpers ────────────────────────────────────────────────────────────

pub(crate) async fn http_post(
    http: &Client,
    base_url: &str,
    path: &str,
    body: &impl serde::Serialize,
) -> Result<serde_json::Value, ShurikenError> {
    let url = format!("{base_url}{path}");
    let resp = http.post(&url).json(body).send().await?;
    let status = resp.status();
    let request_id = resp
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ShurikenError::Auth(resp.text().await.unwrap_or_default()));
    }
    if !status.is_success() {
        return Err(ShurikenError::Api {
            status: status.as_u16(),
            message: resp.text().await.unwrap_or_default(),
            request_id,
        });
    }
    Ok(resp.json().await?)
}

// ── Session management ──────────────────────────────────────────────────────

pub(crate) async fn fetch_session(
    http: &Client,
    base_url: &str,
    filters: &[SubscriptionFilter],
) -> Result<SessionResponse, ShurikenError> {
    let value = http_post(
        http,
        base_url,
        "/api/v2/ws/session",
        &serde_json::json!({ "subscriptions": filters }),
    )
    .await?;
    serde_json::from_value(value).map_err(ShurikenError::from)
}

pub(crate) async fn expand_session(
    http: &Client,
    base_url: &str,
    session: &mut Option<SessionResponse>,
    socket_id: &Option<String>,
    sink: &Arc<Mutex<Option<WsSink>>>,
    subscriptions: &Mutex<Vec<ActiveSubscription>>,
    new_filters: &[SubscriptionFilter],
) -> Result<(), ShurikenError> {
    let all_filters = {
        let subs = subscriptions.lock().await;
        let mut filters: Vec<SubscriptionFilter> = subs.iter().map(|s| s.filter.clone()).collect();
        for f in new_filters {
            let key = (&f.stream, &f.filter);
            if !filters.iter().any(|e| (&e.stream, &e.filter) == key) {
                filters.push(f.clone());
            }
        }
        filters
    };

    let new_session = fetch_session(http, base_url, &all_filters).await?;
    *session = Some(new_session.clone());

    let mut subs = subscriptions.lock().await;
    for sub in subs.iter_mut() {
        if sub.resolved.is_some() {
            continue;
        }
        if let Some(resolved) = new_session
            .subscriptions
            .iter()
            .find(|r| r.stream == sub.filter.stream)
        {
            sub.channel = resolved.channel.clone();
            sub.event = resolved.event.clone();
            sub.resolved = Some(resolved.clone());
            if let Err(e) = pusher_subscribe(
                http,
                base_url,
                session,
                socket_id,
                sink,
                &resolved.channel,
                &resolved.visibility,
            )
            .await
            {
                warn!("Failed to subscribe to channel {}: {e}", resolved.channel);
            }
        }
    }
    Ok(())
}

// ── Pusher protocol ─────────────────────────────────────────────────────────

pub(crate) async fn pusher_subscribe(
    http: &Client,
    base_url: &str,
    session: &Option<SessionResponse>,
    socket_id: &Option<String>,
    sink: &Arc<Mutex<Option<WsSink>>>,
    channel: &str,
    visibility: &str,
) -> Result<(), ShurikenError> {
    let auth = if visibility == "presence"
        || channel.starts_with("private-")
        || channel.starts_with("presence-")
    {
        let sid = socket_id
            .as_ref()
            .ok_or_else(|| ShurikenError::Session("No socket_id".into()))?;
        let auth_endpoint = session
            .as_ref()
            .map(|s| s.connection.auth_endpoint.clone())
            .ok_or_else(|| ShurikenError::Session("No session".into()))?;
        let value = http_post(
            http,
            base_url,
            &auth_endpoint,
            &serde_json::json!({
                "socket_id": sid,
                "channel_name": channel,
            }),
        )
        .await?;
        Some(
            value["auth"]
                .as_str()
                .ok_or_else(|| ShurikenError::Session("Missing auth in response".into()))?
                .to_string(),
        )
    } else {
        None
    };

    let msg = serde_json::to_string(&PusherSubscribe {
        event: "pusher:subscribe",
        data: PusherSubscribeData {
            channel: channel.to_string(),
            auth,
            channel_data: None,
        },
    })
    .map_err(|e| ShurikenError::Session(format!("Serialize error: {e}")))?;

    if let Some(sink) = sink.lock().await.as_mut() {
        sink.send(Message::Text(msg.into()))
            .await
            .map_err(|e| ShurikenError::Session(format!("Send error: {e}")))?;
    }
    Ok(())
}

// ── Dispatch ────────────────────────────────────────────────────────────────

pub(crate) async fn dispatch(subscriptions: &Mutex<Vec<ActiveSubscription>>, msg: PusherMessage) {
    if msg.event.starts_with("pusher:") || msg.event.starts_with("pusher_internal:") {
        return;
    }
    let Some(channel) = &msg.channel else { return };
    let data = match &msg.data {
        Some(serde_json::Value::String(s)) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::String(s.clone()))
        }
        Some(v) => v.clone(),
        None => return,
    };
    let subs = subscriptions.lock().await;
    for sub in subs.iter() {
        if sub.channel == *channel && sub.event == msg.event {
            let _ = sub.tx.send(data.clone());
        }
    }
}
