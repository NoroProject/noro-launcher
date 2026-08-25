use crate::db::models::ModSuggestionRow;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Предложение мода от игрока.
pub struct NewModSuggestion<'a> {
    pub server_id: Uuid,
    pub build_id: Option<Uuid>,
    pub provider: &'a str,
    pub project_id: &'a str,
    pub title: &'a str,
    pub icon_url: Option<&'a str>,
    pub description: Option<&'a str>,
    pub suggested_by: Uuid,
}

pub async fn create_mod_suggestion(
    pool: &PgPool,
    s: NewModSuggestion<'_>,
) -> Result<ModSuggestionRow> {
    let NewModSuggestion {
        server_id,
        build_id,
        provider,
        project_id,
        title,
        icon_url,
        description,
        suggested_by,
    } = s;
    let row = sqlx::query_as::<_, ModSuggestionRow>(
        r#"
        INSERT INTO mod_suggestions 
            (server_id, build_id, provider, project_id, title, icon_url, description, suggested_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, server_id, build_id, provider, project_id, title, icon_url, description, suggested_by, status, created_at
        "#,
    )
    .bind(server_id)
    .bind(build_id)
    .bind(provider)
    .bind(project_id)
    .bind(title)
    .bind(icon_url)
    .bind(description)
    .bind(suggested_by)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_mod_suggestions(
    pool: &PgPool,
    server_id: Uuid,
    status: Option<&str>,
) -> Result<Vec<ModSuggestionRow>> {
    let rows = if let Some(st) = status {
        sqlx::query_as::<_, ModSuggestionRow>(
            r#"
            SELECT s.id, s.server_id, s.build_id, s.provider, s.project_id, s.title,
                   s.icon_url, s.description, s.suggested_by, u.mc_username AS suggested_by_name,
                   s.status, s.created_at
            FROM mod_suggestions s
            LEFT JOIN users u ON u.id = s.suggested_by
            WHERE s.server_id = $1 AND s.status = $2
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(server_id)
        .bind(st)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ModSuggestionRow>(
            r#"
            SELECT s.id, s.server_id, s.build_id, s.provider, s.project_id, s.title,
                   s.icon_url, s.description, s.suggested_by, u.mc_username AS suggested_by_name,
                   s.status, s.created_at
            FROM mod_suggestions s
            LEFT JOIN users u ON u.id = s.suggested_by
            WHERE s.server_id = $1
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

pub async fn get_mod_suggestion(pool: &PgPool, id: Uuid) -> Result<Option<ModSuggestionRow>> {
    let row = sqlx::query_as::<_, ModSuggestionRow>(
        r#"
        SELECT id, server_id, build_id, provider, project_id, title, icon_url, description, suggested_by, status, created_at
        FROM mod_suggestions
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn update_mod_suggestion_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Option<ModSuggestionRow>> {
    let row = sqlx::query_as::<_, ModSuggestionRow>(
        r#"
        UPDATE mod_suggestions
        SET status = $2
        WHERE id = $1
        RETURNING id, server_id, build_id, provider, project_id, title, icon_url, description, suggested_by, status, created_at
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
