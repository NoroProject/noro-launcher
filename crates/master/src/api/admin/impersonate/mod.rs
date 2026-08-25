//! Impersonation: вход админа в аккаунт игрока.
//!
//! Токен уходит по уже аутентифицированному каналу лаунчера — не через браузер,
//! не через URL и не через аргументы процесса: последние видны в `ps` любому
//! процессу на машине. Нативный диалог в лаунчере при этом работает вторым
//! фактором: он закрывает случай «злоумышленник получил веб-сессию админа, но
//! не доступ к его машине».
//!
//! Игрок не уведомляется — ни пушем, ни записью в кабинете. Поэтому под
//! impersonation в аудит пишется каждое действие, а не факт входа: иначе
//! разрешить спор «меня обокрали» / «это был не я» будет нечем.

mod claim;
mod rules;
pub mod step_up;

pub use claim::claim;
pub use rules::can_impersonate;

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Отдельно от `noro.admin.*`: это вход в чужой аккаунт, а не правка карточки.
pub use schema::PERM_IMPERSONATE;

#[derive(Deserialize)]
pub struct StartReq {
    pub reason: String,
}

/// Начать вход. Грант уходит в лаунчер актора на подтверждение.
pub async fn start(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(target_id): Path<Uuid>,
    Json(req): Json<StartReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_IMPERSONATE)?;
    let actor_id = admin
        .user_id()
        .ok_or_else(|| AppError::Forbidden("sign in as a user, not with an admin token".into()))?;

    let reason = req.reason.trim();
    if reason.len() < 3 {
        return Err(AppError::BadRequest(
            "a reason is required: it is the only thing that will explain this later".into(),
        ));
    }

    let actor = crate::db::load_profile(&state.db, actor_id).await?;
    let target = crate::db::load_profile(&state.db, target_id).await?;

    // Вложенности нет: сессия, открытая под impersonation, новых грантов не
    // выдаёт — иначе цепочкой можно дотянуться куда угодно.
    if let Some(token) = admin.session_token {
        if crate::db::session_impersonated_by(&state.db, token)
            .await?
            .is_some()
        {
            return Err(AppError::Forbidden(
                "an impersonation session cannot start another sign-in".into(),
            ));
        }
    }
    if !can_impersonate(&actor, &target) {
        return Err(AppError::Forbidden(
            "the target's permissions must be strictly within yours".into(),
        ));
    }
    if !crate::db::step_up_active(&state.db, actor_id).await? {
        // Отдельный код, а не текст: по нему админка открывает подтверждение
        // passkey. Раньше она искала подстроку в сообщении об ошибке.
        return Err(AppError::coded(
            axum::http::StatusCode::FORBIDDEN,
            crate::error_codes::STEP_UP_REQUIRED,
            "confirm it is you before acting as someone else",
        ));
    }

    let grant = crate::db::create_grant(&state.db, actor_id, target_id, reason).await?;

    state.ws.send_to_user(
        actor_id,
        &schema::ServerWsMsg::ImpersonateRequest {
            grant_id: grant.id,
            actor_username: actor.username.clone(),
            target_username: target.username.clone(),
            reason: reason.to_string(),
            expires_at: grant.expires_at,
        },
    );

    audit::record(
        &state,
        &admin.actor,
        audit::actions::IMPERSONATE_REQUEST,
        audit::target("user", target_id),
        json!({ "grant_id": grant.id, "reason": reason }),
    )
    .await;

    Ok(Json(json!({
        "grant_id": grant.id,
        "expires_at": grant.expires_at,
        // Лаунчер не в сети — код вводится руками в поле «Код входа».
        "launcher_online": state.ws.is_user_connected(actor_id),
    })))
}

/// Состояние гранта — для поллинга из веба.
pub async fn status(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(grant_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_IMPERSONATE)?;
    let grant = crate::db::get_grant(&state.db, grant_id)
        .await?
        .ok_or_else(|| AppError::NotFound("grant".into()))?;
    if Some(grant.actor_id) != admin.user_id() {
        return Err(AppError::Forbidden("this grant is not yours".into()));
    }
    Ok(Json(json!({ "status": grant.status() })))
}
