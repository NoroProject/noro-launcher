//! Списки версий MC и загрузчиков для автодополнения в админке.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;

use schema::PERM_SERVERS_VIEW;
use serde::Deserialize;
use serde_json::{json, Value};

/// Версии Minecraft (release/snapshot) из манифеста Mojang.
pub async fn minecraft(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_VIEW)?;
    let manifest: Value = get_json(
        &state,
        "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await?;
    // Пустой список здесь читался бы в админке как «у Mojang нет версий».
    // Если манифест другой формы — это сбой источника, и говорить надо о нём.
    let versions: Vec<String> = array(&manifest["versions"], "манифест Mojang")?
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    Ok(Json(json!({ "versions": versions })))
}

/// Массив из ответа стороннего сервиса — или ошибка с указанием, чьего.
fn array<'a>(value: &'a Value, source: &str) -> AppResult<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("{source} returned an unexpected shape")))
}

#[derive(Deserialize)]
pub struct LoaderQuery {
    pub mc: String,
}

/// Версии загрузчика для заданной версии MC.
pub async fn loader(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(kind): Path<String>,
    Query(q): Query<LoaderQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_VIEW)?;
    let versions = match kind.as_str() {
        "fabric" => fabric_like(&state, "https://meta.fabricmc.net/v2", &q.mc).await?,
        "quilt" => fabric_like(&state, "https://meta.quiltmc.org/v3", &q.mc).await?,
        "neoforge" => neoforge(&state, &q.mc).await?,
        "forge" => forge(&state, &q.mc).await?,
        "vanilla" => vec![],
        other => return Err(AppError::BadRequest(format!("unknown modloader: {other}"))),
    };
    Ok(Json(json!({ "versions": versions })))
}

async fn fabric_like(state: &AppState, base: &str, mc: &str) -> AppResult<Vec<String>> {
    let url = format!("{base}/versions/loader/{mc}");
    let list: Value = get_json(state, &url).await?;
    Ok(array(&list, base)?
        .iter()
        .filter_map(|e| e["loader"]["version"].as_str().map(String::from))
        .collect())
}

async fn neoforge(state: &AppState, mc: &str) -> AppResult<Vec<String>> {
    // neoforge версии вида 21.1.x для MC 1.21.1.
    let prefix = format!("{}.", mc.strip_prefix("1.").unwrap_or(mc));
    let url = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
    let resp: Value = get_json(state, url).await?;
    Ok(array(&resp["versions"], "maven.neoforged.net")?
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|v| v.starts_with(&prefix))
        .map(String::from)
        .rev()
        .collect())
}

async fn forge(state: &AppState, mc: &str) -> AppResult<Vec<String>> {
    // maven-metadata содержит версии вида "1.21.1-52.0.0".
    let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
    let xml = state
        .http()
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .text()
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    let needle = format!("{mc}-");
    let mut out: Vec<String> = xml
        .split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split("</version>").next())
        .filter(|v| v.starts_with(&needle))
        .map(|v| v.trim_start_matches(&needle).to_string())
        .collect();
    out.reverse(); // новые версии — первыми
    Ok(out)
}

async fn get_json(state: &AppState, url: &str) -> AppResult<Value> {
    state
        .http()
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json()
        .await
        .map_err(|e| AppError::Other(e.into()))
}
