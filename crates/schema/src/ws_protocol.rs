//! WebSocket-протокол между лаунчером и мастером.
//!
//! Сериализуется в JSON-текстовые фреймы. Тег `t` определяет вариант.

use crate::build::BuildManifest;
use crate::launcher::LauncherVersion;
use crate::news::NewsItem;
use crate::server::ServerEntry;
use crate::user::UserProfile;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum ClientWsMsg {
    Authenticate {
        access_token: String,
        /// Версия и платформа лаунчера — чтобы админка видела, кто на чём сидит.
        /// С `default`: лаунчеры, выпущенные до появления полей, продолжают
        /// авторизовываться, просто без этих данных.
        #[serde(default)]
        launcher_version: String,
        #[serde(default)]
        platform: String,
    },
    RequestServerList,
    RequestNews,
    RequestBuildManifest {
        server_id: Uuid,
        /// Какую версию собирать. `None` — текущая опубликованная; так шлют
        /// лаунчеры, выпущенные до появления выбора версии.
        #[serde(default)]
        build_id: Option<Uuid>,
    },
    SetOptionalMods {
        server_id: Uuid,
        enabled: Vec<String>,
    },
    ReportGameStart {
        server_id: Uuid,
    },
    /// Сверка каталога с манифестом перед запуском. Отправляется всегда, в том
    /// числе когда всё сошлось: молчание неотличимо от «лаунчер не проверял».
    ReportIntegrity {
        report: crate::integrity::IntegrityReport,
    },
    ReportGameStop {
        server_id: Uuid,
        playtime_secs: u64,
    },
    /// Диагностика по запросу мастера.
    DiagnosticsReport {
        report: crate::integrity::DiagnosticsReport,
    },
    /// Ответ на запрос логов. `accepted: false` — игрок отказал.
    LogRequestResponse {
        request_id: Uuid,
        accepted: bool,
    },
    /// Ответ на диалог «войти в аккаунт игрока».
    ImpersonateResponse {
        grant_id: Uuid,
        accepted: bool,
    },
    Ping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotifLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum ServerWsMsg {
    /// Собрать и прислать диагностику. Личного в ней нет, согласия не нужно.
    RequestDiagnostics,
    /// Выполнить действие на стороне лаунчера.
    RemoteAction {
        action: crate::integrity::RemoteAction,
        /// Сборка, к которой относится действие.
        #[serde(default)]
        server_id: Option<Uuid>,
        /// Кто попросил — игрок увидит это в подтверждении.
        actor_username: String,
    },
    /// Админ просит логи. Модалка показывает, кто и зачем, а игрок решает.
    ///
    /// `forced` — сбор без согласия: у админа отдельное право на это, а игрок
    /// видит, что логи уже уехали. Прятать это незачем — журнал всё равно
    /// покажет, а честность здесь дешевле недоверия.
    LogRequest {
        request_id: Uuid,
        actor_username: String,
        reason: String,
        forced: bool,
        server_id: Option<Uuid>,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    /// Админ нажал «Login as» в вебе — лаунчер спрашивает подтверждение.
    ///
    /// Нативный диалог здесь второй фактор: он закрывает случай
    /// «злоумышленник получил веб-сессию админа, но не доступ к его машине».
    ImpersonateRequest {
        grant_id: Uuid,
        actor_username: String,
        target_username: String,
        reason: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    AuthOk {
        user: UserProfile,
    },
    AuthFail {
        /// Ключ перевода причины отказа.
        reason: String,
    },
    ServerList {
        servers: Vec<ServerEntry>,
    },
    News {
        items: Vec<NewsItem>,
    },
    /// Манифест подписан ed25519.
    BuildManifest {
        manifest: BuildManifest,
    },
    LauncherUpdate {
        version: LauncherVersion,
    },
    /// Уведомление ключом, а не готовым текстом: перевод живёт в лаунчере,
    /// иначе язык интерфейса не влиял бы на сообщения от мастера.
    Notification {
        key: String,
        #[serde(default)]
        args: std::collections::BTreeMap<String, String>,
        level: NotifLevel,
    },
    ServersChanged,
    NewsChanged,
    /// Каталог перевода изменился — лаунчер перекачает свой язык.
    TranslationsChanged,
    BuildsChanged {
        server_id: Uuid,
    },
    PermissionsUpdated {
        user: UserProfile,
    },
    /// Карточка дела изменилась. Данных нет намеренно: кто смотрит на дело,
    /// тот сходит за карточкой сам — иначе кадр пришлось бы держать в согласии
    /// с формой ответа `/api/admin/cases/{id}`, а она меняется чаще протокола.
    ///
    /// Шлётся модератору, за которым замок.
    CaseUpdated {
        case_id: Uuid,
    },
    Pong,
}

impl ClientWsMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ClientWsMsg сериализуется")
    }
}

impl ServerWsMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ServerWsMsg сериализуется")
    }
}
