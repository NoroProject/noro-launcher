//! Выбранный игроком язык.
//!
//! Отдельно от остального кабинета: сюда пишет не человек в браузере, а
//! лаунчер — он единственный, кто знает, на каком языке игрок пользуется
//! программой, и делает это молча при смене языка.
//!
//! Зачем вообще хранить: внутри игры язык берёт сам клиент MC, но на экране
//! отказа при входе клиент ещё ничего не сообщил. Там причина бана и нужна
//! понятной больше всего.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LocaleReq {
    /// Код языка: `ru`, `en`, `pt-br`. `null` — сбросить к языку по умолчанию.
    pub locale: Option<String>,
}

/// `PUT /api/me/locale`
pub async fn set_locale(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<LocaleReq>,
) -> AppResult<Json<serde_json::Value>> {
    let locale = match req.locale {
        Some(raw) => Some(normalize(&raw)?),
        None => None,
    };
    sqlx::query("UPDATE users SET locale = $2 WHERE id = $1")
        .bind(user.user_id)
        .bind(&locale)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({ "locale": locale })))
}

/// Приводим к виду `ru` / `pt-br` и отсекаем всё, что не похоже на язык.
///
/// Список допустимых кодов не проверяем: переводы добавляются в админке, и
/// мастер не должен знать заранее, какие из них уже набраны. Неизвестный код
/// безвреден — сообщение просто откатится на язык по умолчанию.
fn normalize(raw: &str) -> AppResult<String> {
    let value = raw.trim().to_ascii_lowercase().replace('_', "-");
    let shaped = match value.split_once('-') {
        Some((lang, region)) => {
            is_alpha(lang, 2..=3) && is_alpha(region, 2..=3) && !value.contains("--")
        }
        None => is_alpha(&value, 2..=3),
    };
    if !shaped {
        return Err(AppError::BadRequest(
            "locale: language code like `ru` or `pt-br`".into(),
        ));
    }
    Ok(value)
}

fn is_alpha(part: &str, len: std::ops::RangeInclusive<usize>) -> bool {
    len.contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn accepts_common_shapes() {
        assert_eq!(normalize("RU").unwrap(), "ru");
        assert_eq!(normalize(" pt_BR ").unwrap(), "pt-br");
    }

    #[test]
    fn rejects_junk() {
        for bad in ["", "r", "russian-federation-extra", "ru-", "12"] {
            assert!(normalize(bad).is_err(), "{bad} должен быть отвергнут");
        }
    }
}
