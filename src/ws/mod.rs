mod connection;
pub mod streams;
pub mod subscription;
mod types;

pub use streams::{IntoFilterMap, StreamDef};
pub use subscription::Subscription;
pub use types::{
    ConnectionInfo, ResolvedSubscription, SessionInfo, SessionResponse, SubscriptionFilter,
};

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, warn};

use crate::error::ShurikenError;

use connection::{
    dispatch, expand_session, fetch_session, transport_subscribe, ActiveSubscription, WsSink,
};
use types::*;

const DEFAULT_BASE_URL: &str = "https://api.shuriken.trade";

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

// ── ShurikenWsClient ───────────────────────────────────────────────────────

pub struct ShurikenWsClient {
    http: Client,
    base_url: String,
    session: Option<SessionResponse>,
    socket_id: Option<String>,
    sink: Arc<Mutex<Option<WsSink>>>,
    subscriptions: Arc<Mutex<Vec<ActiveSubscription>>>,
    state: ConnectionState,
    /// Senders for state-change subscribers. The event loop also holds a clone
    /// of this list so it can broadcast disconnection/failure events.
    state_subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<Result<ConnectionStateEvent, ShurikenError>>>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    unsub_tx: mpsc::UnboundedSender<usize>,
    unsub_rx: Option<mpsc::UnboundedReceiver<usize>>,
    next_sub_id: usize,
}

impl ShurikenWsClient {
    pub fn new(api_key: &str) -> Result<Self, ShurikenError> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self, ShurikenError> {
        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|e| ShurikenError::Auth(e.to_string()))?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);

        let http = Client::builder().default_headers(headers).build()?;
        let base_url = base_url.trim_end_matches('/').to_string();

        let (unsub_tx, unsub_rx) = mpsc::unbounded_channel();

        Ok(Self {
            http,
            base_url,
            session: None,
            socket_id: None,
            sink: Default::default(),
            subscriptions: Default::default(),
            state: ConnectionState::Disconnected,
            state_subscribers: Default::default(),
            shutdown_tx: None,
            unsub_tx,
            unsub_rx: Some(unsub_rx),
            next_sub_id: 0,
        })
    }

    pub async fn connect(&mut self) -> Result<(), ShurikenError> {
        if self.state == ConnectionState::Connected || self.state == ConnectionState::Connecting {
            return Err(ShurikenError::Session("Already connected".into()));
        }

        self.set_state(ConnectionState::Connecting, None).await;

        // Bootstrap with a default subscription to get connection info
        let session = fetch_session(
            &self.http,
            &self.base_url,
            &[SubscriptionFilter {
                stream: "alpha.signalFeedGlobal".into(),
                filter: HashMap::new(),
            }],
        )
        .await?;

        let conn = &session.connection;
        let scheme = if conn.force_tls { "wss" } else { "ws" };
        let url = format!(
            "{scheme}://{}:{}/app/{}?protocol=7&client=shuriken-sdk-rs&version=0.2.0",
            conn.ws_host, conn.ws_port, conn.app_key,
        );

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| ShurikenError::Session(format!("WebSocket connect failed: {e}")))?;

        let (sink, mut stream) = ws_stream.split();
        *self.sink.lock().await = Some(sink);
        self.session = Some(session);

        // Wait for connection_established from real-time transport
        let socket_id = loop {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(msg) = serde_json::from_str::<TransportMessage>(&text) {
                        if msg.event == "pusher:connection_established" {
                            if let Some(data) = &msg.data {
                                let data_str = match data {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string(),
                                };
                                let established: TransportConnectionEstablished =
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

        self.socket_id = Some(socket_id);
        self.set_state(ConnectionState::Connected, None).await;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let subscriptions = Arc::clone(&self.subscriptions);
        let state_subscribers = Arc::clone(&self.state_subscribers);
        let mut unsub_rx = self.unsub_rx.take();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(m) = serde_json::from_str::<TransportMessage>(&text) {
                                    dispatch(&subscriptions, m).await;
                                }
                            }
                            Some(Ok(Message::Ping(_))) => {
                                debug!("WebSocket ping");
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                warn!("WebSocket closed");
                                broadcast_state(
                                    &state_subscribers,
                                    ConnectionState::Disconnected,
                                    Some("Connection closed".into()),
                                ).await;
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {e}");
                                broadcast_state(
                                    &state_subscribers,
                                    ConnectionState::Failed,
                                    Some(e.to_string()),
                                ).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("WebSocket shutdown requested");
                        break;
                    }
                    Some(sub_id) = async {
                        match unsub_rx.as_mut() {
                            Some(rx) => rx.recv().await,
                            None => std::future::pending::<Option<usize>>().await,
                        }
                    } => {
                        let mut subs = subscriptions.lock().await;
                        subs.retain(|s| s.id != sub_id);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(mut sink) = self.sink.lock().await.take() {
            let _ = futures_util::SinkExt::close(&mut sink).await;
        }
        self.subscriptions.lock().await.clear();
        self.session = None;
        self.socket_id = None;
        self.set_state(ConnectionState::Disconnected, None).await;
    }

    pub async fn subscribe<P, F>(
        &mut self,
        stream: StreamDef<P, F>,
        filter: F,
    ) -> Result<Subscription<P>, ShurikenError>
    where
        P: DeserializeOwned + Send + 'static,
        F: IntoFilterMap,
    {
        self.subscribe_inner(stream.name, filter.into_filter_map())
            .await
    }

    pub async fn subscribe_raw(
        &mut self,
        stream: &str,
        filter: HashMap<String, String>,
    ) -> Result<Subscription<serde_json::Value>, ShurikenError> {
        if self.state != ConnectionState::Connected {
            return Err(ShurikenError::Session(
                "Not connected. Call connect() first.".into(),
            ));
        }

        let sub_filter = SubscriptionFilter {
            stream: stream.to_string(),
            filter,
        };

        let (raw_tx, raw_rx) = mpsc::unbounded_channel::<Result<serde_json::Value, ShurikenError>>();
        let sub_id = self.next_sub_id;
        self.next_sub_id += 1;

        self.register_subscription(sub_id, raw_tx, sub_filter)
            .await?;

        Ok(Subscription {
            rx: raw_rx,
            id: sub_id,
            unsub_tx: self.unsub_tx.clone(),
        })
    }

    pub async fn on_state_change(&mut self) -> Subscription<ConnectionStateEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let sub_id = self.next_sub_id;
        self.next_sub_id += 1;

        self.state_subscribers.lock().await.push(tx);

        Subscription {
            rx,
            id: sub_id,
            unsub_tx: self.unsub_tx.clone(),
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn session(&self) -> Option<&SessionResponse> {
        self.session.as_ref()
    }

    // ── Internal ────────────────────────────────────────────────────────────

    async fn set_state(&mut self, new_state: ConnectionState, reason: Option<String>) {
        self.state = new_state;
        broadcast_state(&self.state_subscribers, new_state, reason).await;
    }

    async fn register_subscription(
        &mut self,
        sub_id: usize,
        raw_tx: mpsc::UnboundedSender<Result<serde_json::Value, ShurikenError>>,
        sub_filter: SubscriptionFilter,
    ) -> Result<(), ShurikenError> {
        let stream_name = sub_filter.stream.clone();
        let resolved = self.session.as_ref().and_then(|s| {
            s.subscriptions
                .iter()
                .find(|r| r.stream == stream_name)
                .cloned()
        });

        if let Some(resolved) = resolved {
            transport_subscribe(
                &self.http,
                &self.base_url,
                &self.session,
                &self.socket_id,
                &self.sink,
                &resolved.channel,
                &resolved.visibility,
            )
            .await?;
            self.subscriptions.lock().await.push(ActiveSubscription {
                id: sub_id,
                channel: resolved.channel.clone(),
                event: resolved.event.clone(),
                tx: raw_tx,
                filter: sub_filter,
                resolved: Some(resolved),
            });
        } else {
            self.subscriptions.lock().await.push(ActiveSubscription {
                id: sub_id,
                channel: String::new(),
                event: String::new(),
                tx: raw_tx,
                filter: sub_filter.clone(),
                resolved: None,
            });
            expand_session(
                &self.http,
                &self.base_url,
                &mut self.session,
                &self.socket_id,
                &self.sink,
                &self.subscriptions,
                &[sub_filter],
            )
            .await?;
        }
        Ok(())
    }

    async fn subscribe_inner<P>(
        &mut self,
        stream_name: &str,
        filter: HashMap<String, String>,
    ) -> Result<Subscription<P>, ShurikenError>
    where
        P: DeserializeOwned + Send + 'static,
    {
        if self.state != ConnectionState::Connected {
            return Err(ShurikenError::Session(
                "Not connected. Call connect() first.".into(),
            ));
        }

        let sub_filter = SubscriptionFilter {
            stream: stream_name.to_string(),
            filter,
        };

        let (raw_tx, mut raw_rx) =
            mpsc::unbounded_channel::<Result<serde_json::Value, ShurikenError>>();
        let sub_id = self.next_sub_id;
        self.next_sub_id += 1;

        self.register_subscription(sub_id, raw_tx, sub_filter)
            .await?;

        // Spawn a deserialization bridge: Result<Value> -> Result<P>
        let (typed_tx, typed_rx) = mpsc::unbounded_channel::<Result<P, ShurikenError>>();
        tokio::spawn(async move {
            while let Some(result) = raw_rx.recv().await {
                let mapped = match result {
                    Ok(val) => serde_json::from_value::<P>(val).map_err(ShurikenError::from),
                    Err(e) => Err(e),
                };
                if typed_tx.send(mapped).is_err() {
                    break;
                }
            }
        });

        Ok(Subscription {
            rx: typed_rx,
            id: sub_id,
            unsub_tx: self.unsub_tx.clone(),
        })
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

async fn broadcast_state(
    subscribers: &Mutex<Vec<mpsc::UnboundedSender<Result<ConnectionStateEvent, ShurikenError>>>>,
    state: ConnectionState,
    reason: Option<String>,
) {
    let mut subs = subscribers.lock().await;
    let event = ConnectionStateEvent { state, reason };
    subs.retain(|tx| tx.send(Ok(event.clone())).is_ok());
}
