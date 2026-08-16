//! Локальные аккаунты и recovery-коды.

use crate::api::auth::admin_token;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::UserRow;

/// Сколько кодов выдаётся разом. Десять — чтобы хватило на несколько входов до
/// настройки домена, но не столько, чтобы их перестали беречь.
pub const RECOVERY_CODE_COUNT: usize = 10;

/// Завести локальный аккаунт.
///
/// `mc_uuid` генерируется, но играть аккаунт по умолчанию не может: это
/// операторский вход, а лишний игровой профиль — лишняя поверхность.
pub async fn create_local_account(pool: &PgPool, username: &str, root: bool) -> Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (mc_uuid, mc_username, is_local_account, can_play, is_root)
         VALUES (gen_random_uuid(), $1, TRUE, FALSE, $2)
         RETURNING *",
    )
    .bind(username)
    .bind(root)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Есть ли уже root. Второй завести нельзя — уникальный частичный индекс не
/// даст, но спросить об этом лучше заранее, чем ловить ошибку БД.
pub async fn root_exists(pool: &PgPool) -> Result<bool> {
    Ok(
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE is_root)")
            .fetch_one(pool)
            .await?,
    )
}

/// Root ли это. Единственный аккаунт, который нельзя забанить и удалить.
pub async fn is_root_user(pool: &PgPool, id: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE((SELECT is_root FROM users WHERE id = $1), FALSE)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

/// Выпустить новый набор кодов, погасив прежние.
///
/// Возвращает коды в открытом виде — единственный момент, когда их видно.
pub async fn issue_recovery_codes(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM recovery_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    let mut codes = Vec::with_capacity(RECOVERY_CODE_COUNT);
    for _ in 0..RECOVERY_CODE_COUNT {
        let code = generate_code();
        let hash = admin_token::hash(&code)?;
        sqlx::query("INSERT INTO recovery_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(&hash)
            .execute(&mut *tx)
            .await?;
        codes.push(code);
    }
    tx.commit().await?;
    Ok(codes)
}

/// Формат `xxxx-xxxx-xxxx`: читается вслух и переписывается с бумаги.
fn generate_code() -> String {
    use rand::Rng;
    // Без похожих друг на друга символов: 0/O и 1/l/I переписывают неверно.
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let group = |rng: &mut rand::rngs::ThreadRng| -> String {
        (0..4)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect()
    };
    format!(
        "{}-{}-{}",
        group(&mut rng),
        group(&mut rng),
        group(&mut rng)
    )
}

/// Погасить код и вернуть владельца.
///
/// Коды солёные, найти строку по коду нельзя — перебираем непогашенные. Их не
/// больше десяти на аккаунт, так что цена проверки ограничена.
pub async fn consume_recovery_code(
    pool: &PgPool,
    username: &str,
    code: &str,
) -> Result<Option<Uuid>> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid, String)>(
        "SELECT r.id, r.user_id, r.code_hash
         FROM recovery_codes r
         JOIN users u ON u.id = r.user_id
         WHERE r.used_at IS NULL AND u.mc_username = $1",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;

    for (id, user_id, hash) in rows {
        if admin_token::verify(code, &hash) {
            // Гасим сразу: код одноразовый, и повторное использование — это
            // либо ошибка, либо чужая попытка.
            let burned = sqlx::query(
                "UPDATE recovery_codes SET used_at = NOW() WHERE id = $1 AND used_at IS NULL",
            )
            .bind(id)
            .execute(pool)
            .await?;
            if burned.rows_affected() == 1 {
                return Ok(Some(user_id));
            }
        }
    }
    Ok(None)
}

/// Сколько кодов осталось. Меньше трёх — повод предупредить в админке.
pub async fn recovery_codes_left(pool: &PgPool, user_id: Uuid) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM recovery_codes WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}
