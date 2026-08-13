//! API лаунчера: WebSocket с мастером, раздача файлов, версия лаунчера.

use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use schema::{ClientWsMsg, ServerWsMsg};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct VersionQuery {
    pub platform: Option<String>,
}

/// Текущая версия лаунчера для платформы.
pub async fn current_version(
    State(state): State<AppState>,
    Query(q): Query<VersionQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let platform = q
        .platform
        .unwrap_or_else(|| schema::current_platform().to_string());
    let row = crate::db::current_launcher_version(&state.db, &platform).await?;
    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "version": r.version,
            "platform": r.platform,
            "url": state.config.file_url(&r.file_sha1),
            "sha256": r.sha256,
            "signature": r.signature,
        }))),
        None => Ok(Json(serde_json::json!(null))),
    }
}

/// Установщики под все платформы — для кнопки скачивания на сайте.
///
/// Отдаётся без авторизации: лаунчер качают до того, как заводят аккаунт.
pub async fn downloads(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = crate::db::current_bootstrappers(&state.db).await?;
    let items: Vec<_> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "platform": r.platform,
                "version": r.version,
                "size": r.size,
                "sha256": r.sha256,
                "url": state.config.file_url(&r.file_sha1),
                "filename": filename_for(&r.platform),
            })
        })
        .collect();
    Ok(Json(serde_json::json!(items)))
}

/// Имя файла для сохранения: стор адресуется по хешу и своего имени не знает.
fn filename_for(platform: &str) -> String {
    match platform {
        p if p.starts_with("windows") => "NoroLauncher.exe".into(),
        p if p.starts_with("macos") => "NoroLauncher.dmg".into(),
        _ => "noro-launcher".into(),
    }
}

/// WebSocket-апгрейд.
pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();

    // Канал для исходящих сообщений (из read-loop и из hub broadcast).
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerWsMsg>();
    let conn_id = state.ws.register(tx.clone());

    // Задача отправки.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink
                .send(Message::Text(msg.to_json().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut authed_user: Option<Uuid> = None;

    while let Some(Ok(msg)) = stream.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => continue,
        };
        let parsed: Result<ClientWsMsg, _> = serde_json::from_str(&text);
        let Ok(client_msg) = parsed else {
            continue;
        };

        if let Err(e) = handle_client_msg(&state, conn_id, &mut authed_user, client_msg, &tx).await
        {
            tracing::warn!(error = %e, "WS message handling failed");
            let _ = tx.send(ServerWsMsg::Notification {
                key: "notif-server-error".into(),
                args: [("reason".to_string(), e.to_string())].into(),
                level: schema::NotifLevel::Error,
            });
        }
    }

    state.ws.unregister(conn_id);
    send_task.abort();
}

async fn handle_client_msg(
    state: &AppState,
    conn_id: crate::ws::ConnId,
    authed_user: &mut Option<Uuid>,
    msg: ClientWsMsg,
    tx: &mpsc::UnboundedSender<ServerWsMsg>,
) -> anyhow::Result<()> {
    match msg {
        ClientWsMsg::Authenticate {
            access_token,
            launcher_version,
            platform,
        } => {
            let token = Uuid::parse_str(&access_token).ok();
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
                    if let Err(e) = crate::db::record_launcher_client(
                        &state.db,
                        user_id,
                        &launcher_version,
                        &platform,
                    )
                    .await
                    {
                        tracing::warn!(error = %e, "не удалось записать версию лаунчера");
                    }
                    let _ = tx.send(ServerWsMsg::AuthOk { user: profile });
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
        }

        ClientWsMsg::RequestServerList => {
            let rows = crate::db::list_servers(&state.db, true).await?;
            let user_profile = match *authed_user {
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
                let entry = crate::db::server_entry(&state.db, s).await?;
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
            let _ = tx.send(ServerWsMsg::ServerList { servers });
        }

        ClientWsMsg::RequestNews => {
            let rows = crate::db::list_news(&state.db, 20).await?;
            let items = news_items(state, rows).await?;
            let _ = tx.send(ServerWsMsg::News { items });
        }

        ClientWsMsg::RequestBuildManifest { server_id } => {
            let Some(user_id) = *authed_user else {
                let _ = tx.send(ServerWsMsg::AuthFail {
                    reason: "auth-sign-in-first".into(),
                });
                return Ok(());
            };
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
            match crate::db::latest_published_build(&state.db, server_id).await? {
                Some(build) => {
                    let manifest = crate::manifest::build_manifest(state, &build).await?;
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
        }

        ClientWsMsg::SetOptionalMods { .. } => {
            // Лаунчер хранит выбор локально; здесь можно записывать для статистики.
        }

        ClientWsMsg::ReportGameStart { server_id } => {
            if let Some(user_id) = *authed_user {
                let _ = crate::db::record_play_start(&state.db, user_id, server_id).await;
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
            }
        }

        ClientWsMsg::Ping => {
            let _ = tx.send(ServerWsMsg::Pong);
        }
    }
    Ok(())
}

async fn news_items(
    state: &AppState,
    rows: Vec<crate::db::models::NewsRow>,
) -> anyhow::Result<Vec<schema::NewsItem>> {
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let author_name = match r.author_id {
            Some(id) => crate::db::get_user(&state.db, id)
                .await?
                .map(|u| u.mc_username),
            None => None,
        };
        items.push(schema::NewsItem {
            id: r.id,
            title: r.title,
            body: r.body,
            preview_img_url: r.preview_img_url,
            author_name,
            pinned: r.pinned,
            published_at: r.published_at,
        });
    }
    Ok(items)
}
