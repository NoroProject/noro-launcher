//! WebSocket client for the master, with reconnect and backoff.

use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use schema::{ClientWsMsg, ServerWsMsg};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct WsClient {
    pub out: UnboundedSender<ClientWsMsg>,
    token: Arc<RwLock<Option<String>>>,
}

impl WsClient {
    pub fn send(&self, msg: ClientWsMsg) {
        let _ = self.out.send(msg);
    }

    /// The session token, held in one place for every consumer. The case panel
    /// shares this session with the socket; a second copy would be one more
    /// thing to keep out of the game's JVM.
    pub fn token(&self) -> Option<String> {
        self.token.read().clone()
    }

    /// Login and logout both come through here. The next connection sends
    /// `Authenticate` on its own; this covers the connection already open.
    pub fn set_token(&self, token: Option<String>) {
        *self.token.write() = token.clone();
        if let Some(t) = token {
            let _ = self.out.send(authenticate(t));
        }
    }
}

/// `inbound` receives messages from the master; `conn_state` gets a bool every
/// time the connection comes up or goes down.
pub fn spawn(
    ws_url: String,
    initial_token: Option<String>,
    inbound: UnboundedSender<ServerWsMsg>,
    conn_state: UnboundedSender<bool>,
) -> WsClient {
    let (out_tx, out_rx) = mpsc::unbounded_channel::<ClientWsMsg>();
    let token = Arc::new(RwLock::new(initial_token));

    let client = WsClient {
        out: out_tx,
        token: token.clone(),
    };

    tokio::spawn(connection_loop(ws_url, token, out_rx, inbound, conn_state));
    client
}

async fn connection_loop(
    ws_url: String,
    token: Arc<RwLock<Option<String>>>,
    mut out_rx: UnboundedReceiver<ClientWsMsg>,
    inbound: UnboundedSender<ServerWsMsg>,
    conn_state: UnboundedSender<bool>,
) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                backoff = Duration::from_secs(1);
                let _ = conn_state.send(true);
                let (mut sink, mut stream) = ws_stream.split();

                // Clone out of the lock first — the guard isn't Send and must
                // not be held across the await below.
                let auth_token = token.read().clone();
                if let Some(t) = auth_token {
                    let _ = sink.send(Message::Text(authenticate(t).to_json())).await;
                }

                loop {
                    tokio::select! {
                        out = out_rx.recv() => {
                            match out {
                                Some(msg) => {
                                    if sink.send(Message::Text(msg.to_json())).await.is_err() {
                                        break;
                                    }
                                }
                                None => return, // channel closed: the backend is shutting down
                            }
                        }
                        incoming = stream.next() => {
                            match incoming {
                                Some(Ok(Message::Text(t))) => {
                                    if let Ok(msg) = serde_json::from_str::<ServerWsMsg>(&t) {
                                        let _ = inbound.send(msg);
                                    }
                                }
                                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                                _ => {}
                            }
                        }
                    }
                }
                let _ = conn_state.send(false);
            }
            Err(e) => {
                tracing::debug!("could not connect to the master: {e}");
                let _ = conn_state.send(false);
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }
}

/// Version and platform ride along with the auth message so the admin panel can
/// see who is still on an old launcher without asking separately.
fn authenticate(access_token: String) -> ClientWsMsg {
    ClientWsMsg::Authenticate {
        access_token,
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    }
}
