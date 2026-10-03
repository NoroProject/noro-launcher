//! The HTTP client every backend request goes through.
//!
//! reqwest has no timeouts by default. A half-open connection — laptop sleep,
//! a NAT dropping the mapping, a proxy that stops answering — then hangs the
//! request forever: no error, so no retry. Every download stage shares one
//! HTTP/2 connection, so one dead socket stalled the whole sync.

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
        .tcp_keepalive(Duration::from_secs(30))
        // Notices a dead HTTP/2 connection between requests, before the next
        // download is queued on it.
        .http2_keep_alive_interval(Duration::from_secs(20))
        .http2_keep_alive_timeout(Duration::from_secs(10))
        .http2_keep_alive_while_idle(true)
        .build()
}
