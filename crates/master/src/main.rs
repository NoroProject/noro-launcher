//! Точка входа noro-master.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // .env (если есть) — простой загрузчик без зависимости.
    load_dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,noro_master=debug,master=debug".into()),
        )
        .init();

    master::run().await
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
