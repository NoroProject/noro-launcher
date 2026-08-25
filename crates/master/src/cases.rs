//! Дела: событие ленты и пуш тем, кто на дело смотрит.
//!
//! Запись в ленту и кадр «карточка изменилась» ходят парой. Разделить их —
//! значит однажды забыть второе: половина событий приходит от агента, половина
//! из админки, и «а этот путь пушит?» превращается в вопрос без ответа.
//!
//! Кто смотрит — тот, за кем замок. Дело без замка никому не открыто: очередь
//! перечитывается при открытии, ей пуш не нужен.

use crate::state::AppState;
use anyhow::Result;
use schema::ServerWsMsg;
use uuid::Uuid;

/// Записать событие дела и толкнуть модератора, который его ведёт.
pub async fn event(
    state: &AppState,
    case_id: Uuid,
    actor_id: Option<Uuid>,
    actor_label: &str,
    source: &str,
    kind: &str,
    payload: serde_json::Value,
) -> Result<()> {
    crate::db::add_event(
        &state.db,
        case_id,
        actor_id,
        actor_label,
        source,
        kind,
        payload,
    )
    .await?;
    updated(state, case_id).await;
    Ok(())
}

/// Кадр «карточка изменилась» модератору, за которым замок.
///
/// Молчит, когда замка нет или лаунчер не в сети: пуш — удобство, а не
/// доставка. Состояние всё равно лежит на мастере и перечитывается запросом.
pub async fn updated(state: &AppState, case_id: Uuid) {
    // Вкладкам админки — всем: на карточку смотрит и тот, кто дело не брал.
    // В кадре нет данных, а за самой карточкой страница идёт обычным запросом,
    // где права и проверяются.
    state
        .admin_ws
        .broadcast(&schema::AdminWsMsg::CaseUpdated { case_id });

    let Ok(Some(case)) = crate::db::get_case(&state.db, case_id).await else {
        return;
    };
    let Some(owner) = case.claimed_by else {
        return;
    };
    state
        .ws
        .send_to_user(owner, &ServerWsMsg::CaseUpdated { case_id });
}
