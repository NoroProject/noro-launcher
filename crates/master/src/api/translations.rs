//! Каталоги локализации: раздача лаунчеру и правка из админки.
//!
//! Лаунчер сначала берёт список языков с хешами, и качает только тот каталог,
//! чей sha1 разошёлся с закешированным.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Serialize)]
pub struct LocaleInfo {
    pub locale: String,
    pub sha1: String,
}

/// Какие языки есть на мастере и каковы их хеши.
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<LocaleInfo>>> {
    let mut map = std::collections::HashMap::new();
    map.insert(
        "en".to_string(),
        hex::encode(Sha1::digest(i18n::Locale::En.builtin_ftl().as_bytes())),
    );
    map.insert(
        "ru".to_string(),
        hex::encode(Sha1::digest(i18n::Locale::Ru.builtin_ftl().as_bytes())),
    );

    let rows = crate::db::list_translations(&state.db).await?;
    for (locale, sha1) in rows {
        map.insert(locale, sha1);
    }

    let mod_map = crate::api::moderation_messages::load_map(&state.db).await;
    for locale in mod_map.keys() {
        if !map.contains_key(locale) {
            map.insert(locale.clone(), "".to_string());
        }
    }

    let mut result: Vec<_> = map
        .into_iter()
        .map(|(locale, sha1)| LocaleInfo { locale, sha1 })
        .collect();
    result.sort_by(|a, b| a.locale.cmp(&b.locale));
    Ok(Json(result))
}

#[derive(Serialize)]
pub struct Catalog {
    pub locale: String,
    pub sha1: String,
    /// Переопределение с мастера. Пусто — используется только встроенный.
    pub ftl: String,
    /// Встроенный каталог этого языка: эталон ключей для редактора.
    pub builtin: String,
}

/// Сам каталог. Отдаётся текстом `.ftl` внутри JSON.
pub async fn get(
    State(state): State<AppState>,
    Path(locale): Path<String>,
) -> AppResult<Json<Catalog>> {
    let stored = crate::db::get_translation(&state.db, &locale).await?;
    let (ftl, sha1) = stored.unwrap_or_default();
    let known_builtin = i18n::Locale::from_code(&locale)
        .map(|k| k.builtin_ftl().to_string())
        .unwrap_or_else(|| i18n::Locale::En.builtin_ftl().to_string());

    Ok(Json(Catalog {
        locale,
        sha1,
        ftl,
        builtin: known_builtin,
    }))
}

#[derive(Deserialize)]
pub struct PutReq {
    pub ftl: String,
}

/// Залить/заменить каталог. Лаунчеры узнают об этом пушем по WebSocket.
pub async fn put(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(locale): Path<String>,
    Json(req): Json<PutReq>,
) -> AppResult<Json<LocaleInfo>> {
    admin.require(schema::PERM_TRANSLATIONS_EDIT)?;
    if locale.is_empty()
        || locale.len() > 16
        || !locale
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(AppError::BadRequest("invalid locale code".into()));
    }
    let sha1 = hex::encode(Sha1::digest(req.ftl.as_bytes()));
    crate::db::upsert_translation(&state.db, &locale, &req.ftl, &sha1, admin.user_id()).await?;
    state
        .ws
        .broadcast(&schema::ServerWsMsg::TranslationsChanged);
    Ok(Json(LocaleInfo { locale, sha1 }))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(locale): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(schema::PERM_TRANSLATIONS_EDIT)?;
    crate::db::delete_translation(&state.db, &locale).await?;
    state
        .ws
        .broadcast(&schema::ServerWsMsg::TranslationsChanged);
    Ok(Json(serde_json::json!({ "ok": true })))
}
