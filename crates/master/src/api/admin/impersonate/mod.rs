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
pub const PERM_IMPERSONATE: &str = "noro.admin.impersonate";

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
        .ok_or_else(|| AppError::Forbidden("нужен вход пользователем, не admin-токен".into()))?;

    let reason = req.reason.trim();
    if reason.len() < 3 {
        return Err(AppError::BadRequest(
            "нужна причина: она единственное, что потом объяснит, зачем это было".into(),
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
                "из сессии impersonation новый вход не выдаётся".into(),
            ));
        }
    }
    if !can_impersonate(&actor, &target) {
        return Err(AppError::Forbidden(
            "цель должна иметь права строго внутри ваших".into(),
        ));
    }
    if !crate::db::step_up_active(&state.db, actor_id).await? {
        return Err(AppError::Forbidden("step_up_required".into()));
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
        "impersonate.request",
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
        .ok_or_else(|| AppError::NotFound("грант".into()))?;
    if Some(grant.actor_id) != admin.user_id() {
        return Err(AppError::Forbidden("это не ваш грант".into()));
    }
    Ok(Json(json!({ "status": grant.status() })))
}
