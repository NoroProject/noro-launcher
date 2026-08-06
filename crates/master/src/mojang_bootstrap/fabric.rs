//! Bootstrap Fabric/Quilt. У этих модлоадеров нет installer — только JSON-профиль
//! с mainClass и списком библиотек (maven-координаты + базовый URL).

use super::maven::maven_to_path;
use super::BootstrapCtx;
use anyhow::{anyhow, Result};
use schema::{ArtifactKind, Modloader};
use serde_json::Value;

pub async fn bootstrap_fabric(ctx: &mut BootstrapCtx<'_>, loader: Modloader) -> Result<()> {
    let http = ctx.state.http();
    let meta_base = match loader {
        Modloader::Quilt => "https://meta.quiltmc.org/v3",
        _ => "https://meta.fabricmc.net/v2",
    };

    // Определить версию загрузчика (если не задана — взять последнюю стабильную).
    let loader_version = match &ctx.modloader_version {
        Some(v) => v.clone(),
        None => {
            let list: Value = http
                .get(format!("{meta_base}/versions/loader/{}", ctx.mc_version))
                .send()
                .await?
                .json()
                .await?;
            list.as_array()
                .and_then(|a| a.first())
                .and_then(|e| e["loader"]["version"].as_str())
                .ok_or_else(|| anyhow!("не удалось определить версию {} loader", loader.as_str()))?
                .to_string()
        }
    };
    ctx.modloader_version = Some(loader_version.clone());
    ctx.logf(format!("{} loader {loader_version}", loader.as_str()));

    let profile_url = format!(
        "{meta_base}/versions/loader/{}/{}/profile/json",
        ctx.mc_version, loader_version
    );
    let profile: Value = http.get(&profile_url).send().await?.json().await?;

    // mainClass: строка или объект {client, server}.
    match &profile["mainClass"] {
        Value::String(s) => ctx.main_class = s.clone(),
        Value::Object(o) => {
            if let Some(c) = o.get("client").and_then(|v| v.as_str()) {
                ctx.main_class = c.to_string();
            }
        }
        _ => {}
    }

    // Библиотеки.
    if let Some(libs) = profile["libraries"].as_array() {
        for lib in libs {
            let Some(name) = lib["name"].as_str() else {
                continue;
            };
            let Some(path) = maven_to_path(name) else {
                continue;
            };
            // base URL: поле "url" + maven path, либо готовый downloads.artifact.url.
            let url = if let Some(u) = lib["downloads"]["artifact"]["url"].as_str() {
                u.to_string()
            } else if let Some(base) = lib["url"].as_str() {
                format!("{}/{}", base.trim_end_matches('/'), path)
            } else {
                continue;
            };
            let sha1 = lib["downloads"]["artifact"]["sha1"].as_str();
            ctx.register(
                &format!("libraries/{path}"),
                ArtifactKind::Library,
                "both",
                &url,
                sha1,
                "library",
            )
            .await?;
        }
    }

    // Аргументы (если профиль их задаёт) — добавляем к ванильным.
    if let Some(jvm) = profile["arguments"]["jvm"].as_array() {
        for a in jvm {
            if let Some(s) = a.as_str() {
                ctx.jvm_args.push(schema::ManifestArg::new_string(s));
            }
        }
    }
    if let Some(game) = profile["arguments"]["game"].as_array() {
        for a in game {
            if let Some(s) = a.as_str() {
                ctx.game_args.push(schema::ManifestArg::new_string(s));
            }
        }
    }

    Ok(())
}
