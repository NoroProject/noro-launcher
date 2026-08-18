//! Переключение модели уже загруженного скина.
//!
//! Отдельно от загрузки, потому что это про другое: картинка остаётся та же,
//! меняется только геометрия рук. Заставлять игрока перезаливать файл ради
//! галочки — значит требовать от него хранить исходник, которого у него может
//! и не быть: скин мог приехать по нику с чужого аккаунта.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::UserProfile;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModelReq {
    /// `slim` либо `classic`. Строкой, а не флагом: в Yggdrasil модель тоже
    /// названа словом, и одинаковый словарь по всей цепочке дешевле перевода.
    pub model: String,
}

/// `PUT /api/me/skin/model`
pub async fn set_model(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ModelReq>,
) -> AppResult<Json<UserProfile>> {
    let slim = super::skin_model::parse_choice(&req.model)
        .ok_or_else(|| AppError::BadRequest("model: `slim` or `classic`".into()))?;

    let current = crate::db::get_user(&state.db, user.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    // Скина нет — переключать нечего. Записать флаг молча значит показать
    // игроку выбранную модель на общем Стиве, которого он не загружал.
    let Some(skin_url) = current.skin_url.as_deref() else {
        return Err(AppError::BadRequest("upload a skin first".into()));
    };

    crate::db::set_skin(&state.db, user.user_id, Some(skin_url), slim).await?;

    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    // Тем же кадром, что и загрузка: у игрока может быть открыт и лаунчер, и
    // сайт, и вторая вкладка обязана перерисоваться сама.
    state.ws.send_to_user(
        user.user_id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}
