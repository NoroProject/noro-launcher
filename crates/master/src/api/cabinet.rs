//! Личный кабинет: профиль, смена ника, скин.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, State};
use axum::Json;
use schema::UserProfile;

use serde::Deserialize;

pub async fn me(user: AuthUser) -> Json<UserProfile> {
    Json(user.profile)
}

#[derive(Deserialize)]
pub struct UsernameReq {
    pub username: String,
}

fn valid_username(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn set_username(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UsernameReq>,
) -> AppResult<Json<UserProfile>> {
    crate::api::validate::Validation::new()
        .rule(
            "username",
            valid_username(&req.username),
            "invalid_format",
            "3 to 16 characters: latin letters, digits or underscore",
        )
        .finish()?;
    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE mc_username = $1 AND id <> $2)",
    )
    .bind(&req.username)
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await?;
    if taken {
        return Err(AppError::Conflict("username is taken".into()));
    }
    crate::db::set_username(&state.db, user.user_id, &req.username).await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}

/// Загрузка скина (multipart, поле `skin`, PNG).
pub async fn upload_skin(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<UserProfile>> {
    // Оба поля собираем до записи. Раньше запись шла прямо в ветке файла, и
    // модель терялась, если клиент клал её в форму после картинки — а именно
    // так делает `FormData.append` в привычном порядке «сначала файл».
    let mut chosen: Option<bool> = None;
    let mut skin: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("model") | Some("slim") => {
                if let Ok(txt) = field.text().await {
                    chosen = super::skin_model::parse_choice(&txt);
                }
            }
            Some("skin") => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                // Простая проверка PNG-сигнатуры.
                if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
                    return Err(AppError::bad(
                        crate::error_codes::UPLOAD_BAD_FORMAT,
                        "PNG expected",
                    ));
                }
                if data.len() > 256 * 1024 {
                    return Err(AppError::bad(
                        crate::error_codes::UPLOAD_TOO_LARGE,
                        "skin is too large",
                    ));
                }
                skin = Some(data.to_vec());
            }
            _ => {}
        }
    }

    if let Some(data) = skin {
        // Игрок не выбрал — угадываем по картинке. Большинство и не знает, что
        // такое «slim», а нарисованный под Алекс скин с толстыми руками виден
        // сразу и выглядит как поломка сервера, а не как выбор по умолчанию.
        let slim = match chosen {
            Some(value) => value,
            None => super::skin_model::detect_slim(&data),
        };
        let stored = state
            .files
            .put_bytes(&data)
            .await
            .map_err(AppError::Other)?;
        let url = state.config.file_url(&stored.sha1);
        crate::db::set_skin(&state.db, user.user_id, Some(&url), slim).await?;
        let profile = crate::db::load_profile(&state.db, user.user_id).await?;
        state.ws.send_to_user(
            user.user_id,
            &schema::ServerWsMsg::PermissionsUpdated {
                user: profile.clone(),
            },
        );
        return Ok(Json(profile));
    }
    Err(AppError::bad(
        crate::error_codes::UPLOAD_FIELD_MISSING,
        "missing skin field",
    ))
}

pub async fn delete_skin(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<UserProfile>> {
    crate::db::set_skin(&state.db, user.user_id, None, false).await?;
    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    state.ws.send_to_user(
        user.user_id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct SkinFromUsernameReq {
    pub username: String,
}

/// Установка скина по нику игрока Minecraft.
pub async fn upload_skin_from_username(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SkinFromUsernameReq>,
) -> AppResult<Json<UserProfile>> {
    let username = req.username.trim();
    if username.is_empty() {
        return Err(AppError::BadRequest("Username is required".into()));
    }
    let url = format!("https://minotar.net/skin/{username}");
    let resp = state.http.get(&url).send().await.map_err(|e| {
        AppError::upstream(
            crate::error_codes::UPSTREAM_FAILED,
            format!("Failed to fetch skin: {e}"),
        )
    })?;

    if !resp.status().is_success() {
        return Err(AppError::BadRequest("Skin for player not found".into()));
    }

    let data = resp
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(AppError::BadRequest(
            "Downloaded skin is not a valid PNG".into(),
        ));
    }

    let stored = state
        .files
        .put_bytes(&data)
        .await
        .map_err(AppError::Other)?;
    let skin_url = state.config.file_url(&stored.sha1);
    // Модель берём у самой картинки, а не у того, что стояло раньше: скин
    // чужой, и нарисован он под свою геометрию. Оставить прежнюю значит выдать
    // тонкий скин с толстыми руками — это видно сразу и читается как поломка.
    let slim = super::skin_model::detect_slim(&data);
    crate::db::set_skin(&state.db, user.user_id, Some(&skin_url), slim).await?;
    let _ = sqlx::query(
        "INSERT INTO user_skin_presets (user_id, name, skin_url, skin_slim)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user.user_id)
    .bind(username)
    .bind(&skin_url)
    .bind(slim)
    .execute(&state.db)
    .await;

    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    state.ws.send_to_user(
        user.user_id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}

pub async fn list_capes(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<schema::CapeRow>>> {
    let is_admin = user.profile.has_permission(schema::PERM_USERS_VIEW);
    if is_admin {
        Ok(Json(crate::db::list_capes(&state.db).await?))
    } else {
        Ok(Json(
            crate::db::list_capes_for_user(&state.db, user.user_id).await?,
        ))
    }
}

pub async fn set_cape(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<schema::SelectCapeReq>,
) -> AppResult<Json<UserProfile>> {
    let is_admin = user.profile.has_permission(schema::PERM_USERS_VIEW);
    let cape_url = match req.cape_id {
        Some(cape_id) => {
            if !is_admin {
                let allowed_ids =
                    crate::db::list_user_granted_cape_ids(&state.db, user.user_id).await?;
                if !allowed_ids.contains(&cape_id) {
                    return Err(AppError::Forbidden(
                        "access to this cape is not granted".into(),
                    ));
                }
            }
            Some(
                crate::db::get_cape_url(&state.db, cape_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("cape".into()))?,
            )
        }
        None => None,
    };
    crate::db::set_user_cape(&state.db, user.user_id, cape_url.as_deref()).await?;
    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    state.ws.send_to_user(
        user.user_id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SkinPresetItem {
    pub id: uuid::Uuid,
    pub name: String,
    pub skin_url: String,
    /// Модель, с которой пресет сохраняли. Без неё переключение между
    /// сохранёнными скинами ломало бы вид: картинка та, руки другие.
    pub skin_slim: bool,
}

#[derive(serde::Deserialize)]
pub struct CreateSkinPresetReq {
    pub name: String,
    pub skin_url: String,
    /// Не прислали — берём ту, что стоит у игрока сейчас.
    #[serde(default)]
    pub skin_slim: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct RenameSkinPresetReq {
    pub name: String,
}

pub async fn list_skin_presets(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<SkinPresetItem>>> {
    let rows = sqlx::query_as::<_, SkinPresetItem>(
        "SELECT id, name, skin_url, skin_slim FROM user_skin_presets WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(rows))
}

pub async fn create_skin_preset(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSkinPresetReq>,
) -> AppResult<Json<SkinPresetItem>> {
    let preset_slim = match req.skin_slim {
        Some(value) => value,
        None => crate::db::get_user(&state.db, user.user_id)
            .await?
            .is_some_and(|u| u.skin_slim),
    };
    let row = sqlx::query_as::<_, SkinPresetItem>(
        "INSERT INTO user_skin_presets (user_id, name, skin_url, skin_slim)
         VALUES ($1, $2, $3, $4) RETURNING id, name, skin_url, skin_slim",
    )
    .bind(user.user_id)
    .bind(&req.name)
    .bind(&req.skin_url)
    .bind(preset_slim)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(row))
}

pub async fn rename_skin_preset(
    State(state): State<AppState>,
    user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(req): Json<RenameSkinPresetReq>,
) -> AppResult<Json<()>> {
    sqlx::query("UPDATE user_skin_presets SET name = $1 WHERE id = $2 AND user_id = $3")
        .bind(&req.name)
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(()))
}

pub async fn delete_skin_preset(
    State(state): State<AppState>,
    user: AuthUser,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> AppResult<Json<()>> {
    sqlx::query("DELETE FROM user_skin_presets WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(()))
}

pub async fn punishments(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<crate::db::punishments::PunishmentRow>>> {
    crate::db::expire_punishments(&state.db).await?;
    Ok(Json(
        crate::db::list_punishments(&state.db, user.user_id).await?,
    ))
}
