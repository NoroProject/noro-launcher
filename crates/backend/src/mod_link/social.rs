//! The player's side panel: news, direct messages, tickets, pictures.
//!
//! Unlike cases, none of this needs moderator rights — it's the same account
//! endpoints the site uses, on the player's own token. The master's answers
//! are read into local shapes and re-cut into `mod_link` ones, so a field the
//! site adds tomorrow doesn't reach a jar that shipped today.

use super::master::{Answer, Api, Denied};
use super::ModLink;
use crate::backend::Ctx;
use base64::Engine;
use chrono::{DateTime, Utc};
use mod_link::{ChatMessage, ChatThread, NewsPost, Ticket, TicketMessage, ToMod};
use schema::Page;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// The longest side of a news picture once it reaches the game. The panel is a
/// few hundred GUI pixels wide; anything bigger is upload size for nothing.
const IMAGE_MAX_SIDE: u32 = 640;
/// A picture bigger than this isn't a news illustration, and the launcher
/// won't hold it in memory to find out.
const IMAGE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct ThreadRow {
    peer_id: Uuid,
    peer_name: String,
    #[serde(default)]
    peer_is_bot: bool,
    #[serde(default)]
    preview: String,
    #[serde(default)]
    preview_is_mine: bool,
    #[serde(default)]
    unread: i64,
    last_message_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct DmRow {
    id: Uuid,
    author_id: Uuid,
    author_name: String,
    body: String,
    at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Conversation {
    peer_name: String,
    messages: Page<DmRow>,
}

#[derive(Deserialize)]
struct Unread {
    unread: i64,
}

impl Api {
    async fn chats(&self) -> Answer<Page<ThreadRow>> {
        let path = format!("/api/dm?limit={}&offset=0", mod_link::LIST_PAGE);
        self.json(self.get(&path)).await
    }

    async fn conversation(&self, peer: Uuid) -> Answer<Conversation> {
        let path = format!("/api/dm/{peer}?limit={}&offset=0", mod_link::HISTORY_PAGE);
        self.json(self.get(&path)).await
    }

    async fn tickets(&self) -> Answer<Page<Ticket>> {
        let path = format!("/api/tickets?limit={}&offset=0", mod_link::LIST_PAGE);
        self.json(self.get(&path)).await
    }

    async fn ticket_messages(&self, id: Uuid) -> Answer<Page<TicketMessage>> {
        let path = format!(
            "/api/tickets/{id}?limit={}&offset=0",
            mod_link::HISTORY_PAGE
        );
        self.json(self.get(&path)).await
    }
}

pub async fn send_chat(api: &Api, peer: Uuid, body: String) -> Answer<()> {
    api.post(&format!("/api/dm/{peer}"), json!({ "body": body }))
        .await
}

pub async fn create_ticket(api: &Api, subject: String, content: String) -> Answer<Uuid> {
    let ticket: Ticket = api
        .json(
            api.req(reqwest::Method::POST, "/api/tickets")
                .json(&json!({ "subject": subject, "content": content })),
        )
        .await?;
    Ok(ticket.id)
}

pub async fn reply_ticket(api: &Api, id: Uuid, content: String) -> Answer<()> {
    api.post(
        &format!("/api/tickets/{id}/messages"),
        json!({ "content": content }),
    )
    .await
}

pub async fn refresh_chats(ctx: &Ctx, link: &ModLink) -> Answer<()> {
    let Some(api) = Api::new(ctx) else {
        return Ok(());
    };
    let page = api.chats().await?;
    let unread = api
        .json::<Unread>(api.get("/api/dm/unread"))
        .await
        .map(|u| u.unread)
        .unwrap_or_else(|_| page.items.iter().map(|t| t.unread).sum());
    let threads = page
        .items
        .into_iter()
        .map(|t| ChatThread {
            peer_id: t.peer_id,
            peer_name: t.peer_name,
            peer_is_bot: t.peer_is_bot,
            preview: t.preview,
            preview_is_mine: t.preview_is_mine,
            unread: t.unread,
            last_message_at: t.last_message_at,
        })
        .collect();
    link.send(ToMod::Chats {
        threads,
        total: page.total,
        unread,
    });
    Ok(())
}

/// Send the conversation and mark it read: it's on screen, so it has been seen.
pub async fn refresh_chat(ctx: &Ctx, link: &ModLink, peer: Uuid) -> Answer<()> {
    let Some(api) = Api::new(ctx) else {
        return Ok(());
    };
    let me = ctx.profile().map(|u| u.id);
    let conversation = api.conversation(peer).await?;
    // The master pages newest first; the panel reads top to bottom.
    let messages = conversation
        .messages
        .items
        .into_iter()
        .rev()
        .map(|m| ChatMessage {
            id: m.id,
            mine: Some(m.author_id) == me,
            author_name: m.author_name,
            body: m.body,
            at: m.at,
        })
        .collect();
    link.send(ToMod::Chat {
        peer_id: peer,
        peer_name: conversation.peer_name,
        messages,
        total: conversation.messages.total,
    });
    api.post(&format!("/api/dm/{peer}/read"), json!({})).await
}

pub async fn refresh_tickets(ctx: &Ctx, link: &ModLink) -> Answer<()> {
    let Some(api) = Api::new(ctx) else {
        return Ok(());
    };
    let page = api.tickets().await?;
    link.send(ToMod::Tickets {
        tickets: page.items,
        total: page.total,
    });
    Ok(())
}

/// Reading a ticket marks staff replies read on the master, so the list is
/// re-sent after it for the badge to go out.
pub async fn refresh_ticket(ctx: &Ctx, link: &ModLink, id: Uuid) -> Answer<()> {
    let Some(api) = Api::new(ctx) else {
        return Ok(());
    };
    let page = api.ticket_messages(id).await?;
    let mut messages = page.items;
    messages.reverse();
    link.send(ToMod::TicketMessages {
        ticket_id: id,
        messages,
        total: page.total,
    });
    refresh_tickets(ctx, link).await
}

/// The master's news as the panel draws it. Relative links — the preview and
/// pictures inside the body — are made absolute here, since the mod doesn't
/// know where the master lives.
pub fn news_posts(ctx: &Ctx, items: &[schema::NewsItem]) -> Vec<NewsPost> {
    let base = master_base(ctx);
    items
        .iter()
        .map(|n| NewsPost {
            id: n.id,
            title: n.title.clone(),
            body: absolute_links(&n.body, &base),
            image_url: n.preview_img_url.as_deref().map(|u| absolute(u, &base)),
            author_name: n.author_name.clone(),
            pinned: n.pinned,
            published_at: n.published_at,
        })
        .collect()
}

/// Fetch a picture and answer with it, or with `None` so the mod stops waiting.
pub async fn send_image(ctx: &Ctx, link: &ModLink, url: String) {
    let png = match fetch_png(ctx, &url).await {
        Ok(png) => Some(base64::engine::general_purpose::STANDARD.encode(png)),
        Err(e) => {
            tracing::debug!(%url, "mod_link: picture not fetched: {e:?}");
            None
        }
    };
    link.send(ToMod::Image {
        url,
        png_base64: png,
    });
}

async fn fetch_png(ctx: &Ctx, url: &str) -> Answer<Vec<u8>> {
    let url = absolute(url, &master_base(ctx));
    // Only the web: anything else would let the mod read local files through
    // the launcher.
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(Denied::Invalid(0));
    }
    let res = ctx
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| Denied::Offline(e.to_string()))?;
    if !res.status().is_success() {
        return Err(Denied::NotFound(0));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| Denied::Offline(e.to_string()))?;
    if bytes.len() > IMAGE_MAX_BYTES {
        return Err(Denied::Invalid(0));
    }
    // Decoding is CPU work the size of the picture; it stays off the runtime.
    tokio::task::spawn_blocking(move || reencode(&bytes))
        .await
        .map_err(|e| Denied::Offline(e.to_string()))?
}

/// Any format the master serves becomes a PNG the game can read, no larger
/// than the panel needs.
fn reencode(bytes: &[u8]) -> Answer<Vec<u8>> {
    let picture = image::load_from_memory(bytes).map_err(|e| Denied::Offline(e.to_string()))?;
    let picture = if picture.width().max(picture.height()) > IMAGE_MAX_SIDE {
        picture.thumbnail(IMAGE_MAX_SIDE, IMAGE_MAX_SIDE)
    } else {
        picture
    };
    let mut out = std::io::Cursor::new(Vec::new());
    picture
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| Denied::Offline(e.to_string()))?;
    Ok(out.into_inner())
}

fn master_base(ctx: &Ctx) -> String {
    ctx.config
        .get()
        .master_url
        .trim_end_matches('/')
        .to_string()
}

fn absolute(url: &str, base: &str) -> String {
    if url.starts_with('/') && !url.starts_with("//") {
        format!("{base}{url}")
    } else {
        url.to_string()
    }
}

/// `](/files/x.png)` → `](https://master/files/x.png)` — the only relative form
/// the news editor produces.
fn absolute_links(body: &str, base: &str) -> String {
    body.replace("](/", &format!("]({base}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_picture_links_get_the_master_address() {
        let body = "text ![a](/api/files/1.png) and ![b](https://cdn/x.png)";
        assert_eq!(
            absolute_links(body, "https://m"),
            "text ![a](https://m/api/files/1.png) and ![b](https://cdn/x.png)"
        );
    }

    #[test]
    fn protocol_relative_links_are_left_alone() {
        assert_eq!(absolute("//cdn/x.png", "https://m"), "//cdn/x.png");
        assert_eq!(absolute("/x.png", "https://m"), "https://m/x.png");
    }
}
