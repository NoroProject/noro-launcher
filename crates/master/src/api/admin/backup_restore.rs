//! Разбор и применение архива.
//!
//! Право отдельное от выгрузки — `noro.admin.backup.restore`: скачивание
//! читает, восстановление затирает базу и файлы целиком.
//!
//! Шаг разбора и шаг применения разнесены намеренно. Архив весит как весь том,
//! и заливать его дважды — сперва «посмотреть», потом «применить» — значит
//! ждать вдвое дольше на ровном месте. Поэтому `inspect` кладёт загрузку в
//! служебный каталог и возвращает её id, а `restore` берёт по этому id уже
//! лежащий файл.
//!
//! Вердикт `inspect` при этом ничего не разрешает: `restore` проверяет архив
//! заново. Между двумя запросами файл мог подменить любой, у кого есть доступ
//! к тому.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::Json;
use futures::StreamExt;
use schema::PERM_BACKUP_RESTORE;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Сколько загруженный архив ждёт подтверждения, прежде чем его уберут.
const UPLOAD_TTL_HOURS: i64 = 6;

/// `POST /api/admin/backup/inspect` — тело запроса это сам `tar.gz`.
///
/// Ничего не трогает: разбирает паспорт, сверяет sha256 частей и подпись.
pub async fn inspect(
    State(state): State<AppState>,
    admin: AdminAuth,
    body: Body,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BACKUP_RESTORE)?;
    let work = work_dir(&state)?;
    // Отменённый разбор оставил бы архив лежать до конца места на диске.
    let _ = crate::backup::swap::sweep(&work, UPLOAD_TTL_HOURS);

    let id = Uuid::new_v4();
    let path = upload_path(&work, id);
    if let Err(e) = save(&path, body).await {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(AppError::Other(e));
    }

    let verdict = crate::backup::restore::check(&state, path.clone()).await;
    match verdict {
        Ok(verdict) => {
            // Причина отказа считается здесь же, чтобы админка не повторяла
            // правила проверки у себя и не разошлась с ними.
            let refusal = verdict.refusal();
            Ok(Json(
                json!({ "upload": id, "verdict": verdict, "refusal": refusal }),
            ))
        }
        Err(e) => {
            // Разобрать не вышло — держать файл незачем.
            let _ = tokio::fs::remove_file(&path).await;
            Err(e)
        }
    }
}

#[derive(Deserialize)]
pub struct RestoreReq {
    /// id загрузки, выданный `inspect`.
    pub upload: Uuid,
}

/// `POST /api/admin/backup/restore` — применить ранее загруженный архив.
///
/// В случае успеха мастер отвечает и через пару секунд выходит: база залита
/// под ним, и работать дальше на прежнем пуле и кэшах он не может.
pub async fn restore(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<RestoreReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BACKUP_RESTORE)?;
    let path = upload_path(&work_dir(&state)?, req.upload);
    if !path.is_file() {
        return Err(AppError::NotFound(
            "загрузка не найдена — возможно, её убрали по сроку. Загрузите архив заново".into(),
        ));
    }

    let out = crate::backup::restore::apply(&state, path.clone(), &admin.actor).await;
    // Архив весит как весь том: держать его после применения незачем, а после
    // отказа — тем более.
    let _ = tokio::fs::remove_file(&path).await;
    Ok(Json(out?))
}

fn work_dir(state: &AppState) -> AppResult<PathBuf> {
    let work = crate::backup::work_dir(&state.config.data_dir);
    crate::backup::swap::ensure(&work).map_err(AppError::Other)?;
    Ok(work)
}

/// id — это `Uuid`, так что в имя не попадёт ни `..`, ни слеш.
fn upload_path(work: &std::path::Path, id: Uuid) -> PathBuf {
    work.join(format!("upload-{id}.tar.gz"))
}

/// Сложить тело запроса на диск потоком: архив не помещается в память.
async fn save(path: &std::path::Path, body: Body) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }
    file.flush().await?;
    Ok(())
}
