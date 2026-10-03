//! The session with the master: refreshing tokens, checking the stored one
//! at startup, and asking whether a newer launcher is out.

use super::*;

const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// The update check is a nicety; it must not hold anything up.
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// Why a refresh didn't produce new tokens.
pub enum RefreshError {
    /// No refresh token, or the master refused it: the session is over.
    Rejected,
    /// The master couldn't be reached; try again later.
    Unreachable,
}

pub(super) fn master_base(ctx: &Ctx) -> String {
    ctx.config
        .get()
        .master_url
        .trim_end_matches('/')
        .to_string()
}

/// Trade the stored refresh token for a new pair.
pub async fn refresh_tokens(ctx: &Ctx) -> Result<token_store::StoredAuth, RefreshError> {
    let Some(stored) = token_store::load().filter(|s| !s.refresh_token.is_empty()) else {
        tracing::info!("no refresh token in the keyring");
        return Err(RefreshError::Rejected);
    };
    let resp = ctx
        .http
        .post(format!("{}/auth/refresh", master_base(ctx)))
        .json(&serde_json::json!({ "refresh_token": stored.refresh_token }))
        .timeout(STARTUP_REQUEST_TIMEOUT)
        .send()
        .await;
    let resp = match resp {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("token refresh: master unreachable: {e}");
            return Err(RefreshError::Unreachable);
        }
    };
    if resp.status().is_server_error() {
        tracing::warn!("token refresh: master returned {}", resp.status());
        return Err(RefreshError::Unreachable);
    }
    if !resp.status().is_success() {
        tracing::info!("token refresh refused: {}", resp.status());
        return Err(RefreshError::Rejected);
    }
    let v: serde_json::Value = resp.json().await.map_err(|_| RefreshError::Unreachable)?;
    match (v["access_token"].as_str(), v["refresh_token"].as_str()) {
        (Some(at), Some(rt)) => {
            tracing::info!("token refreshed");
            Ok(token_store::StoredAuth {
                access_token: at.to_string(),
                refresh_token: rt.to_string(),
            })
        }
        _ => {
            tracing::warn!("token refresh: response had no tokens in it");
            Err(RefreshError::Unreachable)
        }
    }
}

pub(super) enum MeError {
    Rejected,
    Unreachable,
}

pub(super) async fn fetch_me(ctx: &Ctx, token: &str) -> Result<UserProfile, MeError> {
    let api = crate::master_api::MasterApi::new(
        ctx.http.clone(),
        &ctx.config.get().master_url,
        Some(token.to_string()),
    )
    .ok_or(MeError::Unreachable)?;
    let resp = api
        .request(reqwest::Method::GET, "/api/me")
        .timeout(STARTUP_REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("restore_session: master unreachable: {e}");
            MeError::Unreachable
        })?;
    match resp.status().as_u16() {
        401 | 403 => Err(MeError::Rejected),
        s if !(200..300).contains(&s) => {
            tracing::warn!("restore_session: master returned {s}");
            Err(MeError::Unreachable)
        }
        _ => resp.json::<UserProfile>().await.map_err(|e| {
            tracing::warn!("restore_session: profile did not parse: {e}");
            MeError::Unreachable
        }),
    }
}

/// Check the stored session at startup, refreshing it once if it expired.
/// Reports back through `InternalEvent`s; never logs the player out over a
/// network problem.
pub(super) async fn restore_session(ctx: Ctx, token: String) {
    let event = match fetch_me(&ctx, &token).await {
        Ok(user) => {
            // No name here: this log goes out with "report a problem".
            tracing::info!("restore_session: session restored");
            InternalEvent::SessionRestored { user }
        }
        Err(MeError::Unreachable) => InternalEvent::SessionUnverified,
        Err(MeError::Rejected) => match refresh_tokens(&ctx).await {
            Ok(auth) => {
                let access = auth.access_token.clone();
                let _ = ctx.internal.send(InternalEvent::TokensRefreshed { auth });
                match fetch_me(&ctx, &access).await {
                    Ok(user) => InternalEvent::SessionRestored { user },
                    Err(MeError::Rejected) => InternalEvent::SessionRejected,
                    Err(MeError::Unreachable) => InternalEvent::SessionUnverified,
                }
            }
            Err(RefreshError::Rejected) => InternalEvent::SessionRejected,
            Err(RefreshError::Unreachable) => InternalEvent::SessionUnverified,
        },
    };
    let _ = ctx.internal.send(event);
}

/// Background check for a newer launcher. Silent on any failure: being offline
/// is no reason to bother the player, and the bootstrapper checks too.
pub(super) async fn check_launcher_update(ctx: Ctx) {
    let url = format!(
        "{}/api/launcher/version?platform={}",
        master_base(&ctx),
        schema::current_platform()
    );
    let Ok(Ok(r)) = tokio::time::timeout(UPDATE_CHECK_TIMEOUT, ctx.http.get(&url).send()).await
    else {
        return;
    };
    let Ok(v) = r.json::<serde_json::Value>().await else {
        return;
    };
    let Some(version) = v["version"].as_str() else {
        return;
    };
    // The master reports a git tag, "v1.2.0" (or the legacy "launcher-v1.2.0"),
    // while the crate exposes "1.2.0". Without stripping the prefix they never
    // match and the update banner is always up.
    let reported = version
        .trim_start_matches("launcher-")
        .trim_start_matches('v');
    if reported == env!("CARGO_PKG_VERSION") {
        return;
    }
    if let Ok(lv) = serde_json::from_value::<schema::LauncherVersion>(build_launcher_version(&v)) {
        ctx.send(MessageToFrontend::LauncherUpdateAvailable { version: lv });
    }
}

/// `/api/launcher/version` answers with a subset of `LauncherVersion`; the rest
/// is filled in here so it can be deserialized as one.
pub(super) fn build_launcher_version(v: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "id": Uuid::nil(),
        "version": v["version"],
        "platform": v["platform"],
        "url": v["url"],
        "sha256": v["sha256"],
        "signature": v["signature"],
        "is_current": true,
    })
}
