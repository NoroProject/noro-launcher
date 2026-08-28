//! Мод → лаунчер: намерения.
//!
//! Ровно как `MessageToBackend`: мод говорит, чего хочет, и не знает, каким
//! запросом это делается. URL админки в моде нет и не будет — иначе правка
//! ручки ломала бы jar, который уже уехал к людям.
//!
//! Действий в мире здесь нет намеренно: телепорт, заморозку и слежку мод шлёт
//! теми же командами `/case …`, что и кнопки в чате. Серверный агент из-за
//! панели не пересобирается.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToLauncher {
    /// Первый кадр соединения: ключ из файла рукопожатия и версия договора.
    /// Без него не принимается ничего.
    Hello {
        key: String,
        protocol: u32,
    },
    /// Перечитать очередь с начала и без фильтра.
    ///
    /// Осталась ради модов, уехавших со старыми сборками: у них есть только этот
    /// кадр. Добавить сюда поля нельзя — `data` у кадра без полей отсутствует, и
    /// `{"type": "RequestQueue"}` от старого мода перестал бы разбираться вовсе.
    /// Новые моды шлют `RequestQueuePage`.
    RequestQueue,
    /// Страница очереди с поиском.
    ///
    /// Ищет и режет на страницы мастер: панель в игре показывает десяток дел за
    /// раз, а очередь на большом сервере в него не помещается. Пока фильтр жил в
    /// моде, он видел только загруженный кусок — и «дела нет» означало лишь «его
    /// нет на первой странице».
    RequestQueuePage {
        /// Ник, сервер, модератор или номер дела. Пусто — вся очередь.
        #[serde(default)]
        query: Option<String>,
        /// Сколько дел пропустить от начала очереди.
        #[serde(default)]
        offset: i64,
    },
    /// Открыть карточку. Лаунчер запомнит дело открытым и будет присылать его
    /// заново на каждое `CaseUpdated` с мастера.
    OpenCase {
        case_id: Uuid,
    },
    /// Закрыть карточку — перестать слать обновления по ней.
    CloseCase,
    Claim {
        case_id: Uuid,
    },
    Release {
        case_id: Uuid,
    },
    Resolve {
        case_id: Uuid,
        /// `confirmed`, `rejected` или `insufficient`.
        verdict: String,
        #[serde(default)]
        resolution: String,
        #[serde(default)]
        rule_code: Option<String>,
    },
    AddNote {
        case_id: Uuid,
        text: String,
    },
    Punish {
        case_id: Uuid,
        /// `mute`, `warn`, `kick`, `ban`.
        kind: String,
        reason: String,
        #[serde(default)]
        rule_code: Option<String>,
        /// Пусто — бессрочно, как и в админке.
        #[serde(default)]
        duration_secs: Option<i64>,
    },
    /// Попросить у сервера срез чата вокруг события. Ответ придёт не сюда, а
    /// обновлённой карточкой: срез снимает агент, и это занимает время.
    RequestChat {
        case_id: Uuid,
    },
    RequestInventory {
        case_id: Uuid,
    },
    /// Кадр экрана в дело. PNG едет base64: канал текстовый, а отдельный
    /// бинарный кадр ради одного случая усложнил бы обе стороны.
    Attach {
        case_id: Uuid,
        #[serde(default)]
        note: String,
        png_base64: String,
    },
    /// Указание на сообщение в чате, а не само сообщение.
    ///
    /// Клиент не источник доказательств: в дело поедет строка из `ChatRing`
    /// агента, найденная по отправителю и времени. Хеш — только чтобы понять,
    /// что мод и сервер говорят про одну и ту же строку.
    Quote {
        case_id: Uuid,
        sender: String,
        at: DateTime<Utc>,
        hash: String,
    },
    /// Досье игрока под прицелом.
    Lookup {
        username: String,
    },
    /// Свод правил. Публичный документ — прав на него не нужно.
    RequestRules,
    /// Свои наказания: что действует и когда кончится.
    RequestOwnPunishments,
}
