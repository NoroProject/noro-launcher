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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    /// Внутренний UUID пользователя в БД мастера. Нужен админке, CLI и ACL-операциям.
    pub id: Uuid,
    /// MC UUID (UUID v5 из discord_id).
    pub uuid: Uuid,
    /// MC ник.
    pub username: String,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub skin_url: Option<String>,
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
