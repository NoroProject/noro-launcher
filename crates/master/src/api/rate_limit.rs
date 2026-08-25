//! Ограничение частоты запросов по IP для эндпоинтов авторизации.
//!
//! Ставится только на браузерный/лаунчерный вход: обмен кодов, refresh и
//! passkeys. Yggdrasil сюда намеренно не входит — его зовёт игровой сервер при
//! каждом заходе игрока, и лимит там резал бы вход в игру, а не перебор.

use crate::error::AppError;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
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

/// Решение лимитера. `reset_after` нужен клиенту, чтобы знать, когда повторять,
/// а не гадать: без него единственная стратегия — долбиться дальше.
pub struct Verdict {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_after: u64,
}

#[derive(Default)]
pub struct RateLimiter {
    windows: DashMap<IpAddr, Window>,
}

impl RateLimiter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn check(&self, ip: IpAddr) -> Verdict {
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

        Verdict {
            allowed: entry.hits <= MAX_PER_WINDOW,
            remaining: MAX_PER_WINDOW.saturating_sub(entry.hits),
            reset_after: WINDOW
                .saturating_sub(now.duration_since(entry.started))
                .as_secs()
                .max(1),
        }
    }

    /// `true` — запрос в пределах лимита.
    #[cfg(test)]
    fn allow(&self, ip: IpAddr) -> bool {
        self.check(ip).allowed
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
    let verdict = limiter.check(ip);
    if !verdict.allowed {
        tracing::warn!(%ip, path = %req.uri().path(), "превышен лимит запросов");
        return too_many(&verdict);
    }
    next.run(req).await
}

/// Отказ по лимиту в том же конверте, что и любая другая ошибка API.
///
/// Раньше здесь была голая строка — единственный ответ мастера, который не
/// JSON. Клиент, разбирающий тело отказа, спотыкался на нём и показывал
/// «malformed JSON» вместо «слишком часто».
fn too_many(verdict: &Verdict) -> Response {
    let mut res = AppError::coded(
        StatusCode::TOO_MANY_REQUESTS,
        crate::error_codes::RATE_LIMITED,
        format!(
            "too many requests — try again in {} seconds",
            verdict.reset_after
        ),
    )
    .into_response();

    let headers = res.headers_mut();
    headers.insert(header::RETRY_AFTER, num(verdict.reset_after));
    headers.insert("ratelimit-limit", num(MAX_PER_WINDOW as u64));
    headers.insert("ratelimit-remaining", num(verdict.remaining as u64));
    headers.insert("ratelimit-reset", num(verdict.reset_after));
    res
}

/// Число в заголовок.
///
/// Без запасного значения: десятичная запись `u64` — всегда валидный
/// `HeaderValue`, а подстановка «0» на отказе означала бы «повторяй немедленно»,
/// то есть ровно противоположное тому, зачем этот заголовок нужен.
fn num(v: u64) -> HeaderValue {
    HeaderValue::from_str(&v.to_string()).expect("десятичное число — валидный заголовок")
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
#[path = "rate_limit_tests.rs"]
mod tests;
