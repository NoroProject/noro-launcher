//! Договор между лаунчером и клиентским модом разбора.
//!
//! Устроен как `crates/bridge`: мод шлёт намерение (`ToLauncher`) и рисует
//! состояние, которое ему прислали (`ToMod`). Ни request-id, ни корреляции
//! ответов здесь нет — их нет и у фронта лаунчера, потому что мод не
//! «запрашивает и ждёт», а подписан.
//!
//! Отличие от внутрипроцессного bridge одно, но важное: версии расходятся. Мод
//! приезжает со сборкой и живёт у людей месяцами, лаунчер обновляется сам.
//! Отсюда `PROTOCOL`, `#[serde(default)]` на новых полях и правило «незнакомый
//! кадр логируется и пропускается» — тем же приёмом держится `ClientWsMsg`.

mod case;
mod dossier;
mod player;
mod to_launcher;
mod to_mod;
#[cfg(test)]
mod wire_tests;

pub use case::{
    CaseBrief, CaseEvent, CaseMessage, CasePunishment, CaseReport, CaseView, InventorySlot,
};
pub use dossier::Dossier;
pub use player::{OwnPunishment, RuleCategory, RuleItem, RuleSanction};
pub use to_launcher::ToLauncher;
pub use to_mod::ToMod;

/// Версия договора. Растёт, когда старый мод перестаёт понимать новый лаунчер;
/// добавление поля с `default` или кадра — не тот случай.
pub const PROTOCOL: u32 = 1;

/// Сколько дел в странице очереди.
///
/// Панель разбора показывает список в углу экрана — больше десятка строк туда
/// не помещается, а тянуть всю очередь ради десяти видимых незачем. Знают
/// значение обе стороны: мод считает по нему номера страниц.
pub const QUEUE_PAGE: i64 = 10;

/// Имя файла рукопожатия в каталоге инстанса.
///
/// Лежит рядом с игрой, а не в конфиге лаунчера: мод знает только свой
/// `gameDir` и не должен угадывать, где установлен лаунчер.
pub const HANDSHAKE_FILE: &str = "noro-bridge.json";

/// Содержимое файла рукопожатия.
///
/// Ключ обязателен: сокет открыт наружу процесса, и чужая веб-страница может
/// постучаться на `ws://127.0.0.1:port` — CORS на WebSocket не распространяется.
/// Прочитать файл в каталоге инстанса она при этом не может.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub port: u16,
    pub key: String,
    pub protocol: u32,
}
