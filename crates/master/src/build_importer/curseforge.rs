//! Импорт CurseForge .zip. Моды резолвятся через CurseForge API (нужен ключ).

use crate::state::AppState;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::io::Read;
use uuid::Uuid;

const CF_API: &str = "https://api.curseforge.com";

pub async fn import(
    state: &AppState,
    build_id: Uuid,
    job_id: Uuid,
    bytes: Vec<u8>,
) -> Result<usize> {
    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor)?;

    let mut manifest: Value = Value::Null;
    let mut manifest_prefix = String::new();

    // Pass 1: Find manifest.json
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if name.ends_with("manifest.json") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            if let Ok(parsed) = serde_json::from_slice::<Value>(&buf) {
                manifest = parsed;
                // If the manifest is in a subfolder (e.g. Modpack-1.0/manifest.json), remember the prefix
                if let Some(idx) = name.rfind("manifest.json") {
                    manifest_prefix = name[..idx].to_string();
                }
                break;
            }
        }
    }

    if manifest.is_null() {
        return Err(anyhow!("manifest.json не найден"));
    }

    let overrides_dir = manifest["overrides"].as_str().unwrap_or("overrides");
    let overrides_prefix = format!("{manifest_prefix}{overrides_dir}/");

    let mut overrides: Vec<(String, Vec<u8>)> = Vec::new();

    // Pass 2: Extract overrides
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();

        if name.starts_with(&overrides_prefix) {
            let rel = &name[overrides_prefix.len()..];
            if !rel.is_empty() {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                overrides.push((rel.to_string(), buf));
            }
        }
    }

    let mut count = 0usize;

    if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
        if let Some(mc) = manifest["minecraft"]["version"].as_str() {
            prog.recommended_mc_version = Some(mc.to_string());
        }
        if let Some(loaders) = manifest["minecraft"]["modLoaders"].as_array() {
            if let Some(id) = loaders
                .iter()
                .find(|l| l["primary"].as_bool() == Some(true))
                .and_then(|l| l["id"].as_str())
            {
                if let Some((_, ver)) = id.split_once('-') {
                    prog.recommended_modloader_version = Some(ver.to_string());
                }
            }
        }
    }

    let files = manifest["files"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or_default();
    let total_tasks = files.len() + overrides.len();

    if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
        prog.total = total_tasks;
    }

    let mut current = 0;
    let api_key = state.config.curseforge_api_key.clone();

    if !files.is_empty() {
        if let Some(key) = &api_key {
            for f in files {
                let (Some(project_id), Some(file_id)) =
                    (f["projectID"].as_u64(), f["fileID"].as_u64())
                else {
                    continue;
                };

                if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
                    prog.current = current;
                    prog.current_file = format!("CF File: {}/{}", project_id, file_id);
                }

                match resolve_cf_file(state, key, project_id, file_id).await {
                    Ok((url, filename, sha1)) => {
                        let stored = state
                            .files
                            .put_url(state.http(), &url, sha1.as_deref())
                            .await?;
                        let path = format!("mods/{filename}");
                        crate::db::upsert_build_file(
                            &state.db,
                            build_id,
                            &path,
                            &stored.sha1,
                            stored.size as i64,
                            "both",
                            "mod",
                        )
                        .await?;
                        count += 1;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("no downloadUrl") {
                            if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
                                prog.warnings.push(format!(
                                    "Mod {}/{} forbids third-party downloads — install it by hand.",
                                    project_id, file_id
                                ));
                            }
                        } else {
                            tracing::warn!("CF файл {project_id}/{file_id}: {e}");
                            if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
                                prog.warnings
                                    .push(format!("Mod {}/{} failed: {}", project_id, file_id, e));
                            }
                        }
                    }
                }
                current += 1;
            }
        } else {
            return Err(anyhow!(
                "для импорта модов CurseForge нужен CURSEFORGE_API_KEY (overrides импортируются)"
            ));
        }
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

async fn resolve_cf_file(
    state: &AppState,
    api_key: &str,
    project_id: u64,
    file_id: u64,
) -> Result<(String, String, Option<String>)> {
    let url = format!("{CF_API}/v1/mods/{project_id}/files/{file_id}");
    let resp: Value = state
        .http()
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .json()
        .await?;
    let data = &resp["data"];
    let download_url = data["downloadUrl"]
        .as_str()
        .ok_or_else(|| anyhow!("no downloadUrl (the mod forbids third-party downloads)"))?
        .to_string();
    // Имя файла попадает в mods/ сборки. «mod.jar» для каждого безымянного
    // ответа означал бы, что второй такой мод затирает первый.
    let filename = data["fileName"]
        .as_str()
        .ok_or_else(|| anyhow!("CurseForge не вернул fileName"))?
        .to_string();
    let sha1 = data["hashes"]
        .as_array()
        .and_then(|a| a.iter().find(|h| h["algo"].as_u64() == Some(1)))
        .and_then(|h| h["value"].as_str())
        .map(String::from);
    Ok((download_url, filename, sha1))
}
