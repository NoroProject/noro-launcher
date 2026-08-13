//! Выгрузка дампа БД мастера.
//!
//! Игровые сервера бэкапятся через враппер, а сама база — нет, хотя в ней
//! личности, привязка Discord→MC, права, скины и плащи. Потеря тома означала бы,
//! что у всех игроков одновременно слетают доступы и косметика.
//!
//! Дамп никуда не сохраняется на сервере: он стримится в ответ и оседает у
//! администратора. Копия на том же томе, что и данные, не пережила бы ровно тот
//! отказ, ради которого делается.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::header;
use axum::response::Response;
use schema::PERM_ADMIN_BACKUP;
use tokio::process::Command;

/// `GET /api/admin/backup` — дамп в формате `custom` (уже сжат, разворачивается
/// через `pg_restore`).
pub async fn download(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Response> {
    admin.require(PERM_ADMIN_BACKUP)?;

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
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("pg_dump без stdout")))?;

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
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{name}\""),
        )
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(stdout)))
        .map_err(|e| AppError::Other(e.into()))?)
}
