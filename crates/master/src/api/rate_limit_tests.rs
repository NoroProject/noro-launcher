use super::*;
use axum::body::to_bytes;

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

#[test]
fn остаток_убывает_до_нуля() {
    let limiter = RateLimiter::new();
    assert_eq!(limiter.check(ip(3)).remaining, MAX_PER_WINDOW - 1);
    for _ in 1..MAX_PER_WINDOW {
        limiter.check(ip(3));
    }
    assert_eq!(limiter.check(ip(3)).remaining, 0);
}

/// Отказ по лимиту — такой же JSON, как любой другой отказ API: клиент разбирает
/// тело одним и тем же кодом и не спотыкается о голую строку.
#[tokio::test]
async fn отказ_это_json_с_retry_after() {
    let limiter = RateLimiter::new();
    for _ in 0..MAX_PER_WINDOW {
        limiter.check(ip(4));
    }
    let verdict = limiter.check(ip(4));
    assert!(!verdict.allowed);

    let res = too_many(&verdict);
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(res.headers().contains_key(header::RETRY_AFTER));
    assert_eq!(
        res.headers()["ratelimit-limit"],
        MAX_PER_WINDOW.to_string().as_str()
    );

    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "rate_limited");
}
