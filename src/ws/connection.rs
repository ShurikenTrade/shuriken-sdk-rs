use std::sync::Arc;

use futures_util::SinkExt;
use reqwest::Client;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

use shuriken_api_types::error::ApiErrorResponse;

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
    pub tx: mpsc::UnboundedSender<Result<serde_json::Value, ShurikenError>>,
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
    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("{base_url}{path}")
    };
    let resp = http.post(&url).json(body).send().await?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ShurikenError::Auth(resp.text().await.unwrap_or_default()));
    }
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let response: ApiErrorResponse = serde_json::from_str(&text)?;
        return Err(ShurikenError::Api {
            status: status.as_u16(),
            response,
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
            transport_subscribe(
                http,
                base_url,
                session,
                socket_id,
                sink,
                &resolved.channel,
                &resolved.visibility,
            )
            .await?;
        }
    }
    Ok(())
}

// ── Transport protocol ──────────────────────────────────────────────────────

pub(crate) async fn transport_subscribe(
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

    let msg = serde_json::to_string(&TransportSubscribe {
        event: "pusher:subscribe",
        data: TransportSubscribeData {
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

pub(crate) async fn dispatch(
    subscriptions: &Mutex<Vec<ActiveSubscription>>,
    msg: TransportMessage,
) {
    if msg.event == "pusher:subscription_error" {
        let Some(channel) = &msg.channel else { return };
        let detail = msg
            .data
            .as_ref()
            .and_then(|d| d.as_str())
            .unwrap_or("subscription failed");
        let err = ShurikenError::Session(format!(
            "Channel subscription rejected for \"{channel}\": {detail}"
        ));
        let subs = subscriptions.lock().await;
        for sub in subs.iter() {
            if sub.channel == *channel {
                let _ = sub.tx.send(Err(ShurikenError::Session(err.to_string())));
            }
        }
        return;
    }
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
            let _ = sub.tx.send(Ok(data.clone()));
        }
    }
}
