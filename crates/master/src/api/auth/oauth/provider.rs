//! Какие платформы умеет мастер и чем они отличаются друг от друга.
//!
//! Список закрытый, а не строка из БД: у каждой платформы свой формат ответа
//! профиля и свои особенности запроса, и «добавить провайдера в админке» без
//! кода всё равно не получилось бы.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Discord,
    Twitch,
    Google,
}

/// Профиль игрока на стороне платформы.
pub struct RemoteIdentity {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
}

impl Provider {
    pub const ALL: [Provider; 3] = [Provider::Discord, Provider::Twitch, Provider::Google];

    pub fn slug(self) -> &'static str {
        match self {
            Provider::Discord => "discord",
            Provider::Twitch => "twitch",
            Provider::Google => "google",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.slug() == slug)
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Provider::Discord => "Discord",
            Provider::Twitch => "Twitch",
            Provider::Google => "Google",
        }
    }

    pub fn authorize_url(self) -> &'static str {
        match self {
            Provider::Discord => "https://discord.com/api/oauth2/authorize",
            Provider::Twitch => "https://id.twitch.tv/oauth2/authorize",
            Provider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    pub fn token_url(self) -> &'static str {
        match self {
            Provider::Discord => "https://discord.com/api/oauth2/token",
            Provider::Twitch => "https://id.twitch.tv/oauth2/token",
            Provider::Google => "https://oauth2.googleapis.com/token",
        }
    }

    pub fn userinfo_url(self) -> &'static str {
        match self {
            Provider::Discord => "https://discord.com/api/users/@me",
            Provider::Twitch => "https://api.twitch.tv/helix/users",
            Provider::Google => "https://www.googleapis.com/oauth2/v3/userinfo",
        }
    }

    /// Запрашиваем минимум: игрока опознаём по id, всё остальное — витрина.
    /// Почта не нужна ни для чего, поэтому и не спрашивается.
    pub fn scope(self) -> &'static str {
        match self {
            Provider::Discord => "identify",
            // Helix отдаёт id, login и аватар и без единого scope.
            Provider::Twitch => "",
            Provider::Google => "openid profile",
        }
    }

    /// Достать id, ник и аватар из ответа платформы.
    pub fn parse_identity(self, body: &Value) -> Option<RemoteIdentity> {
        match self {
            Provider::Discord => {
                let id = body["id"].as_str()?.to_string();
                Some(RemoteIdentity {
                    avatar: body["avatar"]
                        .as_str()
                        .map(|h| format!("https://cdn.discordapp.com/avatars/{id}/{h}.png")),
                    username: body["username"].as_str()?.to_string(),
                    id,
                })
            }
            // Helix всегда заворачивает ответ в массив `data`.
            Provider::Twitch => {
                let u = body["data"].get(0)?;
                Some(RemoteIdentity {
                    id: u["id"].as_str()?.to_string(),
                    username: u["login"].as_str()?.to_string(),
                    avatar: u["profile_image_url"].as_str().map(str::to_string),
                })
            }
            Provider::Google => Some(RemoteIdentity {
                id: body["sub"].as_str()?.to_string(),
                // У Google нет обязательного ника, только имя. Пустое имя
                // ломало бы подбор MC-ника, поэтому подстраховываемся `sub`.
                username: body["name"]
                    .as_str()
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| body["sub"].as_str().unwrap_or_default())
                    .to_string(),
                avatar: body["picture"].as_str().map(str::to_string),
            }),
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.slug())
    }
}
