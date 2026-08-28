//! WebSocket for the admin panel in the browser.
//!
//! Deliberately separate from the launcher protocol rather than shared with it.
//! Sharing would hand the browser `LogRequest` and `ImpersonateRequest`, and
//! those prompts exist precisely as the second factor for "attacker has the web
//! session but not the machine".

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum AdminWsClientMsg {
    /// Same session token the admin REST endpoints take.
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
    /// Carries no payload: the page refetches through the usual endpoint, which
    /// is where permissions get checked.
    CaseUpdated {
        case_id: Uuid,
    },
    Pong,
}

impl AdminWsMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("AdminWsMsg is serializable")
    }
}
