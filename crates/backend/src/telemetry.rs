//! Отчёты о падениях лаунчера.
//!
//! До этого паника на машине игрока не оставляла ничего: на Windows у сборки нет
//! консоли вовсе (`windows_subsystem = "windows"`), и окно просто исчезало.
//!
//! Три правила, потому что это чужие машины, а не наш сервер:
//! 1. DSN зашивается на сборке (`NORO_SENTRY_DSN`). Его нет — SDK не поднимается
//!    и ни одного байта никуда не уходит. Дефолтного DSN здесь нет и быть не может.
//! 2. Игрок может выключить отправку в настройках, и тогда мы даже не стартуем.
//! 3. Ничего личного: ни имени машины, ни PII, ни трассировки запросов.

use crate::config::LauncherConfig;
use sentry::SessionMode;

/// Живёт до конца процесса. На `drop` отправляет то, что осталось в очереди.
pub type Guard = Option<sentry::ClientInitGuard>;

/// DSN пустой в dev-сборках: разработчик не должен слать шум в боевой проект.
fn dsn() -> Option<&'static str> {
    option_env!("NORO_SENTRY_DSN").filter(|d| !d.trim().is_empty())
}

/// Версия для группировки в Sentry.
///
/// `NORO_RELEASE` проставляет сборочный workflow из тега (`launcher-v1.6.2`).
/// Без него берётся версия из Cargo.toml, а она отстаёт: тег `launcher-v1.6.2`
/// был выпущен, когда в манифесте всё ещё стояло `1.6.1`, и падения из 1.6.2
/// приписались бы предыдущему релизу.
fn release() -> Option<std::borrow::Cow<'static, str>> {
    match option_env!("NORO_RELEASE").filter(|r| !r.trim().is_empty()) {
        Some(tag) => Some(tag.into()),
        None => sentry::release_name!(),
    }
}

/// Поднимает Sentry, если он вшит в сборку и игрок не отказался.
///
/// Вызывать до создания рантайма и GPUI: хук паники должен стоять раньше, чем
/// появится первый шанс упасть.
pub fn init(config: &LauncherConfig) -> Guard {
    let dsn = dsn()?;
    if !config.crash_reports {
        tracing::info!(target: "telemetry", "отчёты о падениях выключены игроком");
        return None;
    }

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: release(),
            environment: Some(if cfg!(debug_assertions) {
                "development".into()
            } else {
                "production".into()
            }),
            // Имя хоста Sentry подставляет сам, а у людей оно вида
            // «MacBook Ивана» — это персональные данные, и нам они не нужны.
            server_name: None,
            send_default_pii: false,
            traces_sample_rate: 0.0,
            attach_stacktrace: true,
            // Сессия на запуск лаунчера. Из них считается доля запусков без
            // падения по версиям — единственная метрика, по которой видно,
            // стало ли хуже после релиза.
            auto_session_tracking: true,
            session_mode: SessionMode::Application,
            ..Default::default()
        },
    ));
    tracing::info!(target: "telemetry", release = ?release(), "отчёты о падениях включены");
    Some(guard)
}

/// Дождаться отправки очереди.
///
/// Обычный выход из лаунчера идёт через `std::process::exit`, а он не вызывает
/// деструкторы — без явного вызова guard не успел бы ничего отправить, и сессия
/// осталась бы висеть незакрытой.
pub fn flush() {
    sentry::end_session();
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(std::time::Duration::from_secs(2)));
    }
}

/// Логи лаунчера плюс, если Sentry поднят, события уровня `error` — в него.
///
/// Без этого слоя в Sentry попадали бы только паники. А лаунчер большую часть
/// отказов не роняет, а пишет `error!` — не скачался файл, не сошлась подпись
/// манифеста, не запустилась игра. Именно это и надо видеть.
pub fn init_tracing(sentry_on: bool) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,backend=debug,frontend=debug,bridge=debug".into());
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer());

    if sentry_on {
        registry.with(sentry_tracing::layer()).init();
    } else {
        registry.init();
    }
}

/// Вшит ли DSN в эту сборку. Настройка без него — обманка: переключатель есть,
/// а отправлять всё равно некуда.
pub fn is_available() -> bool {
    dsn().is_some()
}

/// Будет ли Sentry поднят: и вшит, и разрешён игроком.
///
/// Нужно знать до `init`, потому что подписчик логов ставится раньше — иначе
/// первые же строки о самой телеметрии писались бы в никуда.
pub fn is_enabled(config: &LauncherConfig) -> bool {
    is_available() && config.crash_reports
}
