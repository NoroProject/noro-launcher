//! OAuth2-приложения: заводит игрок, проверяет оператор.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Состояние приложения.
///
/// Новое приложение живёт в `Pending` и работает только для своего автора —
/// иначе разработчику пришлось бы ждать модерации, чтобы просто проверить, что
/// интеграция вообще собрана правильно.
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_APPROVED: &str = "approved";
pub const STATUS_REJECTED: &str = "rejected";
pub const STATUS_SUSPENDED: &str = "suspended";

pub const ALL_STATUSES: &[&str] = &[
    STATUS_PENDING,
    STATUS_APPROVED,
    STATUS_REJECTED,
    STATUS_SUSPENDED,
];

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct OAuthApp {
    pub id: Uuid,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub client_secret_hash: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    /// Адреса возврата, по одному в строке.
    pub redirect_uris: String,
    /// Наше собственное приложение: модерацию не проходит и общим выключателем
    /// сторонних не гасится.
    pub is_official: bool,
    /// Приложению негде хранить секрет — оно стоит у игрока на машине.
    /// Такому коду верят по PKCE, а не по `client_secret`.
    pub is_public: bool,
    pub owner_id: Option<Uuid>,
    pub status: String,
    /// Потолок доступа: больше этого приложение не выпросит и у игрока.
    pub allowed_scopes: String,
    /// Что оператор ответил автору при отклонении или блокировке.
    pub review_note: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OAuthApp {
    /// Может ли приложение сейчас авторизовать этого игрока.
    ///
    /// До одобрения приложение видит только своего автора: так интеграцию можно
    /// собрать и проверить целиком, но чужие аккаунты в неё не утекут.
    pub fn usable_by(&self, user_id: Uuid) -> bool {
        match self.status.as_str() {
            STATUS_APPROVED => true,
            STATUS_PENDING => self.owner_id == Some(user_id),
            _ => false,
        }
    }

    pub fn redirect_list(&self) -> Vec<&str> {
        self.redirect_uris
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Точное совпадение адреса возврата.
    ///
    /// Без префиксов и подстрок: `https://app.example/cb` не должен разрешать
    /// `https://app.example/cb.evil.tld`, а именно так «начинается с» и работает.
    ///
    /// Исключение — петля у публичного клиента. Лаунчер поднимает временный
    /// сервер на свободном порту и заранее не знает, на каком именно; порт в
    /// адресе петли поэтому не сверяется (RFC 8252 §7.3). Наружу это ничего не
    /// открывает: `127.0.0.1` — это машина самого игрока.
    pub fn allows_redirect(&self, uri: &str) -> bool {
        if self.redirect_list().contains(&uri) {
            return true;
        }
        self.is_public && self.allows_loopback(uri)
    }

    fn allows_loopback(&self, uri: &str) -> bool {
        let Ok(asked) = url::Url::parse(uri) else {
            return false;
        };
        if !matches!(asked.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")) {
            return false;
        }
        self.redirect_list().iter().any(|known| {
            url::Url::parse(known).is_ok_and(|k| {
                k.scheme() == asked.scheme()
                    && k.host_str() == asked.host_str()
                    // Зарегистрирован голый хост — принимаем любой путь на нём.
                    && (k.path() == "/" || k.path() == asked.path())
            })
        })
    }

    pub fn allowed_scope_list(&self) -> Vec<String> {
        schema::parse_scopes(&self.allowed_scopes)
    }
}

const COLUMNS: &str = "id, client_id, client_secret_hash, name, icon_url, description,
     redirect_uris, is_official, is_public, owner_id, status, allowed_scopes, review_note,
     reviewed_by, reviewed_at, created_at, updated_at";

pub async fn app_by_client_id(db: &PgPool, client_id: &str) -> Result<Option<OAuthApp>> {
    let sql = format!("SELECT {COLUMNS} FROM oauth_applications WHERE client_id = $1");
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(client_id)
        .fetch_optional(db)
        .await?)
}

pub async fn app_by_id(db: &PgPool, id: Uuid) -> Result<Option<OAuthApp>> {
    let sql = format!("SELECT {COLUMNS} FROM oauth_applications WHERE id = $1");
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?)
}

pub async fn apps_by_owner(db: &PgPool, owner_id: Uuid) -> Result<Vec<OAuthApp>> {
    let sql =
        format!("SELECT {COLUMNS} FROM oauth_applications WHERE owner_id = $1 ORDER BY created_at");
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(owner_id)
        .fetch_all(db)
        .await?)
}

/// Все приложения инстанса; `status` сужает выборку до одной очереди.
pub async fn list_apps(db: &PgPool, status: Option<&str>) -> Result<Vec<OAuthApp>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM oauth_applications
         WHERE ($1::text IS NULL OR status = $1)
         ORDER BY is_official DESC, created_at DESC"
    );
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(status)
        .fetch_all(db)
        .await?)
}

pub struct NewApp<'a> {
    pub owner_id: Uuid,
    pub client_id: &'a str,
    pub secret_hash: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub redirect_uris: &'a str,
    /// Что приложение будет просить у игроков — выбор владельца из базовых.
    pub scopes: &'a str,
}

pub async fn create_app(db: &PgPool, app: NewApp<'_>) -> Result<OAuthApp> {
    let sql = format!(
        "INSERT INTO oauth_applications
             (client_id, client_secret_hash, name, description, redirect_uris,
              owner_id, status, allowed_scopes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(app.client_id)
        .bind(app.secret_hash)
        .bind(app.name)
        .bind(app.description)
        .bind(app.redirect_uris)
        .bind(app.owner_id)
        .bind(STATUS_PENDING)
        .bind(app.scopes)
        .fetch_one(db)
        .await?)
}

/// Правка от автора. Возвращается к модерации: изменился адрес возврата или
/// название — оператор их не видел.
pub async fn update_app(
    db: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    redirect_uris: &str,
) -> Result<Option<OAuthApp>> {
    let sql = format!(
        "UPDATE oauth_applications
            SET name = $2, description = $3, redirect_uris = $4, updated_at = NOW(),
                status = CASE WHEN is_official OR status = $5 THEN status ELSE $6 END,
                review_note = CASE WHEN is_official THEN review_note ELSE NULL END
          WHERE id = $1
        RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(redirect_uris)
        .bind(STATUS_SUSPENDED)
        .bind(STATUS_PENDING)
        .fetch_optional(db)
        .await?)
}

pub async fn set_icon(db: &PgPool, id: Uuid, url: &str) -> Result<()> {
    sqlx::query("UPDATE oauth_applications SET icon_url = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(url)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_secret_hash(db: &PgPool, id: Uuid, hash: &str) -> Result<()> {
    sqlx::query(
        "UPDATE oauth_applications SET client_secret_hash = $2, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(hash)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_app(db: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM oauth_applications WHERE id = $1 AND is_official = FALSE")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Решение оператора: одобрить, отклонить, заблокировать.
pub async fn set_status(
    db: &PgPool,
    id: Uuid,
    status: &str,
    note: Option<&str>,
    reviewer: Option<Uuid>,
) -> Result<Option<OAuthApp>> {
    let sql = format!(
        "UPDATE oauth_applications
            SET status = $2, review_note = $3, reviewed_by = $4, reviewed_at = NOW(),
                updated_at = NOW()
          WHERE id = $1
        RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(id)
        .bind(status)
        .bind(note)
        .bind(reviewer)
        .fetch_optional(db)
        .await?)
}

/// Потолок доступа приложения. Базовые scope'ы есть у всех, привилегированные
/// оператор выдаёт поимённо.
pub async fn set_allowed_scopes(db: &PgPool, id: Uuid, scopes: &str) -> Result<Option<OAuthApp>> {
    let sql = format!(
        "UPDATE oauth_applications SET allowed_scopes = $2, updated_at = NOW()
          WHERE id = $1 RETURNING {COLUMNS}"
    );
    Ok(sqlx::query_as::<_, OAuthApp>(&sql)
        .bind(id)
        .bind(scopes)
        .fetch_optional(db)
        .await?)
}

/// Сколько игроков впустили приложение к себе — для админского списка.
pub async fn authorization_counts(db: &PgPool) -> Result<Vec<(Uuid, i64)>> {
    Ok(sqlx::query_as::<_, (Uuid, i64)>(
        "SELECT app_id, COUNT(*) FROM user_authorized_apps GROUP BY app_id",
    )
    .fetch_all(db)
    .await?)
}
