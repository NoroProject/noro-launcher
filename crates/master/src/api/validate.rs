//! Проверка присланных полей.
//!
//! Отказ по форме — это не «что-то не так с запросом», а «вот это поле и вот
//! почему». Раньше правило и имя поля склеивались в одну строку («username:
//! 3-16 chars, latin letters, digits, or underscore»), и подсветить виноватое
//! поле в форме было нечем: клиенту доставался абзац текста под всей формой.
//!
//! Собираем все промахи разом, а не падаем на первом: человек, заполнивший три
//! поля неверно, должен увидеть три подсказки, а не исправлять их по одной,
//! отправляя форму каждый раз заново.

use crate::error::AppError;
use serde::Serialize;

/// Промах в одном поле. `code` машинный, `message` человеческий.
#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Validation {
    errors: Vec<FieldError>,
}

impl Validation {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, field: &str, code: &'static str, message: impl Into<String>) {
        self.errors.push(FieldError {
            field: field.to_string(),
            code,
            message: message.into(),
        });
    }

    /// Непустое после обрезки пробелов: строка из одних пробелов — это не имя.
    pub fn required(&mut self, field: &str, value: &str) -> &mut Self {
        if value.trim().is_empty() {
            self.push(field, "required", "this field is required");
        }
        self
    }

    /// Длина в символах, а не в байтах: `Модератор` — девять символов, и по
    /// байтам он не влезал бы в границу, рассчитанную на латиницу.
    pub fn max_len(&mut self, field: &str, value: &str, max: usize) -> &mut Self {
        if value.chars().count() > max {
            self.push(
                field,
                "too_long",
                format!("must be at most {max} characters"),
            );
        }
        self
    }

    pub fn len_between(&mut self, field: &str, value: &str, min: usize, max: usize) -> &mut Self {
        let len = value.chars().count();
        if len < min || len > max {
            self.push(
                field,
                "out_of_range",
                format!("must be {min} to {max} characters"),
            );
        }
        self
    }

    /// Значение из закрытого списка. Список попадает в сообщение: клиент не
    /// обязан знать его наизусть, а без перечисления «unknown kind» ничего не
    /// объясняет.
    pub fn one_of(&mut self, field: &str, value: &str, allowed: &[&str]) -> &mut Self {
        if !allowed.contains(&value) {
            self.push(
                field,
                "not_allowed",
                format!("must be one of: {}", allowed.join(", ")),
            );
        }
        self
    }

    /// Срок, счётчик, порог — строго больше нуля, когда задан.
    pub fn positive(&mut self, field: &str, value: Option<i64>) -> &mut Self {
        if matches!(value, Some(v) if v <= 0) {
            self.push(field, "out_of_range", "must be greater than zero");
        }
        self
    }

    /// Своё правило, которое не выражается остальными.
    pub fn rule(
        &mut self,
        field: &str,
        ok: bool,
        code: &'static str,
        message: impl Into<String>,
    ) -> &mut Self {
        if !ok {
            self.push(field, code, message);
        }
        self
    }

    /// Проверка не про одно поле, а про их набор: «заполните хотя бы одно».
    pub fn any_of(&mut self, fields: &str, present: bool) -> &mut Self {
        if !present {
            self.push(fields, "required", "fill at least one of these");
        }
        self
    }

    /// `Ok(())` — промахов нет.
    pub fn finish(&mut self) -> Result<(), AppError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(std::mem::take(&mut self.errors)))
        }
    }
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
