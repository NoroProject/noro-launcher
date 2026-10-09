//! Turning an intent from the mod into a request to the master.
//!
//! The mod knows no URLs; this is the only place an intent becomes an endpoint.
//! A refusal comes back as `Rejected` with a translation key. The master's 403
//! is the only permission check worth trusting — nothing is decided here.

use super::master::{Answer, Api, Denied};
use super::push;
use super::social;
use super::ModLink;
use crate::backend::Ctx;
use base64::Engine;
use mod_link::{ToLauncher, ToMod};
use serde_json::json;
use uuid::Uuid;

pub async fn handle(ctx: &Ctx, link: &ModLink, frame: ToLauncher) {
    let name = intent_name(&frame);
    let Some(api) = Api::new(ctx) else {
        return link.send(ToMod::Rejected {
            intent: name.into(),
            reason: Denied::Offline(String::new()).key().into(),
            number: 0,
        });
    };

    let case_id = match dispatch(ctx, link, &api, frame).await {
        Ok(case_id) => case_id,
        Err(e) => {
            // The mod gets a key, the log gets the reason. "master unreachable"
            // and "master answered 500" look the same to a moderator, but not
            // to whoever debugs it later.
            if let Denied::Offline(why) = &e {
                tracing::debug!(intent = name, "mod_link: master did not answer: {why}");
            }
            return link.send(ToMod::Rejected {
                intent: name.into(),
                reason: e.key().into(),
                number: e.number(),
            });
        }
    };
    // Anything that changes a case changes the queue too: the claim and the
    // status show in both.
    if let Some(case_id) = case_id {
        push::refresh_case(ctx, link, case_id).await;
        push::refresh_queue_in_place(ctx, link).await;
    }
}

/// `Ok(Some(case_id))` means the case changed and needs re-reading.
async fn dispatch(
    ctx: &Ctx,
    link: &ModLink,
    api: &Api,
    frame: ToLauncher,
) -> Result<Option<Uuid>, Denied> {
    match frame {
        // The handshake was already accepted; a repeat means nothing.
        ToLauncher::Hello { .. } => Ok(None),
        ToLauncher::RequestQueue => {
            push::refresh_queue(ctx, link, None, 0).await;
            Ok(None)
        }
        ToLauncher::RequestQueuePage { query, offset } => {
            push::refresh_queue(ctx, link, query, offset).await;
            Ok(None)
        }
        ToLauncher::OpenCase { case_id } => {
            push::refresh_case(ctx, link, case_id).await;
            Ok(None)
        }
        ToLauncher::CloseCase => {
            link.store().close();
            Ok(None)
        }
        ToLauncher::Claim { case_id } => {
            api.post(&format!("/api/admin/cases/{case_id}/claim"), json!({}))
                .await?;
            Ok(Some(case_id))
        }
        ToLauncher::Release { case_id } => {
            api.post(&format!("/api/admin/cases/{case_id}/release"), json!({}))
                .await?;
            Ok(Some(case_id))
        }
        ToLauncher::Resolve {
            case_id,
            verdict,
            resolution,
            rule_code,
        } => {
            let body =
                json!({ "verdict": verdict, "resolution": resolution, "rule_code": rule_code });
            api.put(&format!("/api/admin/cases/{case_id}/resolve"), body)
                .await?;
            Ok(Some(case_id))
        }
        ToLauncher::AddNote { case_id, text } => {
            api.post(
                &format!("/api/admin/cases/{case_id}/notes"),
                json!({ "text": text }),
            )
            .await?;
            Ok(Some(case_id))
        }
        ToLauncher::Punish {
            case_id,
            kind,
            reason,
            rule_code,
            duration_secs,
        } => {
            // Punishments land on a player, not a case, so the target id has to
            // come out of the card.
            let target = api.card(case_id).await?.brief.target_id;
            let body = json!({
                "kind": kind,
                "reason": reason,
                "rule_code": rule_code,
                "minutes": duration_secs.map(|s| (s / 60).max(1)),
                "case_id": case_id,
            });
            api.post(&format!("/api/admin/users/{target}/punishments"), body)
                .await?;
            Ok(Some(case_id))
        }
        ToLauncher::RequestChat { case_id } => {
            api.post(
                &format!("/api/admin/cases/{case_id}/chat-request"),
                json!({}),
            )
            .await?;
            // The agent takes the snapshot; the card catches up on the next
            // `CaseUpdated`.
            Ok(None)
        }
        ToLauncher::RequestInventory { case_id } => {
            api.post(
                &format!("/api/admin/cases/{case_id}/inventory-request"),
                json!({}),
            )
            .await?;
            Ok(None)
        }
        ToLauncher::Attach {
            case_id,
            note,
            png_base64,
        } => {
            let png = base64::engine::general_purpose::STANDARD
                .decode(png_base64.as_bytes())
                .map_err(|e| Denied::Offline(e.to_string()))?;
            api.attach(case_id, png, note).await?;
            Ok(Some(case_id))
        }
        ToLauncher::Quote {
            case_id,
            sender,
            at,
            hash,
        } => {
            let body = json!({ "sender": sender, "at": at, "hash": hash });
            api.post(&format!("/api/admin/cases/{case_id}/quote"), body)
                .await?;
            Ok(Some(case_id))
        }
        ToLauncher::Lookup { username } => {
            let dossier = api.dossier(&username).await?;
            link.send(ToMod::Dossier { dossier });
            Ok(None)
        }
        ToLauncher::RequestRules => {
            let rules = api.rules(link.server_id()).await?;
            link.send(ToMod::Rules {
                categories: rules.categories,
                rules: rules.rules,
                sanctions: rules.sanctions,
            });
            Ok(None)
        }
        ToLauncher::RequestOwnPunishments => {
            let punishments = api.own_punishments().await?;
            link.send(ToMod::OwnPunishments { punishments });
            Ok(None)
        }
        other => social_intent(ctx, link, api, other).await.map(|()| None),
    }
}

/// The player's panel. Nothing here touches a case.
async fn social_intent(ctx: &Ctx, link: &ModLink, api: &Api, frame: ToLauncher) -> Answer<()> {
    match frame {
        ToLauncher::RequestNews => match link.news() {
            Some(items) => link.send(ToMod::News {
                posts: social::news_posts(ctx, &items),
            }),
            // The answer lands in the backend handler, which pushes it here.
            None => ctx.ws.send(schema::ClientWsMsg::RequestNews),
        },
        ToLauncher::RequestImage { url } => {
            let (ctx, link) = (ctx.clone(), link.clone());
            // Pictures are slow and the panel asks for several at once; none of
            // them should hold up the frame loop.
            tokio::spawn(async move { social::send_image(&ctx, &link, url).await });
        }
        ToLauncher::RequestChats => social::refresh_chats(ctx, link).await?,
        ToLauncher::OpenChat { peer_id } => {
            link.set_open_chat(Some(peer_id));
            social::refresh_chat(ctx, link, peer_id).await?;
            social::refresh_chats(ctx, link).await?;
        }
        ToLauncher::CloseChat => link.set_open_chat(None),
        ToLauncher::SendChat { peer_id, body } => {
            social::send_chat(api, peer_id, body).await?;
            // The master echoes it back as a `DirectMessage`, but only if the
            // launcher's socket is up — re-reading here doesn't depend on that.
            social::refresh_chat(ctx, link, peer_id).await?;
            social::refresh_chats(ctx, link).await?;
        }
        ToLauncher::RequestTickets => social::refresh_tickets(ctx, link).await?,
        ToLauncher::OpenTicket { ticket_id } => {
            link.set_open_ticket(Some(ticket_id));
            social::refresh_ticket(ctx, link, ticket_id).await?;
        }
        ToLauncher::CloseTicket => link.set_open_ticket(None),
        ToLauncher::SetSilentJoin { silent } => {
            let user: schema::UserProfile = api
                .json(
                    api.req(reqwest::Method::PUT, "/api/me/silent-join")
                        .json(&json!({ "silent": silent })),
                )
                .await?;
            // The answer is the whole profile; the switch shows what the master
            // kept, not what was asked for.
            ctx.set_profile(Some(user.clone()));
            link.send(push::profile_of(user));
        }
        ToLauncher::CreateTicket { subject, content } => {
            let id = social::create_ticket(api, subject, content).await?;
            link.set_open_ticket(Some(id));
            social::refresh_ticket(ctx, link, id).await?;
        }
        ToLauncher::ReplyTicket { ticket_id, content } => {
            social::reply_ticket(api, ticket_id, content).await?;
            social::refresh_ticket(ctx, link, ticket_id).await?;
        }
        // Everything else is a case intent and was matched before this.
        _ => {}
    }
    Ok(())
}

/// Frame name for `Rejected` — the mod needs it to know which button to undim.
fn intent_name(frame: &ToLauncher) -> &'static str {
    match frame {
        ToLauncher::Hello { .. } => "Hello",
        ToLauncher::RequestQueue => "RequestQueue",
        ToLauncher::RequestQueuePage { .. } => "RequestQueuePage",
        ToLauncher::OpenCase { .. } => "OpenCase",
        ToLauncher::CloseCase => "CloseCase",
        ToLauncher::Claim { .. } => "Claim",
        ToLauncher::Release { .. } => "Release",
        ToLauncher::Resolve { .. } => "Resolve",
        ToLauncher::AddNote { .. } => "AddNote",
        ToLauncher::Punish { .. } => "Punish",
        ToLauncher::RequestChat { .. } => "RequestChat",
        ToLauncher::RequestInventory { .. } => "RequestInventory",
        ToLauncher::Attach { .. } => "Attach",
        ToLauncher::Quote { .. } => "Quote",
        ToLauncher::Lookup { .. } => "Lookup",
        ToLauncher::RequestRules => "RequestRules",
        ToLauncher::RequestOwnPunishments => "RequestOwnPunishments",
        ToLauncher::RequestNews => "RequestNews",
        ToLauncher::RequestImage { .. } => "RequestImage",
        ToLauncher::RequestChats => "RequestChats",
        ToLauncher::OpenChat { .. } => "OpenChat",
        ToLauncher::CloseChat => "CloseChat",
        ToLauncher::SendChat { .. } => "SendChat",
        ToLauncher::RequestTickets => "RequestTickets",
        ToLauncher::OpenTicket { .. } => "OpenTicket",
        ToLauncher::CloseTicket => "CloseTicket",
        ToLauncher::CreateTicket { .. } => "CreateTicket",
        ToLauncher::ReplyTicket { .. } => "ReplyTicket",
        ToLauncher::SetSilentJoin { .. } => "SetSilentJoin",
    }
}
