//! Фоновый опрос GitHub Releases: уведомляет в лог о новом теге (каждые 15 мин).

use crate::state::AppState;
use std::time::Duration;

pub fn spawn(state: AppState) {
    let Some(repo) = state.config.github_repo.clone() else {
        return; // нечего опрашивать
    };
    tokio::spawn(async move {
        let mut last_seen: Option<String> = None;
        let mut ticker = tokio::time::interval(Duration::from_secs(15 * 60));
        loop {
            ticker.tick().await;
            match latest_tag(&state, &repo).await {
                Ok(Some(tag)) => {
                    if last_seen.as_deref() != Some(tag.as_str()) {
                        // Новый тег — собран ли он уже?
                        let built: Option<String> = sqlx::query_scalar(
                            "SELECT version FROM launcher_versions WHERE version=$1 LIMIT 1",
                        )
                        .bind(&tag)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten();
                        if built.is_none() {
                            tracing::info!(target: "launcher_builder", "доступен новый тег лаунчера: {tag}");
                        }
                        last_seen = Some(tag);
                    }
                }
                Ok(None) => {}
                Err(e) => tracing::warn!("github_watcher: {e}"),
            }
        }
    });
}

async fn latest_tag(state: &AppState, repo: &str) -> anyhow::Result<Option<String>> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let mut req = state.http().get(&url).header("User-Agent", "noro-master");
    if let Some(tok) = &state.config.github_token {
        req = req.bearer_auth(tok);
    }
    let resp: serde_json::Value = req.send().await?.json().await?;
    Ok(resp["tag_name"].as_str().map(String::from))
}
