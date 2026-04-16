use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, warn};

use crate::error::ShurikenError;

use super::types::*;

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

type EventHandler = Box<dyn Fn(serde_json::Value) + Send + Sync>;
type StateHandler = Box<dyn Fn(ConnectionStateEvent) + Send + Sync>;

struct ActiveSubscription {
    channel: String,
    event: String,
    handler: EventHandler,
    filter: SubscriptionFilter,
    resolved: Option<ResolvedSubscription>,
}

pub struct WsHandle {
    http: Client,
    base_url: String,
    session: Arc<RwLock<Option<SessionResponse>>>,
    socket_id: Arc<RwLock<Option<String>>>,
    sink: Arc<Mutex<Option<WsSink>>>,
    subscriptions: Arc<Mutex<Vec<ActiveSubscription>>>,
    state_handlers: Arc<Mutex<Vec<StateHandler>>>,
    state: Arc<RwLock<ConnectionState>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl WsHandle {
    pub(crate) fn new(http: Client, base_url: String) -> Self {
        Self {
            http,
            base_url,
            session: Default::default(),
            socket_id: Default::default(),
            sink: Default::default(),
            subscriptions: Default::default(),
            state_handlers: Default::default(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            shutdown_tx: Default::default(),
        }
    }

    pub async fn connect(&self) -> Result<(), ShurikenError> {
        {
            let state = *self.state.read().await;
            if state == ConnectionState::Connected || state == ConnectionState::Connecting {
                return Err(ShurikenError::Session("Already connected".into()));
            }
        }

        self.emit_state(ConnectionState::Connecting, None).await;

        let session = self
            .fetch_session(&[SubscriptionFilter {
                stream: "alpha.signalFeedGlobal".into(),
                filter: HashMap::new(),
            }])
            .await?;

        let conn = &session.connection;
        let scheme = if conn.force_tls { "wss" } else { "ws" };
        let url = format!(
            "{scheme}://{}:{}/app/{}?protocol=7&client=shuriken-sdk-rs&version=0.1.0",
            conn.ws_host, conn.ws_port, conn.app_key,
        );

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| ShurikenError::Session(format!("WebSocket connect failed: {e}")))?;

        let (sink, mut stream) = ws_stream.split();
        *self.sink.lock().await = Some(sink);
        *self.session.write().await = Some(session);

        // Wait for pusher:connection_established
        let socket_id = loop {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(msg) = serde_json::from_str::<PusherMessage>(&text) {
                        if msg.event == "pusher:connection_established" {
                            if let Some(data) = &msg.data {
                                let data_str = match data {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string(),
                                };
                                let established: PusherConnectionEstablished =
                                    serde_json::from_str(&data_str).map_err(|e| {
                                        ShurikenError::Session(format!(
                                            "Failed to parse connection_established: {e}"
                                        ))
                                    })?;
                                break established.socket_id;
                            }
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => {
                    return Err(ShurikenError::Session(
                        "Connection closed before established".into(),
                    ));
                }
                Some(Err(e)) => {
                    return Err(ShurikenError::Session(format!("WebSocket error: {e}")));
                }
                _ => continue,
            }
        };

        *self.socket_id.write().await = Some(socket_id);
        self.emit_state(ConnectionState::Connected, None).await;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let subscriptions = Arc::clone(&self.subscriptions);
        let state = Arc::clone(&self.state);
        let state_handlers = Arc::clone(&self.state_handlers);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(m) = serde_json::from_str::<PusherMessage>(&text) {
                                    dispatch(&subscriptions, m).await;
                                }
                            }
                            Some(Ok(Message::Ping(_))) => {
                                debug!("WebSocket ping");
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                warn!("WebSocket closed");
                                emit(&state, &state_handlers, ConnectionState::Disconnected, Some("Connection closed".into())).await;
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {e}");
                                emit(&state, &state_handlers, ConnectionState::Failed, Some(e.to_string())).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("WebSocket shutdown");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(()).await;
        }
        if let Some(mut sink) = self.sink.lock().await.take() {
            let _ = sink.close().await;
        }
        self.subscriptions.lock().await.clear();
        *self.session.write().await = None;
        *self.socket_id.write().await = None;
        self.emit_state(ConnectionState::Disconnected, None).await;
    }

    pub async fn subscribe<F>(
        &self,
        stream: &str,
        filter: HashMap<String, String>,
        handler: F,
    ) -> Result<usize, ShurikenError>
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(ShurikenError::Session(
                "Not connected. Call connect() first.".into(),
            ));
        }

        let sub_filter = SubscriptionFilter {
            stream: stream.to_string(),
            filter,
        };

        let resolved = {
            let s = self.session.read().await;
            s.as_ref()
                .and_then(|s| s.subscriptions.iter().find(|r| r.stream == stream).cloned())
        };

        if let Some(resolved) = resolved {
            self.pusher_subscribe(&resolved.channel, &resolved.visibility)
                .await?;
            let mut subs = self.subscriptions.lock().await;
            let id = subs.len();
            subs.push(ActiveSubscription {
                channel: resolved.channel.clone(),
                event: resolved.event.clone(),
                handler: Box::new(handler),
                filter: sub_filter,
                resolved: Some(resolved),
            });
            Ok(id)
        } else {
            let mut subs = self.subscriptions.lock().await;
            let id = subs.len();
            subs.push(ActiveSubscription {
                channel: String::new(),
                event: String::new(),
                handler: Box::new(handler),
                filter: sub_filter.clone(),
                resolved: None,
            });
            drop(subs);
            self.expand_session(&[sub_filter]).await?;
            Ok(id)
        }
    }

    pub async fn unsubscribe(&self, sub_id: usize) {
        let mut subs = self.subscriptions.lock().await;
        if sub_id < subs.len() {
            subs.remove(sub_id);
        }
    }

    pub async fn on_state_change<F>(&self, handler: F)
    where
        F: Fn(ConnectionStateEvent) + Send + Sync + 'static,
    {
        self.state_handlers.lock().await.push(Box::new(handler));
    }

    pub async fn session(&self) -> Option<SessionResponse> {
        self.session.read().await.clone()
    }

    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    // ── Internal ────────────────────────────────────────────────────────────

    async fn http_post(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, ShurikenError> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.post(&url).json(body).send().await?;
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

    async fn fetch_session(
        &self,
        filters: &[SubscriptionFilter],
    ) -> Result<SessionResponse, ShurikenError> {
        let value = self
            .http_post(
                "/api/v2/ws/session",
                &serde_json::json!({ "subscriptions": filters }),
            )
            .await?;
        serde_json::from_value(value).map_err(ShurikenError::from)
    }

    async fn expand_session(
        &self,
        new_filters: &[SubscriptionFilter],
    ) -> Result<(), ShurikenError> {
        let all_filters = {
            let subs = self.subscriptions.lock().await;
            let mut filters: Vec<SubscriptionFilter> =
                subs.iter().map(|s| s.filter.clone()).collect();
            for f in new_filters {
                let key = (&f.stream, &f.filter);
                if !filters.iter().any(|e| (&e.stream, &e.filter) == key) {
                    filters.push(f.clone());
                }
            }
            filters
        };

        let new_session = self.fetch_session(&all_filters).await?;
        *self.session.write().await = Some(new_session.clone());

        let mut subs = self.subscriptions.lock().await;
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
                if let Err(e) = self
                    .pusher_subscribe(&resolved.channel, &resolved.visibility)
                    .await
                {
                    warn!("Failed to subscribe to channel {}: {e}", resolved.channel);
                }
            }
        }
        Ok(())
    }

    async fn pusher_subscribe(&self, channel: &str, visibility: &str) -> Result<(), ShurikenError> {
        let auth = if visibility == "presence"
            || channel.starts_with("private-")
            || channel.starts_with("presence-")
        {
            let socket_id = self
                .socket_id
                .read()
                .await
                .clone()
                .ok_or_else(|| ShurikenError::Session("No socket_id".into()))?;
            let auth_endpoint = self
                .session
                .read()
                .await
                .as_ref()
                .map(|s| s.connection.auth_endpoint.clone())
                .ok_or_else(|| ShurikenError::Session("No session".into()))?;
            let value = self
                .http_post(
                    &auth_endpoint,
                    &serde_json::json!({
                        "socket_id": socket_id,
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

        if let Some(sink) = self.sink.lock().await.as_mut() {
            sink.send(Message::Text(msg.into()))
                .await
                .map_err(|e| ShurikenError::Session(format!("Send error: {e}")))?;
        }
        Ok(())
    }

    async fn emit_state(&self, new_state: ConnectionState, reason: Option<String>) {
        emit(&self.state, &self.state_handlers, new_state, reason).await;
    }
}

async fn emit(
    state: &RwLock<ConnectionState>,
    handlers: &Mutex<Vec<StateHandler>>,
    new_state: ConnectionState,
    reason: Option<String>,
) {
    *state.write().await = new_state;
    let handlers = handlers.lock().await;
    let event = ConnectionStateEvent {
        state: new_state,
        reason,
    };
    for h in handlers.iter() {
        h(event.clone());
    }
}

async fn dispatch(subscriptions: &Mutex<Vec<ActiveSubscription>>, msg: PusherMessage) {
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
            (sub.handler)(data.clone());
        }
    }
}
