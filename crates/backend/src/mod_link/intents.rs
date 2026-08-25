//! Намерение мода → запрос к мастеру.
//!
//! Мод не знает ни одного URL: здесь и только здесь намерение превращается в
//! ручку. Отказ мастера уходит обратно `Rejected` с ключом перевода — решает
//! мастер, и его 403 единственная проверка, которой стоит верить.

use super::master::{Api, Denied};
use super::push;
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
            // Моду уходит ключ, в лог — причина: «мастер недоступен» и «мастер
            // ответил 500» для модератора одно и то же, а для разбора нет.
            if let Denied::Offline(why) = &e {
                tracing::debug!(intent = name, "mod_link: мастер не ответил: {why}");
            }
            return link.send(ToMod::Rejected {
                intent: name.into(),
                reason: e.key().into(),
                number: e.number(),
            });
        }
    };
    // Всё, что меняет дело, меняет и очередь: замок и статус видны в обеих.
    if let Some(case_id) = case_id {
        push::refresh_case(ctx, link, case_id).await;
        push::refresh_queue_in_place(ctx, link).await;
    }
}

/// `Ok(Some(case_id))` — дело изменилось и его надо перечитать.
async fn dispatch(
    ctx: &Ctx,
    link: &ModLink,
    api: &Api,
    frame: ToLauncher,
) -> Result<Option<Uuid>, Denied> {
    match frame {
        // Рукопожатие приняли раньше, повторное ничего не значит.
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
            // Наказывают игрока, а не дело: id цели берём из карточки.
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
            // Срез снимает агент — карточка догонит следующим `CaseUpdated`.
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
            let rules = api.rules().await?;
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
    }
}

/// Имя кадра для `Rejected`: моду нужно понять, какую кнопку гасить.
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
    }
}
