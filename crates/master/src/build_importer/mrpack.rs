//! Импорт Modrinth .mrpack.

use crate::state::AppState;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::io::Read;
use uuid::Uuid;

pub async fn import(
    state: &AppState,
    build_id: Uuid,
    job_id: Uuid,
    bytes: Vec<u8>,
) -> Result<usize> {
    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor)?;

    let mut index: Value = Value::Null;
    let mut manifest_prefix = String::new();

    // Pass 1: Find modrinth.index.json
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if name.ends_with("modrinth.index.json") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            if let Ok(parsed) = serde_json::from_slice::<Value>(&buf) {
                index = parsed;
                if let Some(idx) = name.rfind("modrinth.index.json") {
                    manifest_prefix = name[..idx].to_string();
                }
                break;
            }
        }
    }

    if index.is_null() {
        return Err(anyhow!("modrinth.index.json не найден"));
    }

    let overrides_prefix = format!("{manifest_prefix}overrides/");
    let client_overrides_prefix = format!("{manifest_prefix}client-overrides/");

    let mut overrides: Vec<(String, Vec<u8>)> = Vec::new();

    // Pass 2: Extract overrides
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();

        let rel = if name.starts_with(&overrides_prefix) {
            &name[overrides_prefix.len()..]
        } else if name.starts_with(&client_overrides_prefix) {
            &name[client_overrides_prefix.len()..]
        } else {
            continue;
        };

        if !rel.is_empty() {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            overrides.push((rel.to_string(), buf));
        }
    }

    if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
        if let Some(deps) = index["dependencies"].as_object() {
            for (k, v) in deps {
                if let Some(ver) = v.as_str() {
                    match k.as_str() {
                        "fabric-loader" | "quilt-loader" | "forge" | "neoforge" => {
                            prog.recommended_modloader_version = Some(ver.to_string());
                        }
                        "minecraft" => {
                            prog.recommended_mc_version = Some(ver.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let mut count = 0usize;

    let files = index["files"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or_default();
    let total_tasks = files.len() + overrides.len();

    if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
        prog.total = total_tasks;
    }

    let mut current = 0;

    for f in files {
        let Some(path) = f["path"].as_str() else {
            continue;
        };
        if f["env"]["client"].as_str() == Some("unsupported") {
            current += 1;
            continue;
        }
        let url = f["downloads"]
            .as_array()
            .and_then(|d| d.first())
            .and_then(|u| u.as_str());
        let Some(url) = url else { continue };
        let sha1 = f["hashes"]["sha1"].as_str();

        if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
            prog.current = current;
            prog.current_file = path.to_string();
        }

        let stored = state.files.put_url(state.http(), url, sha1).await?;
        crate::db::upsert_build_file(
            &state.db,
            build_id,
            path,
            &stored.sha1,
            stored.size as i64,
            "both",
            super::kind_for(path),
        )
        .await?;
        count += 1;
        current += 1;
    }

    for (rel, data) in overrides {
        if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
            prog.current = current;
            prog.current_file = rel.clone();
        }
        let stored = state.files.put_bytes(&data).await?;
        crate::db::upsert_build_file(
            &state.db,
            build_id,
            &rel,
            &stored.sha1,
            stored.size as i64,
            "both",
            super::kind_for(&rel),
        )
        .await?;
        count += 1;
        current += 1;
    }

    Ok(count)
}
