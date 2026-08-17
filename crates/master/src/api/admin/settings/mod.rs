//! Админ: настройки инстанса.
//!
//! Секретные поля отдаются только фактом «задано / не задано». Значение не
//! показывается никогда, даже маской длины: длина сама по себе сужает перебор,
//! а пользы от неё нет.

mod diagnostics;

pub use diagnostics::diagnostics;

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::config::keys;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Право на правку настроек. Отдельно: настройки решают, куда игроки ходят за
/// файлами и кто пускает их внутрь.
pub use schema::{PERM_SETTINGS_EDIT, PERM_SETTINGS_VIEW};

#[derive(Serialize)]
pub struct SettingItem {
    pub key: &'static str,
    pub env: &'static str,
    pub value: String,
    /// Значение пришло из окружения — правка в БД его не перекроет.
    pub from_env: bool,
}

/// Текущие настройки.
pub async fn list(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    let stored = crate::db::all_settings(&state.db).await?;

    let items: Vec<SettingItem> = keys::ALL
        .iter()
        .map(|k| {
            let from_env = crate::config::env_opt(k.env);
            SettingItem {
                key: k.name,
                env: k.env,
                value: from_env.clone().unwrap_or_else(|| {
                    stored
                        .get(k.name)
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string()
                }),
                from_env: from_env.is_some(),
            }
        })
        .collect();

    let secrets: BTreeMap<&str, bool> = keys::SECRET_ENV
        .iter()
        .map(|k| (*k, crate::config::env_opt(k).is_some()))
        .collect();

    Ok(Json(json!({ "settings": items, "secrets": secrets })))
}

#[derive(Deserialize)]
pub struct SaveReq {
    pub settings: BTreeMap<String, String>,
}

/// Сохранить настройки. Применяются рестартом — сами мы его не делаем: рестарт
/// рвёт WS лаунчеров и обрывает идущие загрузки, и момент выбирает оператор.
pub async fn save(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;

    let before = crate::db::all_settings(&state.db).await?;
    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    let mut diff = serde_json::Map::new();

    for (key, value) in req.settings {
        if !keys::ALL.iter().any(|k| k.name == key) {
            return Err(AppError::BadRequest(format!("unknown setting: {key}")));
        }
        let value = value.trim().to_string();
        let old = before
            .get(&key)
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if old != value {
            diff.insert(key.clone(), json!({ "from": old, "to": value }));
        }
        values.insert(key, Value::String(value));
    }

    if diff.is_empty() {
        return Ok(Json(json!({ "ok": true, "changed": 0 })));
    }

    crate::db::set_settings(&state.db, &values, admin.user_id()).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::SETTINGS_UPDATE,
        None,
        Value::Object(diff.clone()),
    )
    .await;

    Ok(Json(json!({
        "ok": true,
        "changed": diff.len(),
        // Конфиг читается один раз при старте: пока мастер не перезапущен,
        // сохранённое лежит в БД и ни на что не влияет.
        "restart_required": true,
    })))
}

/// Экспорт в `.env` — чтобы конфиг можно было увезти обратно в compose.
pub async fn export_env(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    let stored = crate::db::all_settings(&state.db).await?;

    let mut lines = vec![
        "# Экспорт настроек инстанса. Секреты сюда не попадают — они и так".into(),
        "# живут только в окружении.".into(),
        format!("NORO_BIND={}", state.config.bind_addr),
    ];
    for k in keys::ALL {
        let value = crate::config::env_opt(k.env).unwrap_or_else(|| {
            stored
                .get(k.name)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        });
        lines.push(format!("{}={}", k.env, value));
    }
    Ok(Json(json!({ "env": lines.join("\n") })))
}

/// Загрузка баннера/иллюстрации главного экрана (Hero Render).
pub async fn upload_hero_image(
    State(state): State<AppState>,
    admin: AdminAuth,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default();
        if name != "image" && name != "file" {
            continue;
        }
        let raw = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let data = tokio::task::spawn_blocking(move || fit_hero_image(&raw))
            .await
            .map_err(|e| AppError::Other(e.into()))?
            .map_err(|e| AppError::BadRequest(format!("invalid image: {e}")))?;
        let stored = state
            .files
            .put_bytes(&data)
            .await
            .map_err(AppError::Other)?;
        let url = if let Some(s3) = &state.config.s3 {
            crate::files::s3::put(state.http(), s3, &stored.sha1, &data)
                .await
                .map_err(AppError::Other)?
        } else {
            state.config.file_url(&stored.sha1)
        };

        let mut values = BTreeMap::new();
        values.insert(
            keys::HERO_IMAGE_URL.name.to_string(),
            Value::String(url.clone()),
        );
        crate::db::set_settings(&state.db, &values, admin.user_id()).await?;

        audit::record(
            &state,
            &admin.actor,
            audit::actions::SETTINGS_UPDATE,
            None,
            json!({ "hero_image_url": url }),
        )
        .await;

        return Ok(Json(json!({ "ok": true, "url": url })));
    }
    Err(AppError::BadRequest("missing the image field".into()))
}

fn fit_hero_image(data: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let img = match image::load_from_memory(data) {
        Ok(i) => i,
        Err(_) => return Ok(data.to_vec()),
    };
    let img = if img.width() > 2048 || img.height() > 2048 {
        img.resize(2048, 2048, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 92);
    img.write_with_encoder(encoder)?;
    Ok(out)
}
