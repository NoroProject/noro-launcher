//! Единый тип ошибки API, конвертируемый в HTTP-ответ.
//!
//! Тело отказа — `{"error": {"code", "message"}}`. Код машинный и стабильный,
//! сообщение человеческое. Раньше отдавалась одна строка, и вид отказа был
//! склеен с причиной («forbidden: step_up_required»); клиентам приходилось
//! разбирать её подстрокой, а агент отдельно отрезал префикс, чтобы показать
//! модератору в чате причину без служебного слова.
//!
//! У отказа по форме добавляется `details` — список полей с их промахами: см.
//! `api::validate`. Форма подсвечивает виноватое поле, а не показывает абзац
//! текста под собой.

use crate::api::validate::FieldError;
use crate::error_codes::{self as codes, ErrorCode};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    /// Форма не прошла проверку. JSON разобрался, значения не годятся — это
    /// 422, а не 400: синтаксис в порядке, смысл нет.
    #[error("request validation failed")]
    Validation(Vec<FieldError>),
    /// Отказ с уточнённым кодом: когда внутри одного статуса клиенту нужно
    /// различать причины. Пример — `step_up_required` среди прочих 403: по нему
    /// админка открывает подтверждение passkey, а не показывает «нет прав».
    #[error("{message}")]
    Coded {
        status: StatusCode,
        code: ErrorCode,
        message: String,
    },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    /// Отказ со своим машинным кодом.
    pub fn coded(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::Coded {
            status,
            code,
            message: message.into(),
        }
    }

    /// `400` с уточнённым кодом — самая частая форма отказа по существу запроса.
    pub fn bad(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::coded(StatusCode::BAD_REQUEST, code, message)
    }

    /// `409`: с запросом всё в порядке, не в порядке состояние.
    pub fn state(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::coded(StatusCode::CONFLICT, code, message)
    }

    /// `502`: подвёл не клиент и не мы, а тот, к кому мы сходили.
    pub fn upstream(code: ErrorCode, message: impl Into<String>) -> Self {
        AppError::coded(StatusCode::BAD_GATEWAY, code, message)
    }

    /// Нарушение уникального индекса — это «такое уже есть», а не сбой сервера.
    ///
    /// Смотрим и в `Db`, и в `Other`: слой БД возвращает `anyhow::Result`, поэтому
    /// до сюда ошибка sqlx доезжает завёрнутой, и проверка только по `Db`
    /// молчала. Пока текст ошибки уходил клиенту, он хотя бы видел `duplicate
    /// key value violates unique constraint …`; теперь текст скрыт, и без этой
    /// ветки повтор имени роли выглядел бы как поломка мастера — 500 без единой
    /// подсказки, что исправлять.
    fn is_unique_violation(&self) -> bool {
        let sqlx_error = match self {
            AppError::Db(e) => Some(e),
            AppError::Other(e) => e.downcast_ref::<sqlx::Error>(),
            _ => None,
        };
        matches!(
            sqlx_error
                .and_then(|e| e.as_database_error())
                .and_then(|d| d.code())
                .as_deref(),
            Some("23505")
        )
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Coded { status, .. } => *status,
            AppError::Db(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            _ if self.is_unique_violation() => StatusCode::CONFLICT,
            AppError::Db(_) | AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Вид отказа: номер называет человек, слаг разбирает клиент.
    fn code(&self) -> ErrorCode {
        match self {
            AppError::NotFound(_) => codes::NOT_FOUND,
            AppError::Unauthorized(_) => codes::UNAUTHORIZED,
            AppError::Forbidden(_) => codes::FORBIDDEN,
            AppError::BadRequest(_) => codes::BAD_REQUEST,
            AppError::Conflict(_) => codes::CONFLICT,
            AppError::Validation(_) => codes::VALIDATION,
            AppError::Coded { code, .. } => *code,
            AppError::Db(sqlx::Error::RowNotFound) => codes::NOT_FOUND,
            _ if self.is_unique_violation() => codes::ALREADY_EXISTS,
            AppError::Db(_) | AppError::Other(_) => codes::INTERNAL,
        }
    }
}

/// JSON-ошибка внутри хендлера — это наша поломка (не разобрали собственное
/// сохранённое состояние), а не плохой запрос. Тело запроса разбирает
/// экстрактор `Json`, и его отказ сюда не попадает.
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Other(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();

        // Текст внутренней ошибки наружу не идёт. `Db` и `Other` прозрачны, то
        // есть их Display — это сообщение sqlx или anyhow целиком: имена таблиц
        // и уникальных индексов («duplicate key value violates unique
        // constraint "users_username_key"»), пути внутри контейнера, содержимое
        // команд. Клиенту от этого нет пользы, а карту базы оно выдаёт даром.
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, code = code.slug, number = code.number, "internal error");
            "internal error".to_string()
        } else if code == codes::ALREADY_EXISTS {
            // Имя индекса тоже не показываем: клиенту хватает «уже существует»,
            // а какая именно колонка уникальна — устройство нашей схемы.
            tracing::info!(error = %self, "нарушен уникальный индекс");
            "such a record already exists".to_string()
        } else {
            self.to_string()
        };

        // Номер идёт первым: его называет человек, слаг разбирает код.
        let mut error = json!({ "number": code.number, "code": code.slug, "message": message });
        // `details` появляется только у отказа по форме: пустой список в каждом
        // ответе клиенту нечего разбирать, а «поле есть — значит есть промахи»
        // проверяется одним условием.
        if let AppError::Validation(fields) = &self {
            error["details"] = json!(fields);
        }

        (status, Json(json!({ "error": error }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
