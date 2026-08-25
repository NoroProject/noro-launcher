//! Очередь дел и репутация репортеров.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseListRow {
    pub id: Uuid,
    pub number: i64,
    pub target_id: Uuid,
    pub target_name: Option<String>,
    pub game_server_id: Option<Uuid>,
    pub server_name: Option<String>,
    pub status: String,
    pub claimed_by: Option<Uuid>,
    pub claimed_by_name: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub verdict: Option<String>,
    pub rule_code: Option<String>,
    /// Жалоб в деле и сколько *разных* людей их написали: десять жалоб от
    /// одного обиженного и от десяти разных — разный вес.
    pub reports_count: i64,
    pub reporters_count: i64,
    pub last_report_at: Option<DateTime<Utc>>,
}

/// Источник строк. Вынесен отдельно от списка колонок, чтобы счётчик считался
/// по тем же соединениям, что и выборка: поиск идёт по именам из `users` и
/// `game_servers`, и `count(*)` без них дал бы другое число.
const FROM_SQL: &str = "
      FROM cases c
      LEFT JOIN users tu ON tu.id = c.target_id
      LEFT JOIN users mu ON mu.id = c.claimed_by
      LEFT JOIN game_servers gs ON gs.id = c.game_server_id
      LEFT JOIN (
            SELECT case_id,
                   COUNT(*)                    AS reports_count,
                   COUNT(DISTINCT reporter_id) AS reporters_count,
                   MAX(created_at)             AS last_report_at
              FROM player_reports WHERE case_id IS NOT NULL GROUP BY case_id
      ) r ON r.case_id = c.id
";

const COLUMNS_SQL: &str = "
    SELECT c.*,
           tu.mc_username AS target_name,
           gs.name        AS server_name,
           mu.mc_username AS claimed_by_name,
           COALESCE(r.reports_count, 0)   AS reports_count,
           COALESCE(r.reporters_count, 0) AS reporters_count,
           r.last_report_at
";

/// Поиск по очереди: ник нарушителя, сервер, кто взял дело, номер дела.
///
/// Номер сравнивается как текст: модератор вводит «142» из чужого сообщения и
/// ждёт, что найдётся дело №142, а не диапазон.
///
/// `$1::text IS NULL` — это «поиска нет». Условие стоит в запросе всегда, чтобы
/// плейсхолдер не пропадал: Postgres выводит их число по наибольшему
/// упомянутому, и запрос, где есть $2 и $3, но нет $1, не готовится вовсе.
const SEARCH_SQL: &str = "($1::text IS NULL OR (
       tu.mc_username ILIKE $1 ESCAPE '\\'
    OR gs.name        ILIKE $1 ESCAPE '\\'
    OR mu.mc_username ILIKE $1 ESCAPE '\\'
    OR c.number::text LIKE  $1 ESCAPE '\\'
))";

/// Очередь. Открытые сверху и по числу разных жалобщиков: пять человек про
/// чит важнее одной жалобы на мат, и разбирать надо с них.
///
/// Возвращает и счётчик по тому же условию. Раньше открытые дела отдавались без
/// лимита вовсе, а архив — с зашитым `LIMIT 100` и без счётчика: сто первое дело
/// не показывалось и не считалось, и по интерфейсу это было неотличимо от «дел
/// больше нет».
///
/// `like` — шаблон `%…%` с уже экранированными подстановочными знаками (см.
/// `PageQuery::like`). Ищет мастер, а не клиент: раньше админка и мод тянули
/// очередь целиком и фильтровали у себя, поэтому находилось только то, что
/// попало в загруженную страницу.
pub async fn list_cases(
    pool: &PgPool,
    open_only: bool,
    like: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<CaseListRow>, i64)> {
    let where_sql = if open_only {
        format!("WHERE c.status IN ('open', 'in_review') AND {SEARCH_SQL}")
    } else {
        format!("WHERE {SEARCH_SQL}")
    };

    let order_sql = if open_only {
        "ORDER BY COALESCE(r.reporters_count, 0) DESC, c.opened_at"
    } else {
        "ORDER BY c.opened_at DESC"
    };

    let rows = sqlx::query_as::<_, CaseListRow>(&format!(
        "{COLUMNS_SQL} {FROM_SQL} {where_sql} {order_sql} LIMIT $2 OFFSET $3"
    ))
    .bind(like)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) {FROM_SQL} {where_sql}"))
        .bind(like)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn get_case_view(pool: &PgPool, id: Uuid) -> Result<Option<CaseListRow>> {
    let sql = format!("{COLUMNS_SQL} {FROM_SQL} WHERE c.id = $1");
    Ok(sqlx::query_as::<_, CaseListRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Сколько жалоб человека подтвердилось. Не хранится колонкой: строк на
/// игрока десятки, а лишнее поле пришлось бы чинить после каждой правки дела.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReporterStats {
    pub total: i64,
    pub confirmed: i64,
    pub rejected: i64,
}

pub async fn reporter_stats(pool: &PgPool, user_id: Uuid) -> Result<ReporterStats> {
    Ok(sqlx::query_as::<_, ReporterStats>(
        "SELECT COUNT(*) AS total,
                COUNT(*) FILTER (WHERE c.verdict = 'confirmed') AS confirmed,
                COUNT(*) FILTER (WHERE c.verdict = 'rejected')  AS rejected
           FROM player_reports p
           LEFT JOIN cases c ON c.id = p.case_id
          WHERE p.reporter_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// Сколько дел завели на игрока и сколько из них подтвердилось. Тот же счёт,
/// что у жалобщика, но с другой стороны: там вес слова, здесь — история.
pub async fn target_stats(pool: &PgPool, user_id: Uuid) -> Result<ReporterStats> {
    Ok(sqlx::query_as::<_, ReporterStats>(
        "SELECT COUNT(*) AS total,
                COUNT(*) FILTER (WHERE verdict = 'confirmed') AS confirmed,
                COUNT(*) FILTER (WHERE verdict = 'rejected')  AS rejected
           FROM cases WHERE target_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// Жалобы внутри дела — с именем автора и его репутацией.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseReportRow {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_name: Option<String>,
    pub reason: String,
    pub world: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub created_at: DateTime<Utc>,
}

pub async fn case_reports(pool: &PgPool, case_id: Uuid) -> Result<Vec<CaseReportRow>> {
    Ok(sqlx::query_as::<_, CaseReportRow>(
        "SELECT p.id, p.reporter_id, u.mc_username AS reporter_name, p.reason,
                p.world, p.x, p.y, p.z, p.created_at
           FROM player_reports p
           LEFT JOIN users u ON u.id = p.reporter_id
          WHERE p.case_id = $1 ORDER BY p.created_at",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await?)
}
