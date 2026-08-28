//! Duplicating a whole build.
//!
//! Nothing is copied on disk: the FileStore is addressed by sha1, so the new
//! build points at the same objects. A copy costs a few row inserts, not a
//! re-upload of the modpack.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Copy a build into a new version of the same server.
///
/// Settings and the file list carry over. Publication does not (the copy is
/// always a draft), nor does the manifest signature — same files, but a
/// different `version` in the manifest, so it has to be signed again.
///
/// Columns are deliberately not listed by name: `to_jsonb(b)` takes them all,
/// so a new column in `builds` ends up in the copy on its own. An explicit list
/// would have to be maintained by hand, and a forgotten setting would surface
/// on a player's machine rather than here.
pub async fn duplicate_build(pool: &PgPool, src: Uuid, version: &str) -> Result<Uuid> {
    let mut tx = pool.begin().await?;

    let new_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO builds
        SELECT (jsonb_populate_record(
            NULL::builds,
            to_jsonb(b) || jsonb_build_object(
                'id', gen_random_uuid(),
                'version', $2::text,
                'published', false,
                'manifest_signature', NULL,
                'created_at', now()
            )
        )).*
        FROM builds b
        WHERE b.id = $1
        RETURNING id
        "#,
    )
    .bind(src)
    .bind(version)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO build_files
        SELECT (jsonb_populate_record(
            NULL::build_files,
            to_jsonb(f) || jsonb_build_object('id', gen_random_uuid(), 'build_id', $2::uuid)
        )).*
        FROM build_files f
        WHERE f.build_id = $1
        "#,
    )
    .bind(src)
    .bind(new_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(new_id)
}

pub async fn build_server_id(pool: &PgPool, build_id: Uuid) -> Result<Option<Uuid>> {
    Ok(
        sqlx::query_scalar("SELECT server_id FROM builds WHERE id = $1")
            .bind(build_id)
            .fetch_optional(pool)
            .await?,
    )
}
