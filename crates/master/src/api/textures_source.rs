//! Откуда взять байты скина для рендера, и что считать ошибкой.
//!
//! Граница проходит между «скина нет» и «скин не получен». Первое — штатный
//! ответ: у игрока действительно дефолтная текстура. Второе раньше выглядело
//! точно так же, и битый url молча превращался в Стива — ошибку никто не видел.

use super::textures::RenderQuery;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub const STEVE: &[u8] = include_bytes!("../../assets/steve.png");
const ALEX: &[u8] = include_bytes!("../../assets/alex.png");
const ARI: &[u8] = include_bytes!("../../assets/ari.png");
const ZURI: &[u8] = include_bytes!("../../assets/zuri.png");
const EFE: &[u8] = include_bytes!("../../assets/efe.png");
const MAKENA: &[u8] = include_bytes!("../../assets/makena.png");
const KAI: &[u8] = include_bytes!("../../assets/kai.png");
const SUNNY: &[u8] = include_bytes!("../../assets/sunny.png");
const NOOR: &[u8] = include_bytes!("../../assets/noor.png");

/// Пресет по имени. `None` — такого имени нет.
pub fn preset_bytes(name: &str) -> Option<&'static [u8]> {
    match name {
        "steve" => Some(STEVE),
        "alex" => Some(ALEX),
        "ari" => Some(ARI),
        "zuri" => Some(ZURI),
        "efe" => Some(EFE),
        "makena" => Some(MAKENA),
        "kai" => Some(KAI),
        "sunny" => Some(SUNNY),
        "noor" => Some(NOOR),
        _ => None,
    }
}

pub async fn resolve_skin_bytes(state: &AppState, q: &RenderQuery) -> AppResult<Vec<u8>> {
    if let Some(p) = &q.preset {
        return preset_bytes(p)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| AppError::NotFound(format!("skin preset {p}")));
    }

    if let Some(u) = &q.url {
        return fetch(u).await;
    }

    let Some(name) = q
        .username
        .as_deref()
        .or(q.uuid.as_deref())
        .or(q.identity.as_deref())
    else {
        // Ничего конкретного не просили — дефолтный скин и есть ответ.
        return Ok(STEVE.to_vec());
    };

    let user = crate::db::find_user_by_any_name(&state.db, name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("player {name}")))?;

    match &user.skin_url {
        // Скин не установлен — это не ошибка, у игрока дефолтная текстура.
        None => Ok(STEVE.to_vec()),
        Some(url) => fetch(url).await,
    }
}

async fn fetch(url: &str) -> AppResult<Vec<u8>> {
    let resp = reqwest::get(url).await.map_err(|e| {
        AppError::upstream(
            crate::error_codes::UPSTREAM_FAILED,
            format!("could not download the skin {url}: {e}"),
        )
    })?;
    if !resp.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "скин {url} отдан со статусом {}",
            resp.status()
        )));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| {
        AppError::upstream(
            crate::error_codes::UPSTREAM_FAILED,
            format!("could not read the skin {url}: {e}"),
        )
    })
}
