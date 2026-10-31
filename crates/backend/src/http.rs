//! The HTTP client every backend request goes through.
//!
//! reqwest has no timeouts by default. A half-open connection — laptop sleep,
//! a NAT dropping the mapping, a proxy that stops answering — then hangs the
//! request forever: no error, so no retry. Every download stage shares one
//! HTTP/2 connection, so one dead socket stalled the whole sync.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Long enough for a slow TLS handshake on a bad link.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// No byte for this long means the connection is gone. Not a cap on the
/// whole request: a large file keeps going as long as data flows.
pub const READ_TIMEOUT: Duration = Duration::from_secs(30);

pub fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("noro-launcher/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .tcp_nodelay(true)
        .tcp_keepalive(Duration::from_secs(30))
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(2 * 1024 * 1024)
        .http2_initial_connection_window_size(8 * 1024 * 1024)
        .http2_max_frame_size(Some(65536))
        // Notices a dead HTTP/2 connection between requests, before the next
        // download is queued on it.
        .http2_keep_alive_interval(Duration::from_secs(20))
        .http2_keep_alive_timeout(Duration::from_secs(10))
        .http2_keep_alive_while_idle(true)
        .build()
}

/// Separate connections for the file sync, each its own TCP and TLS session.
///
/// Measured against production on 31.10.2026: one connection to the origin
/// carried 1.9 MB/s, four carried 5.1 MB/s, eight 5.6 MB/s — the limit is per
/// connection, not the link. HTTP/2 multiplexing puts every stream of a
/// client on one connection, so the sync used to sit at the single-connection
/// figure no matter how many files were in flight.
#[derive(Clone)]
pub struct HttpClientPool {
    clients: Arc<Vec<reqwest::Client>>,
    next: Arc<AtomicUsize>,
}

/// Default number of connections; `NORO_SYNC_CONNECTIONS` overrides it, which
/// is how the sync debug report compares settings on a real link.
pub const SYNC_CONNECTIONS: usize = 4;

pub fn sync_connections() -> usize {
    std::env::var("NORO_SYNC_CONNECTIONS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(SYNC_CONNECTIONS)
        .clamp(1, 16)
}

impl HttpClientPool {
    pub fn new(size: usize) -> reqwest::Result<Self> {
        let clients = (0..size.max(1))
            .map(|_| client())
            .collect::<reqwest::Result<Vec<_>>>()?;
        Ok(Self {
            clients: Arc::new(clients),
            next: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Round robin, with the index so the trace can tell connections apart.
    pub fn pick(&self) -> (usize, &reqwest::Client) {
        let i = self.next.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        (i, &self.clients[i])
    }

    pub fn size(&self) -> usize {
        self.clients.len()
    }
}

impl From<reqwest::Client> for HttpClientPool {
    fn from(c: reqwest::Client) -> Self {
        Self {
            clients: Arc::new(vec![c]),
            next: Arc::new(AtomicUsize::new(0)),
        }
    }
}
