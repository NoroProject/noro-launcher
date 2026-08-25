//! Админ: наказания и заметки на карточке игрока.

use super::punish_limits;
use crate::api::auth::AdminAuth;
use crate::api::validate::Validation;
use crate::audit;
use crate::db::punishments::{NewPunishment, PunishmentRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Utc};
use schema::{PERM_PUNISH_REVOKE, PERM_PUNISH_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReq {
    /// `ban` | `warn` | `server_ban`
    pub kind: String,
    pub reason: String,
    /// Срок в минутах. `None` — навсегда.
    ///
    /// Минуты, а не часы: панель принимает `30m` наравне с `7d`, и округлять
    /// короткий срок вверх до часа значило бы наказывать не тем, что назначили.
    #[serde(default)]
    pub minutes: Option<i64>,
    /// Только для `server_ban`.
    #[serde(default)]
    pub server_id: Option<Uuid>,
    /// Правило, по которому наказывают. Оно же задаёт рамки срока: без
    /// `noro.mod.punish.bypass` выйти за них нельзя.
    #[serde(default)]
    pub rule_id: Option<Uuid>,
    /// То же правило, но кодом. Панель знает `rule_id`, а дело несёт `rule_code`
    /// снимком — и наказать «по тому же пункту, что в деле» иначе означало бы
    /// сначала сходить за списком правил ради одного перевода кода в uuid.
    #[serde(default)]
    pub rule_code: Option<String>,
    /// Разбор, из которого наказывают: наказание встаёт в его ленту.
    #[serde(default)]
    pub case_id: Option<Uuid>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PunishmentRow>>> {
    admin.require(PERM_PUNISH_VIEW)?;
    Ok(Json(crate::db::list_punishments(&state.db, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    Validation::new()
        .one_of("kind", &req.kind, &["ban", "warn", "server_ban", "mute"])
        .rule(
            "reason",
            req.reason.trim().chars().count() >= 3,
            "too_short",
            "a reason is required: it is the only thing that will explain this in six months",
        )
        .rule(
            "server_id",
            req.kind != "server_ban" || req.server_id.is_some(),
            "required",
            "a server restriction needs a server",
        )
        .positive("minutes", req.minutes)
        .finish()?;
    if req.kind == "ban" && crate::db::is_root_user(&state.db, id).await? {
        return Err(AppError::Forbidden("root cannot be banned".into()));
    }

    // Правило решает, что именно допустимо: вид наказания и рамки срока.
    let rule = match (req.rule_id, req.rule_code.as_deref()) {
        (Some(rule_id), _) => crate::db::rule_by_id(&state.db, rule_id).await?,
        (None, Some(code)) => crate::db::rule_by_code(&state.db, code, req.server_id).await?,
        (None, None) => None,
    };
    let sanctions = match &rule {
        Some(rule) => crate::db::sanctions_of_rule(&state.db, rule.id).await?,
        None => Vec::new(),
    };
    punish_limits::check(
        &punish_limits::Request {
            kind: &req.kind,
            minutes: req.minutes,
            rule_cited: rule.is_some(),
        },
        &sanctions,
        |perm| admin.require(perm).is_ok(),
    )?;

    let expires_at = req
        .minutes
        .map(|m| Utc::now() + Duration::minutes(m.max(1)));
    let row = crate::db::create_punishment(
        &state.db,
        NewPunishment {
            user_id: id,
            kind: &req.kind,
            reason: req.reason.trim(),
            actor_id: admin.user_id(),
            actor_label: &admin.actor.label(),
            server_id: req.server_id,
            expires_at,
            rule_id: rule.as_ref().map(|r| r.id),
            rule_code: rule.as_ref().map(|r| r.code.as_str()),
            case_id: req.case_id,
        },
    )
    .await?;

    // Наказание, выданное из разбора, обязано быть видно в его ленте: иначе
    // дело закрывается «подтвердилось», а чем кончилось — непонятно.
    if let Some(case_id) = req.case_id {
        crate::cases::event(
            &state,
            case_id,
            admin.user_id(),
            &admin.actor.label(),
            "web",
            "punishment",
            json!({
                "punishment_id": row.id,
                "kind": row.kind,
                "reason": row.reason,
                "expires_at": row.expires_at,
                "rule": row.rule_code,
            }),
        )
        .await?;
    }

    // Флаг в users остаётся кэшем: по нему ходят Yggdrasil и WS-вход.
    let banned = crate::db::refresh_ban_flag(&state.db, id).await?;
    if banned {
        notify_profile(&state, id).await;
    }
    // Игрок может сидеть в игре прямо сейчас: бан обязан выкинуть его сразу,
    // а мут — заткнуть до того, как он допишет строку.
    crate::agent_link::notify::punished(&state, &row).await;

    audit::record(
        &state,
        &admin.actor,
        audit::actions::punishment(&req.kind),
        audit::target("user", id),
        json!({
            "reason": row.reason,
            "expires_at": row.expires_at,
            "server_id": row.server_id,
            "rule": row.rule_code,
        }),
    )
    .await;

    Ok(Json(json!(row)))
}

pub async fn revoke(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, punishment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_PUNISH_REVOKE)?;
    // Читаем до снятия: агенту нужно знать, что именно отпустить, а после
    // `revoke_punishment` строка уже помечена снятой.
    let punishment = crate::db::punishment_by_id(&state.db, punishment_id).await?;
    if !crate::db::revoke_punishment(&state.db, punishment_id, admin.user_id()).await? {
        return Err(AppError::NotFound(
            "punishment not found or already lifted".into(),
        ));
    }
    crate::db::refresh_ban_flag(&state.db, id).await?;
    notify_profile(&state, id).await;
    if let Some(punishment) = &punishment {
        crate::agent_link::notify::revoked(&state, punishment, &admin.actor.label()).await;
    }

    audit::record(
        &state,
        &admin.actor,
        audit::actions::PUNISHMENT_REVOKE,
        audit::target("user", id),
        json!({ "punishment_id": punishment_id }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

/// Разослать обновлённый профиль: игрок должен узнать о снятии бана сразу, а
/// не при следующем перезапуске лаунчера.
async fn notify_profile(state: &AppState, id: Uuid) {
    if let Ok(profile) = crate::db::load_profile(&state.db, id).await {
        state.ws.send_to_user(
            id,
            &schema::ServerWsMsg::PermissionsUpdated { user: profile },
        );
    }
}
