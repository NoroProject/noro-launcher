//! Админ: что лежит в `{NORO_DATA_DIR}/agents` — агенты и ServerWrapper.
//!
//! Здесь только перечисление: сами файлы раздаёт обычный `/files/{sha1}` с Range
//! и ETag, как и всё остальное. Заводить ради скачивания второй маршрут незачем,
//! а jar'ы агентов не секрет — они и так подписаны.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ADMIN_AGENTS;

use serde::Serialize;

#[derive(Serialize)]
pub struct AgentFile {
    pub file: String,
    /// `paper`, `fabric`, `neoforge`, `forge` — либо `wrapper`.
    pub platform: String,
    /// `None` у враппера: он один на все версии.
    pub mc_version: Option<String>,
    pub size: u64,
    pub sha1: String,
    pub url: String,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<AgentFile>>> {
    admin.require(PERM_ADMIN_AGENTS)?;

    let dir = state.config.data_dir.join("agents");
    let mut files = Vec::new();
    // Каталога может не быть, если агентов ещё не собирали, — это не ошибка,
    // а пустой список и подсказка в админке.
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(Json(files));
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        // Файлы с точки — не агенты. macOS кладёт рядом AppleDouble `._имя.jar`
        // при распаковке архива, собранного её же tar, и такой файл проходил
        // проверку расширения: в админке появлялись платформы `._fabric`,
        // `._paper` и прочие, а мусор ещё и уезжал в общий стор.
        if !name.ends_with(".jar") || name.starts_with('.') {
            continue;
        }
        // Кладём в общий стор: дальше ссылка ведёт на /files/{sha1}, который уже
        // умеет докачку и кеширование.
        let stored = state.files.put_file(&entry.path()).await?;
        let stem = name.trim_end_matches(".jar");
        let (platform, mc_version) = match stem.split_once('-') {
            Some((platform, mc)) => (platform.to_string(), Some(mc.to_string())),
            None => (stem.to_string(), None),
        };
        // `?name=` — чтобы файл сохранился как `wrapper.jar`, а не как свой sha1:
        // ссылка ведёт на мастер, а админка живёт на другом origin, и атрибут
        // `download` в браузере там не работает.
        let url = format!("{}?name={}", state.config.file_url(&stored.sha1), name);
        files.push(AgentFile {
            file: name,
            platform,
            mc_version,
            size: stored.size,
            url,
            sha1: stored.sha1,
        });
    }

    files.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(Json(files))
}
