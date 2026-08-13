//! Ограничение частоты запросов по IP для эндпоинтов авторизации.
//!
//! Ставится только на браузерный/лаунчерный вход: обмен кодов, refresh и
//! passkeys. Yggdrasil сюда намеренно не входит — его зовёт игровой сервер при
//! каждом заходе игрока, и лимит там резал бы вход в игру, а не перебор.

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Длина окна и его ёмкость. Живой человек столько запросов подряд не делает,
/// а перебор упирается сразу.
const WINDOW: Duration = Duration::from_secs(60);
const MAX_PER_WINDOW: u32 = 60;

/// Сверх этого числа отслеживаемых адресов таблица подрезается: иначе она
/// растёт по числу уникальных IP, а не по нагрузке.
const MAX_TRACKED_IPS: usize = 10_000;

struct Window {
    started: Instant,
    hits: u32,
}

#[derive(Default)]
pub struct RateLimiter {
    windows: DashMap<IpAddr, Window>,
}

impl RateLimiter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// `true` — запрос в пределах лимита.
    fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();

        if self.windows.len() > MAX_TRACKED_IPS {
            self.windows
                .retain(|_, w| now.duration_since(w.started) < WINDOW);
        }

        let mut entry = self.windows.entry(ip).or_insert(Window {
            started: now,
            hits: 0,
        });
        if now.duration_since(entry.started) >= WINDOW {
            entry.started = now;
            entry.hits = 0;
        }
        entry.hits += 1;
        entry.hits <= MAX_PER_WINDOW
    }
}

/// Middleware: отдаёт 429, когда окно исчерпано.
pub async fn limit(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&req, peer);
    if !limiter.allow(ip) {
        tracing::warn!(%ip, path = %req.uri().path(), "превышен лимит запросов");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Try again in a minute.",
        )
            .into_response();
    }
    next.run(req).await
}

/// Адрес клиента. Мастер стоит за Traefik, поэтому реальный IP приходит в
/// `X-Forwarded-For`; без прокси там пусто и берётся адрес соединения.
///
/// Заголовку тут можно верить только потому, что порт мастера наружу не
/// опубликован и дойти до него можно исключительно через прокси. Если порт
/// когда-нибудь пробросят на хост, `X-Forwarded-For` станет подделываемым, и
/// лимит начнёт обходиться сменой одного заголовка — тогда доверенные прокси
/// придётся задавать списком.
fn client_ip(req: &Request, peer: SocketAddr) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| peer.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(n: u8) -> IpAddr {
        IpAddr::from([10, 0, 0, n])
    }

    #[test]
    fn allows_up_to_the_cap_then_blocks() {
        let limiter = RateLimiter::new();
        for _ in 0..MAX_PER_WINDOW {
            assert!(limiter.allow(ip(1)));
        }
        assert!(!limiter.allow(ip(1)));
    }

    #[test]
    fn counts_each_address_separately() {
        let limiter = RateLimiter::new();
        for _ in 0..MAX_PER_WINDOW {
            assert!(limiter.allow(ip(1)));
        }
        assert!(limiter.allow(ip(2)));
    }
}
