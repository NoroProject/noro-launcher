//! Источник мода → файл в сторе.
//!
//! Скачивание отделено от того, куда мод потом кладут: один и тот же jar может
//! уехать и в клиентскую сборку, и на несколько игровых серверов сразу. Качать
//! его столько же раз, сколько целей, незачем.

use super::json;
use super::types::{ModProject, ModSource, ResolvedMod};
use super::{curseforge, fetch_json, modrinth};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use urlencoding::encode;

/// Пустая строка — не значение. Modrinth, например, не кладёт автора в карточку
/// проекта вовсе: он есть только в выдаче поиска, откуда его и присылает
/// админка. Отдать отсюда `Some("")` значит затереть то, что уже известно.
fn meta(project: Option<&ModProject>, pick: fn(&ModProject) -> Option<&str>) -> Option<String> {
    let value = pick(project?)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Имя файла, пригодное для пути.
///
/// Имя приходит из чужих рук: с Modrinth, из CurseForge или прямо из ссылки.
/// Из ссылки оно вдобавок процентно закодировано — `sodium%2B1.21.jar` попал бы
/// в манифест ровно в таком виде, и лаунчер скачал бы файл с плюсом в имени.
/// Разделители пути убираем: файл обязан лечь в `mods/`, а не куда решит автор.
pub fn safe_filename(raw: &str) -> String {
    let decoded = urlencoding::decode(raw).unwrap_or(std::borrow::Cow::Borrowed(raw));
    let name = decoded
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('.');
    if name.is_empty() {
        "mod.jar".to_string()
    } else {
        name.to_string()
    }
}

pub async fn resolve(state: &AppState, source: &ModSource) -> AppResult<ResolvedMod> {
    match source {
        ModSource::Modrinth { version_id } => from_modrinth(state, version_id).await,
        ModSource::Curseforge {
            project_id,
            file_id,
        } => from_curseforge(state, *project_id, *file_id).await,
        ModSource::Url { url, filename } => from_url(state, url, filename.clone()).await,
    }
}

async fn from_modrinth(state: &AppState, version_id: &str) -> AppResult<ResolvedMod> {
    let url = format!("https://api.modrinth.com/v2/version/{}", encode(version_id));
    let version = fetch_json(state, &url, None).await?;
    let files = json::items(&version, "files");
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())
        .ok_or_else(|| AppError::BadRequest("this Modrinth version has no files".into()))?;

    let stored = state
        .files
        .put_url(
            state.http(),
            &json::text(file, "url"),
            file["hashes"]["sha1"].as_str(),
        )
        .await
        .map_err(AppError::Other)?;

    // Карточка проекта нужна только опциональным модам — её отсутствие не
    // повод срывать установку.
    let project = modrinth::project(state, &json::text(&version, "project_id"))
        .await
        .ok();
    Ok(ResolvedMod {
        filename: safe_filename(&json::text(file, "filename")),
        sha1: stored.sha1,
        size: stored.size,
        title: meta(project.as_ref(), |p| Some(&p.hit.title)),
        author: meta(project.as_ref(), |p| Some(&p.hit.author)),
        icon_url: meta(project.as_ref(), |p| p.hit.icon_url.as_deref()),
    })
}

async fn from_curseforge(
    state: &AppState,
    project_id: u64,
    file_id: u64,
) -> AppResult<ResolvedMod> {
    let file = curseforge::file_meta(state, project_id, file_id).await?;
    let sha1 = json::items(&file, "hashes")
        .into_iter()
        // algo 1 — sha1, algo 2 — md5. Второй нашему стору не подходит.
        .find(|h| h["algo"].as_u64() == Some(1))
        .and_then(|h| h["value"].as_str().map(String::from));

    let download = curseforge::download_url(state, project_id, file_id).await?;
    let stored = state
        .files
        .put_url(state.http(), &download, sha1.as_deref())
        .await
        .map_err(AppError::Other)?;

    let project = curseforge::project(state, &project_id.to_string())
        .await
        .ok();
    Ok(ResolvedMod {
        filename: safe_filename(&json::text(&file, "fileName")),
        sha1: stored.sha1,
        size: stored.size,
        title: meta(project.as_ref(), |p| Some(&p.hit.title)),
        author: meta(project.as_ref(), |p| Some(&p.hit.author)),
        icon_url: meta(project.as_ref(), |p| p.hit.icon_url.as_deref()),
    })
}

async fn from_url(state: &AppState, url: &str, filename: Option<String>) -> AppResult<ResolvedMod> {
    let stored = state
        .files
        .put_url(state.http(), url, None)
        .await
        .map_err(AppError::Other)?;
    let filename = filename
        .filter(|f| !f.trim().is_empty())
        // Имя из ссылки: хвост пути без query-строки.
        .or_else(|| url.split(['?', '#']).next().map(String::from))
        .unwrap_or_else(|| "mod.jar".into());

    Ok(ResolvedMod {
        filename: safe_filename(&filename),
        sha1: stored.sha1,
        size: stored.size,
        title: None,
        author: None,
        icon_url: None,
    })
}
