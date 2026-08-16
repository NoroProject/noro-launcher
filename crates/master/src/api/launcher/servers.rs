//! Список серверов и выдача манифеста — то, что видно игроку по правам.

use super::messages::Tx;
use crate::state::AppState;
use schema::{ServerEntry, ServerWsMsg};
use uuid::Uuid;

/// Серверы, которые игрок вправе увидеть.
///
/// Список сборок сам по себе говорит, что готовится к выкату, поэтому версии
/// без права отсекаются тут же, а не в клиенте.
pub async fn visible_servers(
    state: &AppState,
    authed_user: Option<Uuid>,
) -> anyhow::Result<Vec<ServerEntry>> {
    let rows = crate::db::list_servers(&state.db, true).await?;
    let user_profile = match authed_user {
        Some(uid) => crate::db::load_profile(&state.db, uid).await.ok(),
        None => None,
    };
    let is_admin = match &user_profile {
        Some(p) => {
            p.has_permission(schema::PERM_ADMIN_SERVERS)
                || p.has_permission(schema::PERM_ADMIN_ALL)
                || p.has_permission(schema::PERM_SUPERADMIN)
        }
        None => false,
    };

    let mut servers = Vec::new();
    for s in &rows {
        let mut entry = crate::db::server_entry(&state.db, s).await?;
        entry.available_builds.retain(|b| match &user_profile {
            Some(p) => p.has_permission(&schema::perm_build_access(
                &s.id.to_string(),
                &b.id.to_string(),
            )),
            None => false,
        });
        if is_admin || entry.current_build_id.is_some() {
            let can_join = match &user_profile {
                Some(p) => p.can_join_server(&s.id, s.limited),
                None => !s.limited,
            };
            if is_admin || can_join {
                servers.push(entry);
            }
        }
    }
    Ok(servers)
}

/// Собрать и отправить персональный манифест сборки.
pub async fn send_manifest(
    state: &AppState,
    user_id: Uuid,
    server_id: Uuid,
    build_id: Option<Uuid>,
    tx: &Tx,
) -> anyhow::Result<()> {
    let profile = crate::db::load_profile(&state.db, user_id).await?;
    let server = crate::db::get_server(&state.db, server_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("server not found"))?;
    if !profile.can_join_server(&server_id, server.limited) {
        let _ = tx.send(ServerWsMsg::Notification {
            key: "notif-no-server-access".into(),
            args: Default::default(),
            level: schema::NotifLevel::Error,
        });
        return Ok(());
    }
    // Запрошенная версия отдаётся только при наличии права на неё:
    // иначе достаточно было бы подставить чужой id в сообщение.
    let requested = match build_id {
        Some(id)
            if profile.has_permission(&schema::perm_build_access(
                &server_id.to_string(),
                &id.to_string(),
            )) =>
        {
            crate::db::get_build(&state.db, id)
                .await?
                .filter(|b| b.server_id == server_id)
        }
        _ => None,
    };

    let chosen = match requested {
        Some(b) => Some(b),
        None => crate::db::latest_published_build(&state.db, server_id).await?,
    };

    match chosen {
        Some(build) => {
            let manifest = crate::manifest::build_manifest(state, &build, Some(&profile)).await?;
            let _ = tx.send(ServerWsMsg::BuildManifest { manifest });
        }
        None => {
            let _ = tx.send(ServerWsMsg::Notification {
                key: "notif-no-published-build".into(),
                args: Default::default(),
                level: schema::NotifLevel::Warning,
            });
        }
    }
    Ok(())
}

/// Записать в лог включённые limited-моды, на которые у игрока нет права.
///
/// Не отказ и не бан: клиентский сигнал — повод для ручного разбора, файлов
/// мода у игрока всё равно нет. Когда появятся integrity-флаги (§7.1), отчёт
/// поедет туда же.
pub async fn report_forbidden_optionals(
    state: &AppState,
    user_id: Uuid,
    server_id: Uuid,
    enabled: &[String],
) -> anyhow::Result<()> {
    if enabled.is_empty() {
        return Ok(());
    }
    let Some(build) = crate::db::latest_published_build(&state.db, server_id).await? else {
        return Ok(());
    };
    let mods: Vec<schema::OptionalMod> = serde_json::from_value(build.optional_mods.clone())?;
    let profile = crate::db::load_profile(&state.db, user_id).await?;

    let forbidden: Vec<&str> = enabled
        .iter()
        .filter(|name| {
            mods.iter().any(|m| {
                &&m.name == name
                    && m.limited
                    && !profile.can_use_optional(&server_id, &m.name, true)
            })
        })
        .map(String::as_str)
        .collect();

    if !forbidden.is_empty() {
        tracing::warn!(
            %user_id, %server_id, mods = ?forbidden,
            "лаунчер сообщил о включённых limited-модах без права"
        );
    }
    Ok(())
}

/// Открыть запись журнала запусков со снапшотом того, с чем игрок зашёл.
///
/// Снимок берётся из последнего integrity-отчёта: он приходит прямо перед
/// стартом и уже содержит и версию сборки, и набор включённых модов.
pub async fn open_play_session(
    state: &AppState,
    user_id: Uuid,
    server_id: Uuid,
) -> anyhow::Result<()> {
    let snapshot = crate::db::latest_integrity_snapshot(&state.db, user_id, server_id).await?;
    let (build_version, launcher_version, optional, ok) =
        snapshot.unwrap_or_else(|| (String::new(), String::new(), serde_json::json!([]), None));
    crate::db::open_play_session(
        &state.db,
        user_id,
        server_id,
        &build_version,
        &launcher_version,
        &optional,
        ok,
    )
    .await?;
    Ok(())
}
