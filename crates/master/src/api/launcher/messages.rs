//! Разбор входящих сообщений лаунчера.

use crate::state::AppState;
use schema::{ClientWsMsg, ServerWsMsg};

use tokio::sync::mpsc;
use uuid::Uuid;

pub type Tx = mpsc::UnboundedSender<ServerWsMsg>;

pub async fn handle(
    state: &AppState,
    conn_id: crate::ws::ConnId,
    authed_user: &mut Option<Uuid>,
    msg: ClientWsMsg,
    tx: &Tx,
) -> anyhow::Result<()> {
    match msg {
        ClientWsMsg::Authenticate {
            access_token,
            launcher_version,
            platform,
        } => {
            authenticate(
                state,
                conn_id,
                authed_user,
                &access_token,
                &launcher_version,
                &platform,
                tx,
            )
            .await?
        }

        ClientWsMsg::RequestServerList => {
            let servers = super::servers::visible_servers(state, *authed_user).await?;
            let _ = tx.send(ServerWsMsg::ServerList { servers });
        }

        ClientWsMsg::RequestNews => {
            let rows = crate::db::list_news(&state.db, 20).await?;
            let items = super::news::news_items(state, rows).await?;
            let _ = tx.send(ServerWsMsg::News { items });
        }

        ClientWsMsg::RequestBuildManifest {
            server_id,
            build_id,
        } => {
            let Some(user_id) = *authed_user else {
                let _ = tx.send(ServerWsMsg::AuthFail {
                    reason: "auth-sign-in-first".into(),
                });
                return Ok(());
            };
            super::servers::send_manifest(state, user_id, server_id, build_id, tx).await?;
        }

        ClientWsMsg::SetOptionalMods { server_id, enabled } => {
            // Выбор лаунчер хранит локально, мастеру он интересен как сигнал:
            // limited-мода без права в манифесте больше нет (manifest::access),
            // так что назвать его может только клиент, дописавший себе список.
            if let Some(user_id) = *authed_user {
                super::servers::report_forbidden_optionals(state, user_id, server_id, &enabled)
                    .await?;
            }
        }

        ClientWsMsg::ReportIntegrity { report } => {
            // Без входа отчёт не к кому привязать, а анонимные находки
            // разбирать не о ком.
            if let Some(user_id) = *authed_user {
                // Снимок пишется всегда — журналу запусков нужна версия сборки
                // и у тех, у кого всё сошлось.
                crate::db::save_integrity_snapshot(&state.db, user_id, &report).await?;
                let saved = crate::db::save_integrity_report(&state.db, user_id, &report).await?;
                if saved > 0 {
                    tracing::warn!(%user_id, findings = saved, "сверка лаунчера нашла расхождения");
                    // Одной строкой на отчёт, а не на находку: подробности уже
                    // лежат в integrity_flags, а журнал не должен ими зарастать.
                    crate::audit::record_by_user(
                        state,
                        user_id,
                        crate::audit::actions::INTEGRITY_FINDINGS,
                        serde_json::json!({
                            "count": saved,
                            "build_version": report.build_version,
                            "subjects": report.findings.iter().take(10)
                                .map(|f| f.subject.clone()).collect::<Vec<_>>(),
                        }),
                    )
                    .await;
                }
            }
        }

        ClientWsMsg::ReportGameStart { server_id } => {
            if let Some(user_id) = *authed_user {
                let _ = crate::db::record_play_start(&state.db, user_id, server_id).await;
                super::servers::open_play_session(state, user_id, server_id).await?;
                crate::audit::record_by_user(
                    state,
                    user_id,
                    crate::audit::actions::GAME_START,
                    serde_json::json!({ "server_id": server_id }),
                )
                .await;
            }
        }

        ClientWsMsg::ReportGameStop {
            server_id,
            playtime_secs,
        } => {
            if let Some(user_id) = *authed_user {
                let _ = crate::db::record_play_stop(
                    &state.db,
                    user_id,
                    server_id,
                    playtime_secs as i64,
                )
                .await;
                let _ = crate::db::close_play_session(
                    &state.db,
                    user_id,
                    server_id,
                    playtime_secs as i64,
                )
                .await;
                crate::audit::record_by_user(
                    state,
                    user_id,
                    crate::audit::actions::GAME_STOP,
                    serde_json::json!({ "server_id": server_id, "playtime_secs": playtime_secs }),
                )
                .await;
            }
        }

        ClientWsMsg::DiagnosticsReport { report } => {
            if let Some(user_id) = *authed_user {
                crate::db::save_diagnostics(&state.db, user_id, &serde_json::to_value(&report)?)
                    .await?;
            }
        }

        ClientWsMsg::LogRequestResponse {
            request_id,
            accepted,
        } => {
            if let Some(user_id) = *authed_user {
                crate::db::answer_log_request(&state.db, request_id, user_id, accepted).await?;
            }
        }

        ClientWsMsg::ImpersonateResponse { grant_id, accepted } => {
            // Отказ записываем так же, как согласие: веб-страница админа ждёт
            // ответа, и молчание оставило бы её в поллинге до истечения гранта.
            if authed_user.is_some() {
                crate::db::set_grant_accepted(&state.db, grant_id, accepted).await?;
            }
        }

        ClientWsMsg::Ping => {
            let _ = tx.send(ServerWsMsg::Pong);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn authenticate(
    state: &AppState,
    conn_id: crate::ws::ConnId,
    authed_user: &mut Option<Uuid>,
    access_token: &str,
    launcher_version: &str,
    platform: &str,
    tx: &Tx,
) -> anyhow::Result<()> {
    let token = Uuid::parse_str(access_token).ok();
    let row = match token {
        Some(t) => crate::db::user_by_access_token(&state.db, t).await?,
        None => None,
    };
    match row {
        Some(r) if !r.banned => {
            let user_id = r.id;
            let profile = crate::db::profile_from_row(&state.db, r).await?;
            *authed_user = Some(user_id);
            state.ws.authenticate(conn_id, user_id);
            // Не критично для входа: если запись не удалась, игрок всё
            // равно должен подключиться — это только статистика.
            if let Err(e) =
                crate::db::record_launcher_client(&state.db, user_id, launcher_version, platform)
                    .await
            {
                tracing::warn!(error = %e, "не удалось записать версию лаунчера");
            }
            let _ = tx.send(ServerWsMsg::AuthOk { user: profile });

            // Отправляем накопившиеся запросы логов из очереди при входе
            if let Ok(pending_requests) = crate::db::list_pending_for_user(&state.db, user_id).await
            {
                for req in pending_requests {
                    let _ = tx.send(ServerWsMsg::LogRequest {
                        request_id: req.id,
                        actor_username: req.actor_label,
                        reason: req.reason,
                        forced: req.forced,
                        server_id: req.server_id,
                        expires_at: req.expires_at,
                    });
                }
            }
        }
        Some(_) => {
            let _ = tx.send(ServerWsMsg::AuthFail {
                reason: "auth-banned".into(),
            });
        }
        None => {
            let _ = tx.send(ServerWsMsg::AuthFail {
                reason: "auth-session-expired".into(),
            });
        }
    }
    Ok(())
}
