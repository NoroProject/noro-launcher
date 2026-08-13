//! Копирование сборки целиком.
//!
//! Файлы при этом не копируются физически: FileStore адресуется по sha1, и
//! новая сборка ссылается на те же объекты. Стоимость копии — вставка строк,
//! а не перезаливка модпака.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Скопировать сборку в новую версию того же сервера.
///
/// Переносятся все настройки и весь список файлов. Не переносятся:
/// публикация (копия всегда черновик), подпись манифеста (её надо ставить
/// заново — файлы те же, но `version` в манифесте другой) и дата создания.
///
/// Колонки не перечисляются поимённо намеренно: `to_jsonb(b)` берёт их все, и
/// новая колонка в `builds` попадает в копию сама. Явный список пришлось бы
/// дополнять руками, а забытая настройка проявилась бы не при копировании, а
/// у игрока — расхождением поведения между версиями сборки.
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

/// Сервер, которому принадлежит сборка.
pub async fn build_server_id(pool: &PgPool, build_id: Uuid) -> Result<Option<Uuid>> {
    Ok(
        sqlx::query_scalar("SELECT server_id FROM builds WHERE id = $1")
            .bind(build_id)
            .fetch_optional(pool)
            .await?,
    )
}
