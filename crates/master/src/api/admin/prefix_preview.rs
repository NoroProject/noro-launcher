//! Role badge preview — renders the exact PNG the game will show.
//!
//! Its own route rather than a field on the role JSON: drawing the badge in the
//! web UI would mean a second font implementation to keep in step with this one.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::prefix;
use axum::extract::Query;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::IntoResponse;
use schema::PERM_ROLES_VIEW;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Ask {
    /// Badge text. Empty falls back to the role name, same as in game.
    #[serde(default)]
    pub text: String,
    /// `#rrggbb`. Anything unparseable renders grey instead of failing.
    #[serde(default)]
    pub color: String,
    /// Name to draw beside the badge. Empty means badge only.
    #[serde(default)]
    pub name: String,
}

/// Takes the fields as query parameters rather than a role id, so the preview
/// updates while someone types instead of only after a save.
pub async fn badge(_admin: AdminAuth, Query(ask): Query<Ask>) -> AppResult<impl IntoResponse> {
    _admin.require(PERM_ROLES_VIEW)?;
    let text = if ask.text.trim().is_empty() {
        "ROLE"
    } else {
        ask.text.trim()
    };
    let png = if ask.name.trim().is_empty() {
        prefix::badge_png(text, &ask.color)?
    } else {
        prefix::badge_line(text, &ask.color, ask.name.trim())?
    };
    Ok((
        [
            (CONTENT_TYPE, "image/png"),
            // The URL changes with every keystroke but browsers still cache it;
            // without this the preview sticks on the first render.
            (CACHE_CONTROL, "no-store"),
        ],
        png,
    ))
}
