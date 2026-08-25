//! Пользователи, роли, профиль.

use crate::permissions::{permission_matches, Permission};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Выдача права в конкретном контексте: `server_id: None` — на всех сборках.
///
/// Плоского списка узлов для админки мало: одно и то же право может быть выдано
/// глобально и на паре сборок сразу, и без `server_id` эти выдачи неразличимы —
/// список показывал их одинаково, а снять точечную было нечем.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGrant {
    pub permission: Permission,
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Role {
    pub id: Uuid,
    /// Машинное имя: "player", "vip", "admin".
    pub name: String,
    pub display_name: String,
    /// HEX-цвет для отображения, например "#5865F2".
    pub color: Option<String>,
    pub permissions: Vec<Permission>,
    /// Выдаётся всем новым пользователям автоматически.
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i32,
    /// Имя группы в LuckPerms — единственная связь двух раздельных моделей
    /// прав. `None` значит, что роль в игру не проецируется.
    #[serde(default)]
    pub lp_group: Option<String>,
    /// Имя иконки из набора либо юникод-символ. Роль показывается рядом с
    /// ником, поэтому нужен глиф, переживающий и веб, и чат в игре.
    #[serde(default)]
    pub icon: Option<String>,
    /// Что стоит перед ником в игре: `&8[&cADMIN&8] `. Не иконка — та остаётся
    /// одним символом для таба и сайта, а здесь произвольная строка с цветами.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Что стоит после ника. Тот же формат, что и у префикса.
    #[serde(default)]
    pub suffix: Option<String>,
    /// Своя картинка плашки. Пусто — плашку рисует мастер сам.
    #[serde(default)]
    pub badge_sha1: Option<String>,
    /// Те же права, но с контекстом сборки. `permissions` остаётся плоским:
    /// проверки прав про контекст не знают, он нужен только админке.
    #[serde(default)]
    pub permission_grants: Vec<PermissionGrant>,
    /// Роль, у которой эта наследует права. `None` — своих достаточно.
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    /// Права, пришедшие по цепочке родителей. Отдельным списком, а не
    /// подмешаны в `permissions`: иначе в админке не отличить своё право от
    /// чужого, а снятие унаследованного молча не делало бы ничего.
    #[serde(default)]
    pub inherited_permissions: Vec<Permission>,
}

/// Привязка аккаунта к внешней платформе. Их у игрока может быть несколько:
/// вход через любую ведёт в один и тот же аккаунт.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdentity {
    /// Машинное имя платформы: "discord", "twitch", "google".
    pub provider: String,
    /// Идентификатор на стороне платформы.
    pub provider_user_id: String,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    /// Платформа, через которую игрок зарегистрировался: из неё выведен его
    /// MC-UUID, и отвязать её нельзя.
    #[serde(default)]
    pub is_primary: bool,
    pub linked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    /// Внутренний UUID пользователя в БД мастера. Нужен админке, CLI и ACL-операциям.
    pub id: Uuid,
    /// MC UUID (UUID v5 из первичной привязки).
    pub uuid: Uuid,
    /// MC ник.
    pub username: String,
    /// Привязанные платформы. Пусто у локального аккаунта: он заведён
    /// оператором, а не внешним входом.
    #[serde(default)]
    pub identities: Vec<UserIdentity>,
    pub skin_url: Option<String>,
    /// Тонкая модель (Алекс). `false` — классическая (Стив): именно её клиент
    /// подразумевает, когда метаданных у текстуры нет, поэтому она и умолчание.
    #[serde(default)]
    pub skin_slim: bool,
    pub cape_url: Option<String>,
    pub roles: Vec<Role>,
    /// Прямые права поверх ролей.
    #[serde(default)]
    pub permissions: Vec<Permission>,
    /// Они же с контекстом сборки — для админки, см. [`PermissionGrant`].
    #[serde(default)]
    pub permission_grants: Vec<PermissionGrant>,
    #[serde(default)]
    pub banned: bool,
    /// Чем бан объяснён. Рядом с флагом, а не только в журнале: первым делом
    /// в карточке ищут именно причину.
    #[serde(default)]
    pub ban_reason: Option<String>,
    /// Когда аккаунт заведён и когда игрок заходил в последний раз.
    ///
    /// Опциональны: их знает только мастер, а профиль ходит и обратно — от
    /// лаунчера, которому эти поля взять неоткуда.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_login_at: Option<DateTime<Utc>>,
    /// Заведён оператором, без привязки к Discord.
    #[serde(default)]
    pub is_local_account: bool,
    /// Может ли входить в игру. Операторский аккаунт по умолчанию не может:
    /// ему это незачем, а игровой профиль — лишняя поверхность.
    #[serde(default = "yes")]
    pub can_play: bool,
    /// Единственный аккаунт, который нельзя забанить и удалить.
    #[serde(default)]
    pub is_root: bool,
    #[serde(default)]
    pub hide_from_online: bool,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default)]
    pub freeze_info: Option<FreezeInfo>,
    #[serde(default)]
    pub silent_join: bool,
}

impl UserProfile {
    /// Платформа, через которую игрок зарегистрировался. У аккаунтов, заведённых
    /// до появления флага, первой идёт самая старая привязка — она же и была
    /// единственной.
    pub fn primary_identity(&self) -> Option<&UserIdentity> {
        self.identities
            .iter()
            .find(|i| i.is_primary)
            .or_else(|| self.identities.first())
    }

    /// Ник на платформе — то, как игрок называет себя вне игры. `None` у
    /// локального аккаунта: он заведён оператором и платформы не имеет.
    pub fn handle(&self) -> Option<&str> {
        self.primary_identity()?.username.as_deref()
    }

    /// Аватар: с первичной платформы, а если там его нет — с любой другой.
    pub fn avatar_url(&self) -> Option<&str> {
        self.primary_identity()
            .and_then(|i| i.avatar_url.as_deref())
            .or_else(|| self.identities.iter().find_map(|i| i.avatar_url.as_deref()))
    }

    /// Привязана ли конкретная платформа.
    pub fn identity(&self, provider: &str) -> Option<&UserIdentity> {
        self.identities.iter().find(|i| i.provider == provider)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FreezeInfo {
    pub reason: String,
    pub frozen_by: String,
    pub frozen_at: DateTime<Utc>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapeRow {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub file_sha1: String,
    pub size: i64,
    pub uploaded_by: Option<Uuid>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectCapeReq {
    pub cape_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCapesData {
    pub granted_cape_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUserCapesReq {
    pub granted_cape_ids: Vec<Uuid>,
    pub active_cape_id: Option<Uuid>,
}

impl UserProfile {
    /// Все эффективные права: прямые + из всех ролей, вместе с тем, что роли
    /// получили от своих родителей.
    pub fn all_permissions(&self) -> impl Iterator<Item = &str> {
        self.permissions
            .iter()
            .map(String::as_str)
            .chain(self.roles.iter().flat_map(|r| {
                r.permissions
                    .iter()
                    .chain(r.inherited_permissions.iter())
                    .map(String::as_str)
            }))
    }

    /// Есть ли у пользователя право (с учётом wildcard'ов).
    pub fn has_permission(&self, required: &str) -> bool {
        self.all_permissions()
            .any(|p| permission_matches(p, required))
    }

    /// Может ли войти на сервер с заданным id.
    pub fn can_join_server(&self, server_id: &Uuid, server_limited: bool) -> bool {
        if !server_limited {
            return true;
        }
        self.has_permission(&crate::permissions::perm_server_join(
            &server_id.to_string(),
        ))
    }

    /// Может ли включить опциональный мод.
    pub fn can_use_optional(&self, server_id: &Uuid, mod_name: &str, limited: bool) -> bool {
        if !limited {
            return true;
        }
        self.has_permission(&crate::permissions::perm_optional_mod(
            &server_id.to_string(),
            mod_name,
        ))
    }

    /// Цвет первой по приоритету (наибольший sort_order) роли — для UI.
    pub fn primary_color(&self) -> Option<&str> {
        self.roles
            .iter()
            .filter(|r| r.color.is_some())
            .max_by_key(|r| r.sort_order)
            .and_then(|r| r.color.as_deref())
    }
}
