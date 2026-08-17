//! Публичный список серверов: витрина проекта на сайте.
//!
//! Отдаёт только то, что и так видно любому, кто зайдёт на игровой адрес:
//! название, версию, адрес и онлайн. Ограниченные сборки не показываются —
//! закрытый тест не должен светиться на главной.

use crate::db::game_servers::GameServerRow;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PublicServer {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub background_url: Option<String>,
    pub modloader: String,
    pub mc_version: String,
    /// Адрес входа для игрока: то, что вбивают в клиент. Пусто — адрес ещё
    /// не заведён, и показывать нечего.
    pub address: String,
    pub online: u32,
    pub max_online: u32,
    /// Хоть один живой бэкенд — иначе сервер лежит, и онлайн равен нулю.
    pub live: bool,
}

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<PublicServer>>> {
    let mut out = Vec::new();
    for server in crate::db::list_servers(&state.db, true).await? {
        if server.limited {
            continue;
        }
        let entries = crate::db::list_game_servers(&state.db, server.id).await?;
        let live: Vec<&GameServerRow> = entries.iter().filter(|e| e.live()).collect();

        // Прокси видит всех игроков сразу, поэтому складывать его с бэкендами
        // нельзя: один и тот же игрок посчитался бы дважды.
        let proxies: Vec<&GameServerRow> = live.iter().copied().filter(|e| e.is_proxy()).collect();
        let counted = if proxies.is_empty() { &live } else { &proxies };

        out.push(PublicServer {
            address: address_of(&entries),
            online: counted.iter().map(|e| e.online.max(0) as u32).sum(),
            max_online: counted.iter().map(|e| e.max_online.max(0) as u32).sum(),
            live: !live.is_empty(),
            id: server.id,
            name: server.name,
            description: server.description,
            icon_url: server.icon_url,
            background_url: server.background_url,
            modloader: server.modloader,
            mc_version: server.mc_version,
        });
    }
    Ok(Json(out))
}

/// Адрес входа — прокси, если он заведён: именно его вбивают в клиент,
/// бэкенды за ним обычно вообще недоступны снаружи.
fn address_of(entries: &[GameServerRow]) -> String {
    let entry = entries
        .iter()
        .find(|e| e.is_proxy() && !e.mc_host.is_empty())
        .or_else(|| entries.iter().find(|e| !e.mc_host.is_empty()));
    match entry {
        // Порт по умолчанию не пишем: `play.example.com` короче и привычнее.
        Some(e) if e.mc_port == 25565 => e.mc_host.clone(),
        Some(e) => format!("{}:{}", e.mc_host, e.mc_port),
        None => String::new(),
    }
}
