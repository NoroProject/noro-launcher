//! Что агент видит в профиле игрока. Формы ответов вынесены из обработчиков:
//! на них смотрят с трёх сторон — мастер, три реализации агента и тесты
//! подписи, — и искать их среди запросов к базе неудобно.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct AgentRole {
    pub name: String,
    pub display_name: String,
    /// Группа LuckPerms. `None` — роль в игру не проецируется.
    pub lp_group: Option<String>,
    pub color: Option<String>,
    /// Один глиф рядом с ником — не префикс. Префикс живёт отдельно, потому
    /// что это произвольная строка, а иконка обязана влезать в таб.
    pub icon: Option<String>,
    /// Что ставится перед ником в игре, с цветами через `&`. Пусто — агент
    /// подставит иконку в цвете роли, как было до появления этого поля.
    pub prefix: Option<String>,
    /// Что ставится после ника.
    pub suffix: Option<String>,
    /// Больше — важнее. Совпадает с весом группы в LuckPerms.
    pub sort_order: i32,
}

#[derive(Serialize)]
pub struct AgentPunishmentSummary {
    pub id: Uuid,
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_code: Option<String>,
}

impl From<crate::db::punishments::PunishmentRow> for AgentPunishmentSummary {
    fn from(row: crate::db::punishments::PunishmentRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            reason: row.reason,
            actor_label: row.actor_label,
            created_at: row.created_at,
            expires_at: row.expires_at,
            rule_code: row.rule_code,
        }
    }
}

#[derive(Serialize)]
pub struct AgentPlayer {
    pub uuid: Uuid,
    pub username: String,
    pub banned: bool,
    pub muted: bool,
    pub active_mute: Option<AgentPunishmentSummary>,
    /// Предупреждения, которые игрок ещё не видел. Агент показывает их при
    /// входе и подтверждает через `/punishments/{id}/ack` — иначе варн
    /// оставался бы записью в базе, о которой наказанный не знает.
    pub pending_warns: Vec<AgentPunishmentSummary>,
    /// Игроку разрешён вход на этот сервер. Агент обязан проверить: манифест
    /// сборки — не пропуск, до сервера можно дойти и мимо лаунчера.
    pub allowed: bool,
    pub roles: Vec<AgentRole>,
    pub skin_url: String,
    pub cape_url: Option<String>,
    /// Группы LuckPerms в порядке важности — готовый результат для агента,
    /// чтобы он не повторял у себя логику отбора.
    pub lp_groups: Vec<String>,
    /// Права, действующие на этом сервере: свои и от ролей, глобальные и
    /// привязанные к сборке. Плюс узлы `prefix.<вес>.<значение>` — LuckPerms
    /// хранит префикс так же, и моды, читающие права напрямую, ищут именно там.
    pub permissions: Vec<String>,
}
