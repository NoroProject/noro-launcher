//! Админ: настройки инстанса.
//!
//! Секретные поля отдаются только фактом «задано / не задано». Значение не
//! показывается никогда, даже маской длины: длина сама по себе сужает перебор,
//! а пользы от неё нет.

mod diagnostics;
pub(crate) mod image_prep;

pub use diagnostics::diagnostics;

use image_prep::{has_transparency, prepare_image};

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::config::keys;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
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

    // Прозрачность залитых картинок: по ней страница настроек объясняет, почему
    // у одной иллюстрации на сайте есть подложка, а у другой нет.
    let mut transparency: BTreeMap<&str, bool> = BTreeMap::new();
    for key in IMAGE_KEYS {
        let Some(meta) = keys::ALL.iter().find(|k| k.name == *key) else {
            continue;
        };
        transparency.insert(key, image_prep::transparency(&state, &stored, *meta).await);
    }

    Ok(Json(json!({
        "settings": items,
        "secrets": secrets,
        "transparency": transparency,
    })))
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

    // Адрес картинки вписали руками — что там внутри, мы не знаем: чужой файл
    // не качаем ради одного флага. Признак прозрачности от прежней картинки
    // тут вреднее, чем его отсутствие, поэтому сбрасываем на «непрозрачная».
    for key in IMAGE_KEYS.iter().filter(|k| diff.contains_key(**k)) {
        values.insert(keys::transparency_key(key), Value::Bool(false));
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
/// Настройки-картинки, которые можно залить файлом. Список закрытый: иначе
/// ручка писала бы произвольный ключ настроек чужим значением.
const IMAGE_KEYS: &[&str] = &[
    keys::LOGO_URL.name,
    keys::HERO_IMAGE_URL.name,
    keys::LOGIN_IMAGE_URL.name,
];

/// Загрузить иллюстрацию и записать её адрес в настройку `key`.
///
/// Одна ручка на все картинки: обложка главной и картинка страницы входа
/// отличаются только именем настройки, а копия обработчика на каждую новую
/// разъезжалась бы в мелочах вроде допустимого размера.
pub async fn upload_image(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(key): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;
    let key = IMAGE_KEYS
        .iter()
        .find(|k| **k == key)
        .ok_or_else(|| AppError::BadRequest(format!("{key} is not an image setting")))?;
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
        // Прозрачность считаем здесь же, пока картинка в руках: по одному URL
        // её потом не узнать — файл может уехать за CDN, который на запрос из
        // браузера не отдаёт CORS-заголовков.
        let (data, transparent) = tokio::task::spawn_blocking(move || {
            let data = prepare_image(&raw)?;
            let transparent = has_transparency(&data);
            Ok::<_, image_prep::ImagePrepError>((data, transparent))
        })
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .map_err(|e| {
            AppError::bad(
                crate::error_codes::UPLOAD_BAD_FORMAT,
                format!("invalid image: {e}"),
            )
        })?;
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
        values.insert(key.to_string(), Value::String(url.clone()));
        values.insert(keys::transparency_key(key), Value::Bool(transparent));
        crate::db::set_settings(&state.db, &values, admin.user_id()).await?;

        audit::record(
            &state,
            &admin.actor,
            audit::actions::SETTINGS_UPDATE,
            None,
            json!({ *key: url }),
        )
        .await;

        return Ok(Json(
            json!({ "ok": true, "url": url, "transparent": transparent }),
        ));
    }
    Err(AppError::bad(
        crate::error_codes::UPLOAD_FIELD_MISSING,
        "missing the image field",
    ))
}
