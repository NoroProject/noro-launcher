//! Точка входа noro-master.

fn main() -> anyhow::Result<()> {
    // .env (если есть) — простой загрузчик без зависимости.
    load_dotenv();

    // Sentry поднимается до рантайма и держится за guard до конца процесса:
    // на выходе он дожидается отправки очереди событий.
    let _sentry = master::telemetry::init();
    master::telemetry::init_tracing(_sentry.is_some());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(master::run())
}

/// Минимальный парсер .env: KEY=VALUE построчно, без кавычек/экранирования.
fn load_dotenv() {
    let Ok(content) = std::fs::read_to_string(".env") else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if std::env::var(k.trim()).is_err() {
                std::env::set_var(k.trim(), v.trim().trim_matches('"'));
            }
        }
    }
}
