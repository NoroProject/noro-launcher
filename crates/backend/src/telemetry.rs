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

/// Живёт до конца процесса. На `drop` отправляет то, что осталось в очереди.
pub type Guard = Option<sentry::ClientInitGuard>;

/// DSN пустой в dev-сборках: разработчик не должен слать шум в боевой проект.
fn dsn() -> Option<&'static str> {
    option_env!("NORO_SENTRY_DSN").filter(|d| !d.trim().is_empty())
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
            release: sentry::release_name!(),
            // Имя хоста Sentry подставляет сам, а у людей оно вида
            // «MacBook Ивана» — это персональные данные, и нам они не нужны.
            server_name: None,
            send_default_pii: false,
            traces_sample_rate: 0.0,
            attach_stacktrace: true,
            ..Default::default()
        },
    ));
    tracing::info!(target: "telemetry", "отчёты о падениях включены");
    Some(guard)
}

/// Дождаться отправки очереди.
///
/// Обычный выход из лаунчера идёт через `std::process::exit`, а он не вызывает
/// деструкторы — без явного вызова guard не успел бы ничего отправить.
pub fn flush() {
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(std::time::Duration::from_secs(2)));
    }
}

/// Вшит ли DSN в эту сборку. Настройка без него — обманка: переключатель есть,
/// а отправлять всё равно некуда.
pub fn is_available() -> bool {
    dsn().is_some()
}
