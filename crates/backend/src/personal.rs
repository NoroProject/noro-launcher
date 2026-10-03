// Over 150 lines: the content browser's whole conversation with the master —
// list, versions, install, remove — and each half needs the other's shapes.
//! Personal content: what the player installs on top of a build.
//!
//! Nothing is downloaded here. The master fetches the jar into its own store
//! and puts it in the signed manifest, so by the time this module is involved
//! the file is already something the normal sync knows how to fetch and verify.
//! That is the whole reason the set lives on the master: `clean_extra` deletes
//! anything under `mods/` that the manifest does not list, and a launcher-side
//! list of exceptions would be a second source of truth about what belongs in
//! the folder.
//!
//! What this module does is talk to the master and hand the answers to the UI.

use crate::backend::Ctx;
use crate::master_api::MasterApi;
use bridge::{ContentVersionInfo, MessageToFrontend};
use schema::personal::{ContentKind, InstallRequest};
use serde_json::Value;
use uuid::Uuid;

fn api(ctx: &Ctx) -> Option<MasterApi> {
    MasterApi::for_session(ctx)
}

fn failed(ctx: &Ctx, e: anyhow::Error) {
    tracing::debug!(error = %e, "personal content request failed");
    ctx.send(MessageToFrontend::ContentActionFailed {
        message: e.to_string(),
    });
}

pub fn request(ctx: &Ctx, server_id: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.personal_content(server_id).await {
            Ok(items) => ctx.send(MessageToFrontend::PersonalContent { server_id, items }),
            Err(e) => failed(&ctx, e),
        }
    });
}

/// Versions of one project, marked against the build they would go on.
///
/// The master is asked for all of them rather than only the matching ones: a
/// list that silently hides everything is indistinguishable from a broken
/// request, and "no version for this build yet" is a useful answer.
pub fn request_versions(
    ctx: &Ctx,
    provider: String,
    project_id: String,
    mc_version: String,
    loader: String,
) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let raw = match api.catalog_versions(&provider, &project_id, "").await {
            Ok(v) => v,
            Err(e) => return failed(&ctx, e),
        };
        let versions = raw
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|v| version_of(v, &mc_version, &loader))
                    .collect()
            })
            .unwrap_or_default();
        ctx.send(MessageToFrontend::ContentVersions {
            provider,
            project_id,
            versions,
        });
    });
}

fn version_of(v: &Value, mc_version: &str, loader: &str) -> Option<ContentVersionInfo> {
    let list = |key: &str| -> Vec<String> {
        v.get(key)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    let game_versions = list("game_versions");
    let loaders = list("loaders");

    // An empty list from a provider means "unstated", not "nothing". Refusing
    // on unstated would hide every resource pack, which no provider tags with
    // a loader.
    let fits_mc = game_versions.is_empty() || game_versions.iter().any(|g| g == mc_version);
    let fits_loader = loaders.is_empty()
        || loaders
            .iter()
            .any(|l| l.eq_ignore_ascii_case(loader) || l == "minecraft");

    Some(ContentVersionInfo {
        id: v.get("id")?.as_str()?.to_string(),
        name: str_at(v, "name"),
        version_number: str_at(v, "version_number"),
        channel: str_at(v, "channel"),
        downloads: v.get("downloads").and_then(Value::as_u64).unwrap_or(0),
        filename: str_at(v, "filename"),
        size: v.get("size").and_then(Value::as_u64).unwrap_or(0),
        downloadable: v
            .get("downloadable")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        compatible: fits_mc && fits_loader,
        game_versions,
        loaders,
    })
}

fn str_at(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

#[allow(clippy::too_many_arguments)]
pub fn install(
    ctx: &Ctx,
    server_id: Uuid,
    kind: ContentKind,
    provider: String,
    project_id: String,
    version_id: String,
    title: String,
    icon_url: Option<String>,
) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let req = InstallRequest {
            server_id,
            kind,
            provider,
            project_id,
            version_id,
            title,
            icon_url,
        };
        match api.install_content(&req).await {
            // The whole list comes back rather than the one row: installing can
            // shadow something else, and a list that only grows would keep
            // showing a mod the build has since taken over.
            Ok(_) => request(&ctx, server_id),
            Err(e) => failed(&ctx, e),
        }
    });
}

pub fn remove(ctx: &Ctx, server_id: Uuid, id: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.remove_content(id).await {
            Ok(()) => request(&ctx, server_id),
            Err(e) => failed(&ctx, e),
        }
    });
}

pub fn set_enabled(ctx: &Ctx, server_id: Uuid, id: Uuid, enabled: bool) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.set_content_enabled(id, enabled).await {
            Ok(()) => request(&ctx, server_id),
            Err(e) => failed(&ctx, e),
        }
    });
}

// ── Java runtimes ───────────────────────────────────────────────────────────

pub fn request_java(ctx: &Ctx, server_id: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.java_runtimes(server_id).await {
            Ok(r) => ctx.send(MessageToFrontend::JavaRuntimes {
                server_id,
                options: r.options,
                default_component: r.default_component,
                selected: r.selected,
            }),
            Err(e) => failed(&ctx, e),
        }
    });
}

/// Picking a runtime makes the master download it, which takes as long as a few
/// hundred megabytes take. The answer only arrives when the files are there, so
/// the choice the UI then shows is one that will actually launch.
pub fn set_java(ctx: &Ctx, server_id: Uuid, component: Option<String>) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.set_java_runtime(server_id, component.as_deref()).await {
            Ok(()) => request_java(&ctx, server_id),
            Err(e) => failed(&ctx, e),
        }
    });
}
