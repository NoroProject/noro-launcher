//! Выгрузка резервной копии мастера.
//!
//! Игровые сервера бэкапятся через враппер, а сама база — нет, хотя в ней
//! личности, привязка Discord→MC, права, скины и плащи. Потеря тома означала бы,
//! что у всех игроков одновременно слетают доступы и косметика.
//!
//! Две ручки: `backup` — только дамп БД, `backup/full` — архив целиком, вместе
//! с файлами `NORO_DATA_DIR` и подписью. Первая осталась как быстрый способ
//! забрать одну базу, когда файлы копируются иначе.
//!
//! Ничего не сохраняется на сервере: и то и другое стримится в ответ и оседает
//! у администратора. Копия на том же томе, что и данные, не пережила бы ровно
//! тот отказ, ради которого делается.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Response;
use axum::Json;
use schema::PERM_BACKUP;
use serde_json::{json, Value};

use tokio::process::Command;

/// `GET /api/admin/backup` — дамп в формате `custom` (уже сжат, разворачивается
/// через `pg_restore`).
pub async fn download(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Response> {
    admin.require(PERM_BACKUP)?;

    let mut child = Command::new("pg_dump")
        .arg("--format=custom")
        .arg("--no-owner")
        .arg("--no-acl")
        .arg(&state.config.database_url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::Other(anyhow::anyhow!(
                "не удалось запустить pg_dump ({e}). Он есть в образе мастера?"
            ))
        })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("pg_dump produced no stdout")))?;

    // Ошибки pg_dump уходят в лог: тело ответа уже начало отдаваться, и
    // сообщить о них в HTTP-статусе нельзя.
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = String::new();
            let mut stderr = stderr;
            if stderr.read_to_string(&mut buf).await.is_ok() && !buf.trim().is_empty() {
                tracing::error!(stderr = %buf.trim(), "pg_dump сообщил об ошибке");
            }
        });
    }
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) if !status.success() => {
                tracing::error!(?status, "pg_dump завершился с ошибкой — дамп неполный")
            }
            Err(e) => tracing::error!(error = %e, "pg_dump не дождался завершения"),
            _ => tracing::info!("дамп БД выгружен"),
        }
    });

    let name = format!("noro-{}.dump", chrono::Utc::now().format("%Y-%m-%d-%H%M%S"));
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{name}\""),
        )
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(stdout)))
        .map_err(|e| AppError::Other(e.into()))
}

/// `GET /api/admin/backup/full` — `noro-backup-{дата}.tar.gz`: дамп, файлы
/// данных, паспорт и подпись. Для скриптов и CLI, где заголовок поставить есть кому.
pub async fn full(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Response> {
    admin.require(PERM_BACKUP)?;
    crate::backup::build::run(&state).await
}

/// `POST /api/admin/backup/ticket` — одноразовая ссылка для браузера.
pub async fn ticket(admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_BACKUP)?;
    Ok(Json(json!({ "ticket": crate::backup::ticket::issue() })))
}

/// `GET /api/admin/backup/full/{ticket}` — то же самое, но без заголовка:
/// браузер идёт по ссылке сам и качает потоком на диск. Право проверено при
/// выдаче билета, здесь остаётся только погасить его.
pub async fn full_by_ticket(
    State(state): State<AppState>,
    Path(ticket): Path<uuid::Uuid>,
) -> AppResult<Response> {
    if !crate::backup::ticket::consume(ticket) {
        return Err(AppError::Forbidden(
            "ссылка недействительна или просрочена — нажмите кнопку ещё раз".into(),
        ));
    }
    crate::backup::build::run(&state).await
}
