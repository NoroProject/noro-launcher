//! Кадры канала между мастером и агентом игрового сервера.
//!
//! Вниз (`ToAgent`) идёт «вот что изменилось, примени сейчас»; наказания
//! по-прежнему выдаются обычным HTTP, потому что на них нужен ответ с телом.
//! Вверх (`FromAgent`) — события игры, которых мастеру иначе не увидеть.
//!
//! Игрока адресуем MC UUID, а не `users.id`: агент знает игрока только таким, и
//! перекладывать сопоставление на него значит завести на игровой стороне вторую
//! копию базы.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Наказание в том виде, в каком его показывают игроку.
#[derive(Serialize, Clone, Debug)]
pub struct LivePunishment {
    pub id: Uuid,
    /// MC UUID наказанного.
    pub target: Uuid,
    pub target_name: String,
    /// `ban` | `server_ban` | `mute` | `warn`.
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    /// `None` — навсегда.
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_code: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToAgent {
    /// Наказание выдано или продлено — применить немедленно.
    Punished {
        punishment: LivePunishment,
    },
    /// Наказание снято. `kind` нужен, чтобы агент понял, что именно отпустить:
    /// снятый мут возвращает чат, снятый бан не делает ничего с онлайном.
    Revoked {
        id: Uuid,
        target: Uuid,
        target_name: String,
        kind: String,
        actor_label: String,
    },
    /// Настройки сообщений или автомодерации поменяли в админке — перечитать их с мастера.
    MessagesChanged,
    FiltersChanged,
    RestartNotice {
        seconds: u32,
        reason: Option<String>,
    },
    /// Роли, права или префиксы игрока изменились — перечитать профиль.
    ///
    /// `None` — «перечитать всех»: так уходят правки самой роли, которые
    /// касаются каждого её носителя. Агент берёт таких батчем, иначе на сервере
    /// с сотней игроков это была бы сотня запросов подряд.
    ProfileChanged {
        uuid: Option<Uuid>,
    },
    /// Кикнуть игрока с сервера с указанным текстом.
    Kick {
        target: Uuid,
        message: String,
    },
    /// Написать личное сообщение игроку.
    Tell {
        target: Uuid,
        message: String,
    },
    /// Объявление на весь сервер.
    Announce {
        message: String,
    },
    /// Начались техработы: предупредить и через `countdown_seconds` кикнуть всех (кроме имеющих право bypass).
    MaintenanceStart {
        countdown_seconds: u32,
        reason: Option<String>,
    },
    /// Техработы отменены.
    MaintenanceCancel,
    /// Дело взяли в работу — выдать модератору режим разбора, если он в игре.
    ///
    /// Всё нужное для меню едет кадром: агент не ходит на мастер за карточкой,
    /// иначе меню открывалось бы с задержкой ровно тогда, когда сервер занят.
    CaseAssigned {
        case: Uuid,
        target: Uuid,
        target_name: String,
        moderator: Uuid,
        reason: String,
        world: Option<String>,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        reporter: Option<Uuid>,
        reporter_name: Option<String>,
    },
    /// Дело отпустили или закрыли — выйти из режима, вернуть модератора назад.
    CaseFinished {
        case: Uuid,
        moderator: Uuid,
        closed: bool,
    },
    /// Прислать срез чата вокруг события. `before_secs` — сколько отмотать
    /// назад по буферу агента.
    CaseChatRequest {
        case: Uuid,
        target: Uuid,
        before_secs: u32,
    },
    /// Прислать снимок инвентаря цели.
    CaseInventoryRequest {
        case: Uuid,
        target: Uuid,
    },
}

/// Кадры агент → мастер.
///
/// **Событие — не источник истины.** Канал рвётся, кадры теряются, поэтому
/// ничего необратимого по одному кадру не делается: состав онлайна сверяется по
/// heartbeat, а событие нужно лишь чтобы отреагировать быстро. Неизвестные
/// кадры мастер молча пропускает — старый мастер должен переживать нового
/// агента.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromAgent {
    /// Игрок вошёл. `ip_hash` — уже хеш: сырой адрес мастеру не нужен.
    PlayerJoin {
        uuid: Uuid,
        #[serde(default)]
        ip_hash: Option<String>,
        /// Скрыт ванишем — в публичном онлайне его быть не должно.
        #[serde(default)]
        vanished: bool,
    },
    /// Игрок вышел. `reason` — свободный текст для журнала.
    PlayerLeave {
        uuid: Uuid,
        #[serde(default)]
        reason: Option<String>,
    },
    /// Игровой поток не двигался `stalled_secs` секунд.
    TickStall { stalled_secs: u32 },
    /// Модератор взял дело из игры. Замок ставит мастер: команда в игре и
    /// кнопка на сайте одинаково могут проиграть гонку.
    CaseClaim { case: Uuid, moderator: Uuid },
    /// Что модератор сделал в режиме разбора: телепорт, заморозка, слежка.
    /// `payload` кладётся в ленту как есть — форму знает тот, кто рисует.
    CaseAction {
        case: Uuid,
        moderator: Uuid,
        kind: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
    /// Срез чата по запросу или по факту жалобы.
    CaseChatSlice {
        case: Uuid,
        messages: Vec<crate::db::cases::IncomingMessage>,
    },
    /// Снимок инвентаря цели: доказательство дюпа живёт в деле, а не в памяти.
    CaseInventory {
        case: Uuid,
        moderator: Option<Uuid>,
        items: serde_json::Value,
    },
}
