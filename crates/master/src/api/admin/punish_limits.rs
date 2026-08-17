//! Рамки, в которых модератор выдаёт наказание.
//!
//! Смысл всей конструкции: хелперу выдают право на мут, а свод правил говорит,
//! сколько именно он может дать. Всё, что шире, требует отдельного права —
//! иначе «ограничение» держалось бы на договорённости, а не на проверке.

use crate::db::rules::RuleSanctionRow;
use crate::error::{AppError, AppResult};
use schema::{perm_punish, PERM_PUNISH_BYPASS, PERM_PUNISH_PERMANENT};

/// Что просит выдать модератор.
pub struct Request<'a> {
    pub kind: &'a str,
    /// `None` — навсегда.
    pub minutes: Option<i64>,
    pub rule_cited: bool,
}

/// Права проверяющего приходят замыканием: так проверку можно прогнать в
/// тестах, не поднимая ни базы, ни сессии.
pub fn check(
    request: &Request<'_>,
    sanctions: &[RuleSanctionRow],
    has: impl Fn(&str) -> bool,
) -> AppResult<()> {
    if !has(&perm_punish(request.kind)) {
        return Err(AppError::Forbidden(format!(
            "issuing a {} is not allowed for you",
            request.kind
        )));
    }

    let permanent = request.minutes.is_none() && request.kind != "warn";
    if permanent && !has(PERM_PUNISH_PERMANENT) && !has(PERM_PUNISH_BYPASS) {
        return Err(AppError::Forbidden(
            "punishing forever needs noro.mod.punish.permanent".into(),
        ));
    }

    // Байпас снимает рамки свода целиком — это и есть право старшего
    // модератора решать по обстоятельствам.
    if has(PERM_PUNISH_BYPASS) {
        return Ok(());
    }

    if !request.rule_cited {
        return Err(AppError::Forbidden(
            "cite a rule: without noro.mod.punish.bypass a punishment must refer to one".into(),
        ));
    }
    if sanctions.is_empty() {
        return Err(AppError::Forbidden(
            "this rule sets no punishment, so only noro.mod.punish.bypass can issue one".into(),
        ));
    }
    if sanctions
        .iter()
        .any(|s| s.kind == request.kind && s.allows(request.minutes))
    {
        return Ok(());
    }
    Err(AppError::Forbidden(format!(
        "the rule allows only: {}",
        describe(sanctions)
    )))
}

/// Человеческий список допустимого — он попадает в ответ, и по нему модератор
/// понимает, что именно исправить, вместо голого «403».
pub fn describe(sanctions: &[RuleSanctionRow]) -> String {
    sanctions
        .iter()
        .map(|s| {
            let range = match (s.min_minutes, s.max_minutes) {
                _ if s.kind == "warn" => String::new(),
                (None, None) => " any term".into(),
                (Some(min), None) => format!(" from {}", minutes(min)),
                (None, Some(max)) => format!(" up to {}", minutes(max)),
                (Some(min), Some(max)) if min == max => format!(" {}", minutes(min)),
                (Some(min), Some(max)) => format!(" {}–{}", minutes(min), minutes(max)),
            };
            format!("{}{range}", s.kind)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Минуты словами: `10080` → `7d`. Тот же формат, что понимает поле срока.
fn minutes(value: i64) -> String {
    for (unit, size) in [("y", 525_600), ("mo", 43_200), ("d", 1_440), ("h", 60)] {
        if value % size == 0 {
            return format!("{}{unit}", value / size);
        }
    }
    format!("{value}m")
}

#[cfg(test)]
#[path = "punish_limits_tests.rs"]
mod tests;
