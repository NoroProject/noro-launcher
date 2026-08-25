//! Шаблоны сообщений о наказаниях: экран кика, отказ в чате, текст варна.
//!
//! Живут в `instance_settings` одним JSON, а не полями конфига: их правят из
//! админки на живом сервере, а перезапуск мастера ради формулировки бана —
//! слишком дорогая цена за запятую.
//!
//! Подстановки: `{player}`, `{reason}`, `{duration}`, `{expires}`, `{actor}`,
//! `{rule}`, `{rule_title}`, `{id}`, `{kind}`. Рисует их агент — мастер отдаёт
//! шаблоны как есть и ничего в них не дописывает.
//!
//! Разметка та же, что понимает агент: MiniMessage — `<#rrggbb>`, `<red>`,
//! прочие коды стиля, `[текст](ссылка)` и сырые URL — кликабельными их делает
//! уже игровой клиент.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ключ в `instance_settings`.
pub const SETTINGS_KEY: &str = "moderation_messages";

/// Тексты по умолчанию.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ModerationMessages {
    pub ban_permanent: String,
    pub ban_temporary: String,
    pub server_ban_permanent: String,
    pub server_ban_temporary: String,
    pub mute_permanent: String,
    pub mute_temporary: String,
    pub mute_actionbar_permanent: String,
    pub mute_actionbar_temporary: String,
    pub warn_actionbar: String,
    pub warn_notice: String,
    pub broadcast: String,
    pub actor_receipt: String,
    pub reason_by_rule: String,
    pub no_account: String,
    pub no_access: String,
    pub maintenance: String,
}

impl Default for ModerationMessages {
    fn default() -> Self {
        Self::default_en()
    }
}

/// Дополнить сохранённое умолчаниями **своего** языка.
///
/// Через `serde` этого не сделать: контейнерный `#[serde(default)]` берёт
/// `Default`, а он английский. Из-за этого шаблон, которого нет в базе, приезжал
/// по-английски даже в русском наборе — а не хватало их семи из семнадцати,
/// потому что набор рос, а сохранённое значение осталось от старой версии.
fn merge(lang: &str, stored: serde_json::Value) -> ModerationMessages {
    let base = if lang.starts_with("ru") {
        ModerationMessages::default_ru()
    } else {
        ModerationMessages::default_en()
    };
    let Ok(mut merged) = serde_json::to_value(&base) else {
        return base;
    };
    if let (Some(target), Some(source)) = (merged.as_object_mut(), stored.as_object()) {
        for (key, value) in source {
            // Пустую строку сохраняем: у `broadcast` это законный выбор
            // «не объявлять», и подменять его умолчанием нельзя.
            if !value.is_null() {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    serde_json::from_value(merged).unwrap_or(base)
}

/// Загрузить карту шаблонов по языкам.
pub async fn load_map(pool: &sqlx::PgPool) -> HashMap<String, ModerationMessages> {
    let mut map = HashMap::new();
    map.insert("en".to_string(), ModerationMessages::default_en());
    map.insert("ru".to_string(), ModerationMessages::default_ru());

    let stored = match crate::db::get_setting(pool, SETTINGS_KEY).await {
        Ok(Some(value)) => value,
        _ => return map,
    };

    // Старый плоский формат — единственный набор без языков. Кладём его в
    // русский: до появления словаря умолчания были русскими, и тексты в базе
    // тоже. Отдать их как английские значит показать русский экран бана
    // англоязычному игроку, а русскому — нетронутые умолчания.
    if stored.get("ban_permanent").is_some() {
        map.insert("ru".to_string(), merge("ru", stored));
    } else if let Ok(parsed) = serde_json::from_value::<HashMap<String, serde_json::Value>>(stored)
    {
        for (lang, value) in parsed {
            let merged = merge(&lang, value);
            map.insert(lang, merged);
        }
    }
    map
}

pub async fn load_for_lang(pool: &sqlx::PgPool, lang: Option<&str>) -> ModerationMessages {
    let map = load_map(pool).await;
    let clean = lang
        .map(|s| s.split(['-', '_']).next().unwrap_or(s).to_lowercase())
        .unwrap_or_else(|| "en".to_string());
    if let Some(msg) = map.get(&clean) {
        return msg.clone();
    }
    if let Some(msg) = map.get("en") {
        return msg.clone();
    }
    ModerationMessages::default_en()
}

pub async fn load(pool: &sqlx::PgPool) -> ModerationMessages {
    load_for_lang(pool, None).await
}

#[derive(Serialize)]
pub struct AgentMessages {
    #[serde(flatten)]
    pub messages: ModerationMessages,
    pub rules_url: String,
}

#[derive(Deserialize)]
pub struct AgentMessagesQuery {
    pub lang: Option<String>,
    pub locale: Option<String>,
}

pub async fn agent_messages(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _agent: crate::api::auth::AgentAuth,
    axum::extract::Query(query): axum::extract::Query<AgentMessagesQuery>,
) -> axum::Json<AgentMessages> {
    let lang = query.lang.or(query.locale);
    axum::Json(AgentMessages {
        messages: load_for_lang(&state.db, lang.as_deref()).await,
        rules_url: format!("{}/rules", state.config.web_url.trim_end_matches('/')),
    })
}
