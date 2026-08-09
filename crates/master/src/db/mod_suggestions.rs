use crate::db::models::ModSuggestionRow;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_mod_suggestion(
    pool: &PgPool,
    server_id: Uuid,
    build_id: Option<Uuid>,
    provider: &str,
    project_id: &str,
    title: &str,
    icon_url: Option<&str>,
    description: Option<&str>,
    suggested_by: Uuid,
) -> Result<ModSuggestionRow> {
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
            SELECT id, server_id, build_id, provider, project_id, title, icon_url, description, suggested_by, status, created_at
            FROM mod_suggestions
            WHERE server_id = $1 AND status = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .bind(st)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ModSuggestionRow>(
            r#"
            SELECT id, server_id, build_id, provider, project_id, title, icon_url, description, suggested_by, status, created_at
            FROM mod_suggestions
            WHERE server_id = $1
            ORDER BY created_at DESC
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
