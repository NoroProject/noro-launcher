//! Отправка ошибок в Sentry.
//!
//! Включается только при заданном `SENTRY_DSN`. Без него мастер работает как
//! раньше — и говорит об этом в лог, чтобы «тишина в Sentry» не читалась как
//! «ошибок нет».

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Живёт до конца процесса: на drop guard дожидается отправки очереди.
pub type Guard = Option<sentry::ClientInitGuard>;

/// Поднять Sentry. Вызывается до создания рантайма: транспорт заводит свой поток.
pub fn init() -> Guard {
    let dsn = std::env::var("SENTRY_DSN")
        .ok()
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())?;

    let environment = std::env::var("SENTRY_ENVIRONMENT")
        .ok()
        .filter(|e| !e.trim().is_empty())
        .unwrap_or_else(|| "development".to_string());

    Some(sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some(environment.into()),
            // Ошибки, а не трассировка: профилирование запросов здесь не нужно,
            // а бюджет событий на него ушёл бы целиком.
            traces_sample_rate: 0.0,
            attach_stacktrace: true,
            ..Default::default()
        },
    )))
}

/// Логи в stdout плюс, если Sentry поднят, события уровня `error` — в него.
pub fn init_tracing(sentry_on: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,noro_master=debug,master=debug".into());

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer());

    if sentry_on {
        registry.with(sentry_tracing::layer()).init();
        tracing::info!(target: "telemetry", "Sentry подключён");
    } else {
        registry.init();
        tracing::info!(target: "telemetry", "SENTRY_DSN не задан — ошибки только в лог");
    }
}
