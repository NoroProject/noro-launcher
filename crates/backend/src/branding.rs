//! The instance's name and logo, from the master's public settings.
//!
//! What the player sees as the network's name — window title, sidebar, login
//! screen, system notifications, Discord status — is whatever the operator set
//! in the admin panel, not the product's name. It is cached on disk for the
//! same reason the translation catalog is: the window should carry the right
//! name from the first frame, not after the master answers.

use crate::backend::Ctx;
use bridge::MessageToFrontend;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Branding {
    pub name: String,
    /// Absolute URL; empty means the logo the launcher ships with.
    #[serde(default)]
    pub logo_url: String,
}

/// Process-wide, because the places that print the name outside the window —
/// the notification toast and the Discord RPC thread — hold no `Ctx`.
static NAME: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(bridge::brand::initial_instance_name()));

/// The instance's name as last heard.
pub fn name() -> String {
    NAME.read().clone()
}

/// Sends the cached branding straight away, then the master's if it differs.
pub fn refresh(ctx: &Ctx) {
    let cached = read_cache(&cache_path(ctx));
    if let Some(branding) = &cached {
        apply(ctx, branding.clone());
    }

    let ctx = ctx.clone();
    tokio::spawn(async move {
        match fetch(&ctx).await {
            Ok(fresh) if cached.as_ref() != Some(&fresh) => {
                write_cache(&cache_path(&ctx), &fresh);
                apply(&ctx, fresh);
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "could not fetch the instance branding");
            }
        }
    });
}

fn apply(ctx: &Ctx, branding: Branding) {
    *NAME.write() = branding.name.clone();
    ctx.send(MessageToFrontend::Branding {
        name: branding.name,
        logo_url: Some(branding.logo_url).filter(|u| !u.is_empty()),
    });
}

async fn fetch(ctx: &Ctx) -> anyhow::Result<Branding> {
    let base = ctx
        .config
        .get()
        .master_url
        .trim_end_matches('/')
        .to_string();
    let body: serde_json::Value = ctx
        .http
        .get(format!("{base}/api/public/settings"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(parse(&base, &body))
}

/// The master leaves out what isn't set; a logo stored as a path is served
/// from the master itself.
fn parse(base: &str, body: &serde_json::Value) -> Branding {
    let name = body["instance_name"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(bridge::brand::DEFAULT_INSTANCE_NAME)
        .to_string();
    let logo = body["logo_url"].as_str().map(str::trim).unwrap_or_default();
    let logo_url = if logo.starts_with('/') {
        format!("{base}{logo}")
    } else {
        logo.to_string()
    };
    Branding { name, logo_url }
}

fn cache_path(ctx: &Ctx) -> PathBuf {
    ctx.dirs.root().join("branding.json")
}

fn read_cache(path: &Path) -> Option<Branding> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

fn write_cache(path: &Path, branding: &Branding) {
    let Ok(bytes) = serde_json::to_vec(branding) else {
        return;
    };
    if let Err(e) = crate::fsutil::write_atomic_sync(path, &bytes) {
        tracing::warn!(error = %format!("{e:#}"), "could not write the branding cache");
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use serde_json::json;

    #[test]
    fn relative_logo_is_served_by_the_master() {
        let b = parse(
            "https://api.example.com",
            &json!({ "instance_name": "Craft Land", "logo_url": "/files/abc" }),
        );
        assert_eq!(b.name, "Craft Land");
        assert_eq!(b.logo_url, "https://api.example.com/files/abc");
    }

    #[test]
    fn unnamed_instance_falls_back_to_the_product_name() {
        let b = parse("https://api.example.com", &json!({ "instance_name": "  " }));
        assert_eq!(b.name, bridge::brand::DEFAULT_INSTANCE_NAME);
        assert_eq!(b.logo_url, "");
    }
}
