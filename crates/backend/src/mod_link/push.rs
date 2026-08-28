//! Что лаунчер шлёт моду сам: очередь, карточка, готовность.
//!
//! Мод не «запрашивает и ждёт ответ» — он подписан. Сходить за карточкой и
//! разослать её решает эта сторона, поэтому и в протоколе нет ни request-id,
//! ни корреляции ответов.

use super::master::Api;
use super::ModLink;
use crate::backend::Ctx;
use mod_link::{CaseView, ToMod, PROTOCOL};
use uuid::Uuid;

/// Первый кадр после рукопожатия. Права — чтобы мод не рисовал кнопки, которых
/// всё равно не дадут нажать; это удобство, а не защита.
pub fn ready(ctx: &Ctx) -> ToMod {
    let profile = ctx.profile();
    ToMod::Ready {
        protocol: PROTOCOL,
        username: profile
            .as_ref()
            .map(|u| u.username.clone())
            .unwrap_or_default(),
        locale: ctx.config.get().locale,
        // Права с ролями вместе: прямых у модератора обычно нет вовсе, они
        // приходят ролью, и кнопки рисовались бы по пустому списку.
        permissions: profile
            .as_ref()
            .map(|u| u.all_permissions().map(str::to_string).collect())
            .unwrap_or_default(),
    }
}

/// Прочитать страницу очереди и разослать её.
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
        Err(e) => tracing::debug!("mod_link: очередь не прочиталась: {e:?}"),
    }
}

/// Перечитать ту же страницу, что модератор смотрит сейчас.
///
/// Зовётся на `CaseUpdated` с мастера. Именно ту же, а не первую: иначе любое
/// чужое действие над любым делом выбрасывало бы его в начало очереди.
pub async fn refresh_queue_in_place(ctx: &Ctx, link: &ModLink) {
    let (query, offset) = link.store().queue_spot();
    refresh_queue(ctx, link, query, offset).await;
}

/// Перечитать карточку и разослать её. Дело становится открытым: дальше оно
/// обновляется само на каждое `CaseUpdated` с мастера.
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

/// Снимок инвентаря отдельным кадром: в ленте он лежит сырым JSON, и разбирать
/// его в Java — работа на ровном месте.
fn inventory(view: &CaseView) -> Option<ToMod> {
    let event = view
        .events
        .iter()
        .rev()
        .find(|e| e.kind == "inventory_snapshot")?;
    // Слоты приезжают от агента как есть. Битую запись пропускаем, а не роняем
    // весь снимок: один странный предмет не повод не показать остальные.
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
