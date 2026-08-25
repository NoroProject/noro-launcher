//! WebSocket админки в браузере.
//!
//! Отдельный протокол от лаунчерного, а не общий с ним. Общий выглядел бы
//! дешевле, но тогда браузер получал бы `LogRequest` и `ImpersonateRequest` —
//! диалоги, которые существуют именно как второй фактор «злоумышленник добрался
//! до веб-сессии, но не до машины». Отдать их в веб значит убрать этот фактор.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum AdminWsClientMsg {
    /// Тот же токен сессии, что у REST-запросов админки.
    Authenticate {
        access_token: String,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum AdminWsMsg {
    AuthOk,
    AuthFail,
    /// Карточка дела изменилась. Данных нет: страница сходит за ней сама тем же
    /// запросом, что и раньше, — и права проверятся там, а не здесь.
    CaseUpdated {
        case_id: Uuid,
    },
    Pong,
}

impl AdminWsMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("AdminWsMsg сериализуется")
    }
}
