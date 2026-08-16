//! Новости для ленты лаунчера.

use crate::state::AppState;

pub async fn news_items(
    state: &AppState,
    rows: Vec<crate::db::models::NewsRow>,
) -> anyhow::Result<Vec<schema::NewsItem>> {
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let author_name = match r.author_id {
            Some(id) => crate::db::get_user(&state.db, id)
                .await?
                .map(|u| u.mc_username),
            None => None,
        };
        items.push(schema::NewsItem {
            id: r.id,
            title: r.title,
            body: r.body,
            preview_img_url: r.preview_img_url,
            author_name,
            pinned: r.pinned,
            published_at: r.published_at,
        });
    }
    Ok(items)
}
