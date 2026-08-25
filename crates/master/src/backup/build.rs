//! Сборка полного архива и отдача его потоком.

use super::tar_out::{append_bytes, append_file, ChanWrite, Parts};
use super::{BackupMeta, DATA, DUMP, META, SIGNATURE, WORK};
use crate::error::{AppError, AppResult};
use crate::signing::Signer25519;
use crate::state::AppState;
use anyhow::{bail, Context, Result};
use axum::body::Body;
use axum::http::header;
use axum::response::Response;
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

/// Уровень gzip. Первый, а не шестой по умолчанию: на реальном томе в 3.5 ГБ
/// (сборки, jar'ы, нативные библиотеки) шестой стоит лишних 11 секунд из 56 и
/// выигрывает 30 МБ из 2.7 ГБ — 1% размера за 21% времени.
const LEVEL: Compression = Compression::new(1);

/// `GET /api/admin/backup/full`. Дамп сперва ложится во временный файл: размер
/// записи tar стоит в заголовке, а у потока `pg_dump` его узнать неоткуда.
/// Файлы `data/` читаются прямо с диска — это основная масса архива, и второй
/// её копии на томе не возникает.
pub async fn run(state: &AppState) -> AppResult<Response> {
    let now = Utc::now();
    let work = super::work_dir(&state.config.data_dir);
    super::swap::ensure(&work).map_err(AppError::Other)?;

    let dump = work.join(format!("dump-{}.sql", uuid::Uuid::new_v4()));
    dump_database(&state.config.database_url, &dump)
        .await
        .map_err(AppError::Other)?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<std::io::Result<bytes::Bytes>>(8);
    let data_dir = state.config.data_dir.clone();
    let signer = state.signer.clone();
    // Без NORO_SIGNING_KEY подпись была бы dev-ключом и ничего не доказывала:
    // архив честно уходит без неё.
    let signed = !state.config.is_dev_signing();

    tokio::task::spawn_blocking(move || {
        let out = ChanWrite(tx.clone());
        if let Err(e) = write_archive(out, &dump, &data_dir, &signer, signed, now, LEVEL) {
            // Тело ответа уже пошло, статус не изменить. Оборванный архив без
            // meta.json проверку не пройдёт — восстановить его не выйдет.
            tracing::error!(error = %e, "сборка архива оборвалась");
            let _ = tx.blocking_send(Err(std::io::Error::other(e.to_string())));
        }
        let _ = std::fs::remove_file(&dump);
    });

    let stream = futures::stream::poll_fn(move |cx| rx.poll_recv(cx));
    Response::builder()
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", super::archive_name(now)),
        )
        .body(Body::from_stream(stream))
        .map_err(|e| AppError::Other(e.into()))
}

/// Дамп текстом: разворачивается обычным `psql`, без `pg_restore`. `--clean`
/// здесь ради того, кто зальёт файл руками — своё восстановление чистит схему
/// само, см. `db_load::RESET`.
async fn dump_database(database_url: &str, to: &Path) -> Result<()> {
    let file = std::fs::File::create(to).with_context(|| format!("создать {}", to.display()))?;
    let out = tokio::process::Command::new("pg_dump")
        .args([
            "--format=plain",
            "--clean",
            "--if-exists",
            "--no-owner",
            "--no-acl",
        ])
        .arg(database_url)
        .stdout(std::process::Stdio::from(file))
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .context("не удалось запустить pg_dump. Он есть в образе мастера?")?;
    if !out.status.success() {
        let _ = std::fs::remove_file(to);
        bail!("pg_dump: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

/// Обобщено по writer'у ради тестов: в бою труба в тело ответа, в тестах файл.
pub(super) fn write_archive<W: std::io::Write>(
    out: W,
    dump: &Path,
    data_dir: &Path,
    signer: &Signer25519,
    signed: bool,
    now: DateTime<Utc>,
    level: Compression,
) -> Result<()> {
    let mut tar = tar::Builder::new(GzEncoder::new(out, level));
    let mut parts = Parts::new();

    let dump_bytes = append_file(&mut tar, &mut parts, dump, DUMP)?;
    let (data_bytes, data_files) = append_data_dir(&mut tar, &mut parts, data_dir)?;

    let meta = BackupMeta {
        master_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: crate::db::known_schema_version(),
        created_at: now,
        dump_bytes,
        data_bytes,
        data_files,
        signed,
        parts,
    };
    let meta_bytes = serde_json::to_vec_pretty(&meta)?;
    append_bytes(&mut tar, META, &meta_bytes)?;

    if signed {
        let sig = signer.sign(&Sha256::digest(&meta_bytes));
        append_bytes(&mut tar, SIGNATURE, hex::encode(sig).as_bytes())?;
    }

    tar.into_inner()?.finish()?;
    Ok(())
}

/// Обойти `NORO_DATA_DIR` мимо служебного каталога: в нём лежат вытесненные
/// прошлым восстановлением файлы, и брать их значит удваивать архив.
fn append_data_dir<W: std::io::Write>(
    tar: &mut tar::Builder<W>,
    parts: &mut Parts,
    data_dir: &Path,
) -> Result<(u64, usize)> {
    let mut bytes = 0;
    let mut files = 0;
    let walk = WalkDir::new(data_dir)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || e.file_name() != std::ffi::OsStr::new(WORK));
    for entry in walk {
        let entry = entry.context("обход каталога данных")?;
        // Симлинки WalkDir не разворачивает и файлами не считает: тянуть цель
        // за пределами тома мы не подписывались.
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(data_dir)?;
        let name = format!("{DATA}/{}", rel.to_string_lossy().replace('\\', "/"));
        bytes += append_file(tar, parts, entry.path(), &name)?;
        files += 1;
    }
    Ok((bytes, files))
}
