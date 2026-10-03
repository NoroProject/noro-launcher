//! WebSocket client for the master, with reconnect and backoff.

use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use schema::{ClientWsMsg, ServerWsMsg};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

/// A handshake that hasn't finished by now isn't going to.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How often we ping. NAT mappings and proxies drop idle connections after a
/// minute or so; a ping keeps them alive and proves the socket still works.
const PING_EVERY: Duration = Duration::from_secs(20);
/// Nothing at all from the master for this long — not even a pong — and the
/// socket is dead, whatever TCP thinks. After a laptop sleeps the connection
/// otherwise looks online for many minutes while every push is lost.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
/// A connection has to last this long before the backoff resets. A master
/// that accepts and immediately closes would otherwise be hit every second by
/// every launcher at once.
const STABLE_AFTER: Duration = Duration::from_secs(30);
const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

enum Outbound {
    Msg(ClientWsMsg),
    /// Drop the current connection and open a new one. Used on logout: the
    /// open socket is still authenticated as the previous player, and their
    /// pushes would keep arriving.
    Reconnect,
}

#[derive(Clone)]
pub struct WsClient {
    out: UnboundedSender<Outbound>,
    token: Arc<RwLock<Option<String>>>,
}

impl WsClient {
    pub fn send(&self, msg: ClientWsMsg) {
        let _ = self.out.send(Outbound::Msg(msg));
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
        let _ = match token {
            Some(t) => self.out.send(Outbound::Msg(authenticate(t))),
            None => self.out.send(Outbound::Reconnect),
        };
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
    let (out_tx, out_rx) = mpsc::unbounded_channel::<Outbound>();
    let token = Arc::new(RwLock::new(initial_token));

    let client = WsClient {
        out: out_tx,
        token: token.clone(),
    };

    tokio::spawn(connection_loop(ws_url, token, out_rx, inbound, conn_state));
    client
}

/// Why a connection ended.
enum Ended {
    /// The backend is shutting down.
    Shutdown,
    Dropped,
}

async fn connection_loop(
    ws_url: String,
    token: Arc<RwLock<Option<String>>>,
    mut out_rx: UnboundedReceiver<Outbound>,
    inbound: UnboundedSender<ServerWsMsg>,
    conn_state: UnboundedSender<bool>,
) {
    let mut backoff = MIN_BACKOFF;
    // A message taken off the queue whose send failed. It goes out first on
    // the next connection instead of being lost — a game-stop report, say.
    let mut unsent: Option<ClientWsMsg> = None;

    loop {
        let connected =
            tokio::time::timeout(CONNECT_TIMEOUT, tokio_tungstenite::connect_async(&ws_url)).await;
        match connected {
            Ok(Ok((ws_stream, _))) => {
                let _ = conn_state.send(true);
                let opened = Instant::now();
                let ended =
                    run_connection(ws_stream, &token, &mut out_rx, &inbound, &mut unsent).await;
                let _ = conn_state.send(false);
                if matches!(ended, Ended::Shutdown) {
                    return;
                }
                if opened.elapsed() >= STABLE_AFTER {
                    backoff = MIN_BACKOFF;
                }
            }
            Ok(Err(e)) => {
                tracing::debug!("could not connect to the master: {e}");
                let _ = conn_state.send(false);
            }
            Err(_) => {
                tracing::debug!("connecting to the master timed out");
                let _ = conn_state.send(false);
            }
        }

        tokio::time::sleep(backoff + jitter(backoff)).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn run_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    token: &Arc<RwLock<Option<String>>>,
    out_rx: &mut UnboundedReceiver<Outbound>,
    inbound: &UnboundedSender<ServerWsMsg>,
    unsent: &mut Option<ClientWsMsg>,
) -> Ended {
    let (mut sink, mut stream) = ws_stream.split();

    // Clone out of the lock first — the guard isn't Send and must not be held
    // across the await below.
    let auth_token = token.read().clone();
    if let Some(t) = auth_token {
        if sink
            .send(Message::Text(authenticate(t).to_json()))
            .await
            .is_err()
        {
            return Ended::Dropped;
        }
    }
    if let Some(msg) = unsent.take() {
        if sink.send(Message::Text(msg.to_json())).await.is_err() {
            *unsent = Some(msg);
            return Ended::Dropped;
        }
    }

    let mut ping = tokio::time::interval(PING_EVERY);
    ping.tick().await; // the first tick fires immediately
    let mut last_heard = Instant::now();

    loop {
        tokio::select! {
            out = out_rx.recv() => {
                match out {
                    Some(Outbound::Msg(msg)) => {
                        let json = msg.to_json();
                        if sink.send(Message::Text(json)).await.is_err() {
                            *unsent = Some(msg);
                            return Ended::Dropped;
                        }
                    }
                    Some(Outbound::Reconnect) => {
                        let _ = sink.close().await;
                        return Ended::Dropped;
                    }
                    None => return Ended::Shutdown,
                }
            }
            incoming = stream.next() => {
                last_heard = Instant::now();
                match incoming {
                    Some(Ok(Message::Text(t))) => match serde_json::from_str::<ServerWsMsg>(&t) {
                        Ok(msg) => {
                            let _ = inbound.send(msg);
                        }
                        // Usually a message type newer than this launcher.
                        Err(e) => tracing::debug!(error = %e, "unreadable message from the master"),
                    },
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return Ended::Dropped,
                    _ => {}
                }
            }
            _ = ping.tick() => {
                if last_heard.elapsed() >= IDLE_TIMEOUT {
                    tracing::info!("the master has been silent too long, reconnecting");
                    return Ended::Dropped;
                }
                if sink.send(Message::Ping(Vec::new())).await.is_err() {
                    return Ended::Dropped;
                }
            }
        }
    }
}

/// Up to a quarter of the delay, so launchers that lost the master together
/// don't all come back in the same second.
fn jitter(base: Duration) -> Duration {
    use std::time::{SystemTime, UNIX_EPOCH};
    let quarter = (base.as_millis() as u64 / 4).max(1);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_nanos()))
        .unwrap_or(0);
    Duration::from_millis(nanos % quarter)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;

    /// Accepts WebSocket connections, records every text frame, keeps each
    /// connection open until the client goes away.
    async fn master() -> (String, Arc<AtomicUsize>, mpsc::UnboundedReceiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/ws/launcher", listener.local_addr().unwrap());
        let connections = Arc::new(AtomicUsize::new(0));
        let (seen_tx, seen_rx) = mpsc::unbounded_channel();
        let counter = connections.clone();
        tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                let seen = seen_tx.clone();
                tokio::spawn(async move {
                    let Ok(mut ws) = tokio_tungstenite::accept_async(tcp).await else {
                        return;
                    };
                    while let Some(Ok(msg)) = ws.next().await {
                        if let Message::Text(t) = msg {
                            let _ = seen.send(t);
                        }
                    }
                });
            }
        });
        (url, connections, seen_rx)
    }

    async fn wait_for(state: &mut mpsc::UnboundedReceiver<bool>, want: bool) {
        tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(s) = state.recv().await {
                if s == want {
                    return;
                }
            }
        })
        .await
        .expect("connection state never changed");
    }

    #[tokio::test]
    async fn logout_drops_the_authenticated_socket() {
        let (url, connections, mut seen) = master().await;
        let (inbound, _keep) = mpsc::unbounded_channel();
        let (state_tx, mut state) = mpsc::unbounded_channel();
        let client = spawn(url, Some("secret".into()), inbound, state_tx);

        wait_for(&mut state, true).await;
        let first = seen.recv().await.unwrap();
        assert!(
            first.contains("secret"),
            "authenticates on connect: {first}"
        );

        client.set_token(None);
        wait_for(&mut state, false).await;
        wait_for(&mut state, true).await;
        assert_eq!(connections.load(Ordering::SeqCst), 2);

        // The new connection is anonymous: nothing is sent until asked.
        client.send(ClientWsMsg::RequestNews);
        let next = tokio::time::timeout(Duration::from_secs(5), seen.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(!next.contains("secret"), "{next}");
    }
}
