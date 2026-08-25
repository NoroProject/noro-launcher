//! Наказание, выданное командой из игры.
//!
//! Права модератора и рамки свода правил проверяет мастер — теми же функциями,
//! что и админка. Плагин мог бы проверить их и сам, но тогда «кто и сколько
//! может выдать» жило бы в двух местах, а в игре ещё и в трёх реализациях.

use crate::api::admin::punish_limits;
use crate::db::punishments::{NewPunishment, PunishmentRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReq {
    /// `mute` | `warn` | `ban` | `server_ban`
    pub kind: String,
    pub reason: String,
    /// Срок в минутах. `None` — навсегда.
    #[serde(default)]
    pub minutes: Option<i64>,
    /// Код правила из свода. Плагин присылает его из автодополнения, и по нему
    /// же мастер проверяет, что срок укладывается в рамки правила.
    #[serde(default)]
    pub rule_code: Option<String>,
    /// MC UUID того, кто набрал команду. `None` — наказал сам сервер:
    /// автомодерация чата или консоль.
    #[serde(default)]
    pub actor_uuid: Option<Uuid>,
}

/// `POST /api/agent/players/{mc_uuid}/punishments`
pub async fn create_punishment(
    State(state): State<AppState>,
    agent: crate::api::auth::AgentAuth,
    Path(mc_uuid): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<PunishmentRow>> {
    if !matches!(req.kind.as_str(), "ban" | "warn" | "server_ban" | "mute") {
        return Err(AppError::BadRequest("unknown punishment kind".into()));
    }
    if req.reason.trim().len() < 3 {
        return Err(AppError::BadRequest("reason is required".into()));
    }

    let target = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    if req.kind == "ban" && crate::db::is_root_user(&state.db, target.id).await? {
        return Err(AppError::Forbidden("root cannot be banned".into()));
    }

    let server_id = agent.game_server.server_id;
    let actor = super::history::resolve_actor(&state, req.actor_uuid, &agent).await?;

    let rule = match req.rule_code.as_deref() {
        Some(code) => crate::db::rule_by_code(&state.db, code, Some(server_id)).await?,
        None => None,
    };
    let sanctions = match &rule {
        Some(rule) => crate::db::sanctions_of_rule(&state.db, rule.id).await?,
        None => Vec::new(),
    };
    check_limits(&req, &sanctions, rule.is_some(), &actor)?;

    // Мут и бан на сборке действуют там, где выданы: сервер в секрете агента и
    // есть та самая сборка.
    let scoped = matches!(req.kind.as_str(), "server_ban" | "mute");
    let expires_at = req
        .minutes
        .map(|m| chrono::Utc::now() + chrono::Duration::minutes(m.max(1)));

    // Наказание из игры встаёт в идущий разбор, если он есть: модератор в
    // режиме дела наказывает кнопкой, и результат обязан попасть в ленту.
    let case = crate::db::find_open_case(&state.db, target.id, Some(server_id))
        .await
        .ok()
        .flatten();

    let punishment = crate::db::create_punishment(
        &state.db,
        NewPunishment {
            user_id: target.id,
            kind: &req.kind,
            reason: req.reason.trim(),
            actor_id: actor.id,
            actor_label: &actor.label,
            server_id: scoped.then_some(server_id),
            expires_at,
            rule_id: rule.as_ref().map(|r| r.id),
            rule_code: rule.as_ref().map(|r| r.code.as_str()),
            case_id: case.as_ref().map(|c| c.id),
        },
    )
    .await?;

    if let Some(case) = &case {
        crate::cases::event(
            &state,
            case.id,
            actor.id,
            &actor.label,
            "game",
            "punishment",
            serde_json::json!({
                "punishment_id": punishment.id,
                "kind": punishment.kind,
                "reason": punishment.reason,
                "expires_at": punishment.expires_at,
                "rule": punishment.rule_code,
            }),
        )
        .await?;
    }

    if req.kind == "ban" {
        crate::db::refresh_ban_flag(&state.db, target.id).await?;
    }
    crate::agent_link::notify::punished(&state, &punishment).await;

    crate::audit::record(
        &state,
        &actor.audit_actor(),
        crate::audit::actions::punishment(&req.kind),
        crate::audit::target("user", target.id),
        json!({
            "reason": punishment.reason,
            "minutes": req.minutes,
            "rule": punishment.rule_code,
            "from": agent.game_server.name,
        }),
    )
    .await;

    Ok(Json(punishment))
}

/// Живой модератор проверяется как в панели. За сервером прав нет — там некому
/// их предъявить, поэтому ему остаются только рамки свода: команда из игры не
/// должна выдавать то, чего правило не предусматривает.
fn check_limits(
    req: &CreateReq,
    sanctions: &[crate::db::rules::RuleSanctionRow],
    rule_cited: bool,
    actor: &super::history::Actor,
) -> AppResult<()> {
    let request = punish_limits::Request {
        kind: &req.kind,
        minutes: req.minutes,
        rule_cited,
    };
    if let Some(permissions) = &actor.permissions {
        return punish_limits::check(&request, sanctions, |perm| {
            schema::any_permission_matches(permissions.iter().map(String::as_str), perm)
        });
    }
    if sanctions.is_empty() {
        return Ok(());
    }
    if sanctions
        .iter()
        .any(|s| s.kind == req.kind && s.allows(req.minutes))
    {
        return Ok(());
    }
    Err(AppError::Forbidden(format!(
        "the rule allows only: {}",
        punish_limits::describe(sanctions)
    )))
}
