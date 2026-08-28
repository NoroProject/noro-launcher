//! Frames the launcher sends on its own: the queue, a case card, readiness.
//!
//! The mod subscribes rather than requesting and waiting. This side decides when
//! to fetch a card and push it out, which is why the protocol has no request ids
//! and no response correlation.

use super::master::Api;
use super::ModLink;
use crate::backend::Ctx;
use mod_link::{CaseView, ToMod, PROTOCOL};
use uuid::Uuid;

/// First frame after the handshake. The permissions are there so the mod can
/// avoid drawing buttons that would only be refused — convenience, not a check.
pub fn ready(ctx: &Ctx) -> ToMod {
    let profile = ctx.profile();
    ToMod::Ready {
        protocol: PROTOCOL,
        username: profile
            .as_ref()
            .map(|u| u.username.clone())
            .unwrap_or_default(),
        locale: ctx.config.get().locale,
        // Role permissions included. A moderator usually has no direct grants
        // at all, so the direct list alone would be empty.
        permissions: profile
            .as_ref()
            .map(|u| u.all_permissions().map(str::to_string).collect())
            .unwrap_or_default(),
    }
}

pub async fn refresh_queue(ctx: &Ctx, link: &ModLink, query: Option<String>, offset: i64) {
    let Some(api) = Api::new(ctx) else { return };
    match api.queue(query.as_deref(), offset).await {
        Ok(page) => {
            link.store()
                .set_queue(page.items.clone(), page.total, offset, query.clone());
            link.send(ToMod::Queue {
                cases: page.items,
                total: page.total,
                offset,
                query,
            });
        }
        Err(e) => tracing::debug!("mod_link: could not read the queue: {e:?}"),
    }
}

/// Re-read the page the moderator is actually looking at, not the first one:
/// otherwise anyone else's action on any case would throw them back to the top
/// of the queue.
pub async fn refresh_queue_in_place(ctx: &Ctx, link: &ModLink) {
    let (query, offset) = link.store().queue_spot();
    refresh_queue(ctx, link, query, offset).await;
}

/// Fetch the card and push it. This also marks the case open, after which it
/// refreshes itself on every `CaseUpdated` from the master.
pub async fn refresh_case(ctx: &Ctx, link: &ModLink, case_id: Uuid) {
    let Some(api) = Api::new(ctx) else { return };
    match api.card(case_id).await {
        Ok(view) => {
            link.store().set_open(view.clone());
            if let Some(frame) = inventory(&view) {
                link.send(frame);
            }
            link.send(ToMod::Case {
                view: Box::new(view),
            });
        }
        Err(e) => {
            link.send(ToMod::Rejected {
                intent: "OpenCase".into(),
                reason: e.key().into(),
                number: e.number(),
            });
        }
    }
}

/// The inventory snapshot goes out as its own frame. In the event feed it's raw
/// JSON, and parsing that on the Java side is work for nothing.
fn inventory(view: &CaseView) -> Option<ToMod> {
    let event = view
        .events
        .iter()
        .rev()
        .find(|e| e.kind == "inventory_snapshot")?;
    // Slots arrive from the agent as-is. A bad entry is skipped rather than
    // failing the whole snapshot — one odd item shouldn't hide the rest.
    let items = event
        .payload
        .get("items")?
        .as_array()?
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    Some(ToMod::Inventory {
        case_id: view.brief.id,
        items,
    })
}
