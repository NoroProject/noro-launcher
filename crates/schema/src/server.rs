//! Серверные профили, видимые лаунчеру.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modloader {
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl Modloader {
    pub fn as_str(&self) -> &'static str {
        match self {
            Modloader::Vanilla => "vanilla",
            Modloader::Fabric => "fabric",
            Modloader::Quilt => "quilt",
            Modloader::Forge => "forge",
            Modloader::NeoForge => "neoforge",
        }
    }
}

impl std::str::FromStr for Modloader {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "vanilla" => Ok(Modloader::Vanilla),
            "fabric" => Ok(Modloader::Fabric),
            "quilt" => Ok(Modloader::Quilt),
            "forge" => Ok(Modloader::Forge),
            "neoforge" => Ok(Modloader::NeoForge),
            other => Err(format!("неизвестный модлоадер: {other}")),
        }
    }
}

/// Игровой сервер, работающий на сборке. Их может быть несколько.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameServerEntry {
    pub id: Uuid,
    pub name: String,
    pub mc_host: String,
    pub mc_port: u16,
    pub online: u32,
    pub max_online: u32,
    /// Агент выходил на связь недавно. Мёртвые в сумму не попадают.
    pub live: bool,
    /// Точка входа (Velocity/BungeeCord). Игрок коннектится сюда, а онлайн
    /// считают бэкенды за ним — иначе он сложился бы дважды.
    pub proxy: bool,
}

/// Версия сборки, доступная игроку для выбора в лаунчере.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildOption {
    pub id: Uuid,
    pub version: String,
    /// `false` — превью: сборка ещё не выкачена всем.
    pub published: bool,
}

/// Карточка сервера в списке лаунчера.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerEntry {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub background_url: Option<String>,
    /// Куда коннектиться: прокси, если он есть, иначе первый игровой сервер.
    /// Считается мастером — своего адреса у сборки больше нет.
    ///
    /// `None` — игровые серверы ещё не заведены, адреса просто нет. Раньше здесь
    /// оказывался пустой хост и порт 25565: адрес выглядел настоящим, а коннект
    /// уходил в никуда.
    #[serde(default)]
    pub mc_host: Option<String>,
    #[serde(default)]
    pub mc_port: Option<u16>,
    pub modloader: Modloader,
    pub mc_version: String,
    /// id опубликованной сборки, если есть.
    pub current_build_id: Option<Uuid>,
    pub current_version: Option<String>,
    /// Требует права `noro.server.<id>.join`.
    pub limited: bool,
    pub sort_order: i32,
    /// Зарегистрированные игровые сервера сборки.
    #[serde(default)]
    pub game_servers: Vec<GameServerEntry>,
    /// Сборки, доступные этому игроку: опубликованные и те неопубликованные,
    /// на которые у него есть право. Пусто у лаунчеров, выпущенных до появления
    /// выбора версии — они просто продолжают качать текущую.
    #[serde(default)]
    pub available_builds: Vec<BuildOption>,
    /// Сумма онлайна живых серверов. `None` — сервера не зарегистрированы,
    /// то есть онлайн неизвестен; это не то же самое, что ноль игроков.
    #[serde(default)]
    pub online: Option<u32>,
    #[serde(default)]
    pub max_online: Option<u32>,
}
