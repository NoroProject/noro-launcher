// Over 150 lines: four pages of the player's own data, each a fetch and a
// reshape. Split per page they would be four copies of the same eight lines.
//! The player's own pages: punishments, rules, tickets and messages.
//!
//! All four already exist on the master and on the site. The launcher shows
//! them because it is the window that is open while they matter: a ban is read
//! when the game will not start, and a staff reply to a ticket arrives while
//! the player is waiting for it, not while they happen to have the site open.
//!
//! The master's own shapes are wider than any of this needs — staff ids,
//! revocation bookkeeping, sort orders — so each answer is narrowed here into
//! what the screen draws. The narrow types live in `bridge` and are the
//! contract the UI codes against.

use crate::backend::Ctx;
use crate::master_api::MasterApi;
use bridge::{
    DmMessageView, DmThreadView, MessageToFrontend, PunishmentView, RuleView, TicketMessageView,
    TicketView,
};
use serde_json::Value;
use uuid::Uuid;

fn api(ctx: &Ctx) -> Option<MasterApi> {
    MasterApi::for_session(ctx)
}

/// Unix seconds out of an RFC 3339 field. `0` when it is missing: a date of
/// "1970" reads as obviously wrong, which is what an absent timestamp is.
fn at(v: &Value, key: &str) -> i64 {
    v.get(key)
        .and_then(Value::as_str)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp())
        .unwrap_or(0)
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn array(v: &Value) -> Vec<Value> {
    // Both a bare array and a `Page { items }` envelope turn up, depending on
    // which endpoint answered. Accepting either keeps one helper for all four.
    v.get("items")
        .and_then(Value::as_array)
        .or_else(|| v.as_array())
        .cloned()
        .unwrap_or_default()
}

pub fn punishments(ctx: &Ctx) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.punishments().await.inspect_err(log("punishments")) else {
            return;
        };
        let now = chrono::Utc::now().timestamp();
        let items = array(&raw)
            .iter()
            .map(|p| {
                let expires_at = p
                    .get("expires_at")
                    .and_then(Value::as_str)
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.timestamp());
                PunishmentView {
                    kind: text(p, "kind"),
                    reason: text(p, "reason"),
                    actor: text(p, "actor_label"),
                    created_at: at(p, "created_at"),
                    active: p.get("revoked_at").is_none_or(Value::is_null)
                        && expires_at.is_none_or(|e| e > now),
                    expires_at,
                    rule_code: p
                        .get("rule_code")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                }
            })
            .collect();
        ctx.send(MessageToFrontend::PunishmentsLoaded { items });
    });
}

pub fn rules(ctx: &Ctx) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.rules().await.inspect_err(log("rules")) else {
            return;
        };
        // Categories arrive as their own list, keyed by id from each rule.
        let names: std::collections::HashMap<String, String> = raw
            .get("categories")
            .and_then(Value::as_array)
            .map(|list| {
                // A rulebook section calls the field `name`, not `title`: reading
                // `title` left every rule with an empty category.
                list.iter()
                    .map(|c| (text(c, "id"), text(c, "name")))
                    .collect()
            })
            .unwrap_or_default();

        // Punishments come as a separate flat list matched by `rule_id`: a
        // request per rule would turn opening the rulebook into a hundred calls,
        // which is exactly why the master serves them this way.
        let mut sanctions: std::collections::HashMap<String, Vec<bridge::SanctionView>> =
            Default::default();
        for s in raw
            .get("sanctions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            sanctions
                .entry(text(&s, "rule_id"))
                .or_default()
                .push(bridge::SanctionView {
                    kind: text(&s, "kind"),
                    label: text(&s, "label"),
                    min_minutes: s.get("min_minutes").and_then(Value::as_i64),
                    max_minutes: s.get("max_minutes").and_then(Value::as_i64),
                });
        }

        let items = raw
            .get("rules")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .map(|r| RuleView {
                        code: text(r, "code"),
                        title: text(r, "title"),
                        description: text(r, "description"),
                        category: names
                            .get(&text(r, "category_id"))
                            .cloned()
                            .unwrap_or_default(),
                        sanctions: sanctions.remove(&text(r, "id")).unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        ctx.send(MessageToFrontend::RulesLoaded { items });
    });
}

pub fn tickets(ctx: &Ctx) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.tickets(0, 50).await.inspect_err(log("tickets")) else {
            return;
        };
        let items = array(&raw)
            .iter()
            .filter_map(|t| {
                let id: Uuid = t.get("id")?.as_str()?.parse().ok()?;
                Some(TicketView {
                    number: id.simple().to_string()[..8].to_string(),
                    id,
                    subject: text(t, "subject"),
                    status: text(t, "status"),
                    unread: t.get("unread").and_then(Value::as_i64).unwrap_or(0),
                    last_message_at: at(t, "last_message_at"),
                })
            })
            .collect();
        ctx.send(MessageToFrontend::TicketsLoaded { items });
    });
}

pub fn ticket(ctx: &Ctx, id: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.ticket(id).await.inspect_err(log("ticket")) else {
            return;
        };
        // `GET /api/tickets/{id}` returns only a page of messages, not the ticket
        // itself: subject and status come from the list whoever opened it had.
        //
        // The response is newest first, because that is how a page is built. The
        // thread reads top to bottom, so it is reversed here.
        let mut messages: Vec<TicketMessageView> = array(&raw)
            .iter()
            .map(|m| TicketMessageView {
                author: text(m, "author_name"),
                role: m
                    .get("author_role")
                    .and_then(Value::as_str)
                    .filter(|r| !r.is_empty())
                    .map(str::to_string),
                content: text(m, "content"),
                at: at(m, "at"),
                // `player` is the player; everything else is written by staff.
                staff: text(m, "author_side") != "player",
            })
            .collect();
        messages.reverse();

        ctx.send(MessageToFrontend::TicketLoaded {
            id,
            subject: String::new(),
            status: String::new(),
            messages,
        });
    });
}

pub fn open_ticket(ctx: &Ctx, subject: String, content: String) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        if api
            .open_ticket(&subject, &content)
            .await
            .inspect_err(log("open ticket"))
            .is_ok()
        {
            tickets(&ctx);
        }
    });
}

pub fn reply_ticket(ctx: &Ctx, id: Uuid, content: String) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        if api
            .ticket_reply(id, &content)
            .await
            .inspect_err(log("ticket reply"))
            .is_ok()
        {
            ticket(&ctx, id);
        }
    });
}

pub fn dm_threads(ctx: &Ctx) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.dm_threads().await.inspect_err(log("dm threads")) else {
            return;
        };
        let master = ctx.config.get().master_url;
        let items = array(&raw)
            .iter()
            .filter_map(|t| {
                let peer_name = text(t, "peer_name");
                Some(DmThreadView {
                    peer_id: t.get("peer_id")?.as_str()?.parse().ok()?,
                    avatar_url: avatar_of(&master, t, &peer_name),
                    peer_name,
                    preview: text(t, "preview"),
                    unread: t.get("unread").and_then(Value::as_i64).unwrap_or(0),
                    last_message_at: at(t, "last_message_at"),
                })
            })
            .collect();
        ctx.send(MessageToFrontend::DmThreadsLoaded { items });
    });
}

pub fn dm_thread(ctx: &Ctx, peer: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let Ok(raw) = api.dm_thread(peer, 0, 80).await.inspect_err(log("dm")) else {
            return;
        };
        let me = ctx.profile().map(|p| p.id);
        // Messages sit inside the page (`messages.items`), not at the root of the
        // response, and come newest first; the thread reads the other way.
        let mut messages: Vec<DmMessageView> = raw
            .get("messages")
            .map(array)
            .unwrap_or_default()
            .iter()
            .map(|m| DmMessageView {
                author_name: text(m, "author_name"),
                body: text(m, "body"),
                at: at(m, "at"),
                mine: m
                    .get("author_id")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<Uuid>().ok())
                    == me,
            })
            .collect();
        messages.reverse();

        // Opening a conversation is reading it; leaving the badge on would make
        // the count disagree with what is on screen.
        let _ = api.dm_mark_read(peer).await;
        let master = ctx.config.get().master_url;
        let peer_name = text(&raw, "peer_name");
        ctx.send(MessageToFrontend::DmThreadLoaded {
            thread: Box::new(bridge::DmThreadOpen {
                peer,
                avatar_url: avatar_of(&master, &raw, &peer_name),
                peer_name,
                messages,
            }),
        });
    });
}

pub fn send_dm(ctx: &Ctx, peer: Uuid, body: String) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        if api
            .dm_send(peer, &body)
            .await
            .inspect_err(log("send dm"))
            .is_ok()
        {
            dm_thread(&ctx, peer);
        }
    });
}

/// Where to fetch somebody's head.
///
/// A bot carries its own picture; a player has none stored, and the master
/// renders one from their skin. Rendering by username rather than by uuid on
/// purpose: the same endpoint serves the site, and a nick is what both have.
fn avatar_of(master: &str, v: &Value, peer_name: &str) -> Option<String> {
    if let Some(url) = v
        .get("peer_avatar")
        .and_then(Value::as_str)
        .filter(|u| !u.is_empty())
    {
        return Some(url.to_string());
    }
    if peer_name.is_empty() {
        return None;
    }
    Some(format!(
        "{}/api/textures/renders/head?username={}&scale=4",
        master.trim_end_matches('/'),
        urlencoding::encode(peer_name)
    ))
}

/// A failure here is not worth interrupting anybody over: the page stays as it
/// was, and the log says why.
fn log(what: &'static str) -> impl Fn(&anyhow::Error) {
    move |e| tracing::debug!(error = %format!("{e:#}"), "could not load {what}")
}
