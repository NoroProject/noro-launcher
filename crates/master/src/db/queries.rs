//! Запросы к БД, сгруппированные по сущностям. Часть функций возвращают
//! доменные типы из `schema`, часть — row-структуры для админки.

use super::models::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use schema::{Modloader, PermissionGrant, Role, ServerEntry, UserProfile};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Пользователи
// ---------------------------------------------------------------------------

/// Найти пользователя по Discord ID или создать нового (первый вход).
/// Возвращает (user_id, is_new).
pub async fn find_or_create_user(
    pool: &PgPool,
    discord_id: &str,
    discord_username: &str,
    discord_avatar: Option<&str>,
) -> Result<(Uuid, bool)> {
    if let Some(row) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE discord_id = $1")
        .bind(discord_id)
        .fetch_optional(pool)
        .await?
    {
        // Обновим актуальные данные Discord.
        sqlx::query(
            "UPDATE users SET discord_username = $2, discord_avatar = $3, last_login_at = NOW() WHERE id = $1",
        )
        .bind(row.id)
        .bind(discord_username)
        .bind(discord_avatar)
        .execute(pool)
        .await?;
        return Ok((row.id, false));
    }

    let mc_uuid = schema::mc_uuid_from_discord(discord_id);
    let mc_username = unique_mc_username(pool, discord_username).await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (discord_id, discord_username, discord_avatar, mc_uuid, mc_username, last_login_at)
         VALUES ($1, $2, $3, $4, $5, NOW()) RETURNING id",
    )
    .bind(discord_id)
    .bind(discord_username)
    .bind(discord_avatar)
    .bind(mc_uuid)
    .bind(&mc_username)
    .fetch_one(pool)
    .await?;

    // Выдать дефолтные роли.
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE is_default = TRUE
         ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok((id, true))
}

/// Подобрать уникальный MC-ник: санитизация + суффикс при коллизии.
async fn unique_mc_username(pool: &PgPool, discord_username: &str) -> Result<String> {
    let base: String = discord_username
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(14)
        .collect();
    let base = if base.is_empty() {
        "player".to_string()
    } else {
        base
    };

    for attempt in 0..1000 {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            let suffix = attempt.to_string();
            format!("{}{}", &base[..base.len().min(16 - suffix.len())], suffix)
        };
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE mc_username = $1)")
                .bind(&candidate)
                .fetch_one(pool)
                .await?;
        if !exists {
            return Ok(candidate);
        }
    }
    anyhow::bail!("не удалось подобрать уникальный MC-ник")
}

/// Собрать полный UserProfile (роли + права).
pub async fn load_profile(pool: &PgPool, user_id: Uuid) -> Result<UserProfile> {
    let u = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .context("пользователь не найден")?;
    profile_from_row(pool, u).await
}

pub async fn profile_from_row(pool: &PgPool, u: UserRow) -> Result<UserProfile> {
    let roles = load_user_roles(pool, u.id).await?;
    let grants: Vec<(String, Option<Uuid>)> =
        sqlx::query_as("SELECT permission, server_id FROM user_permissions WHERE user_id = $1")
            .bind(u.id)
            .fetch_all(pool)
            .await?;
    let permissions = grants.iter().map(|(p, _)| p.clone()).collect();
    let permission_grants = grants
        .into_iter()
        .map(|(permission, server_id)| PermissionGrant {
            permission,
            server_id,
        })
        .collect();
    Ok(UserProfile {
        id: u.id,
        uuid: u.mc_uuid,
        username: u.mc_username,
        discord_id: u.discord_id,
        discord_username: u.discord_username,
        discord_avatar: u.discord_avatar,
        skin_url: u.skin_url,
        cape_url: u.cape_url,
        roles,
        permissions,
        permission_grants,
        banned: u.banned,
    })
}

/// Загрузить роли пользователя вместе с их правами.
pub async fn load_user_roles(pool: &PgPool, user_id: Uuid) -> Result<Vec<Role>> {
    let role_rows = sqlx::query_as::<_, RoleRow>(
        "SELECT r.* FROM roles r JOIN user_roles ur ON ur.role_id = r.id
         WHERE ur.user_id = $1 ORDER BY r.sort_order DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut roles = Vec::with_capacity(role_rows.len());
    for r in role_rows {
        roles.push(role_with_perms(pool, r).await?);
    }
    Ok(roles)
}

/// Права игрока, действующие на конкретной сборке: глобальные плюс
/// привязанные к ней. Свои и полученные через роли — одним списком.
///
/// Контекст сборки здесь единственное измерение, в отличие от общих контекстов
/// LuckPerms: больше у нас разделять нечего, а лишняя машинерия только мешала бы.
pub async fn effective_permissions(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Uuid,
) -> Result<Vec<String>> {
    let rows: Vec<String> = sqlx::query_scalar(
        // Роли игрока вместе с их предками: наследование должно доезжать до
        // игры так же, как оно видно в админке, иначе право «есть» на сайте и
        // «нет» на сервере.
        "WITH RECURSIVE granted AS ( \
             SELECT role_id AS id FROM user_roles WHERE user_id = $1 \
           UNION \
             SELECT r.parent_id FROM roles r \
               JOIN granted g ON r.id = g.id \
              WHERE r.parent_id IS NOT NULL \
         ) \
         SELECT permission FROM user_permissions \
          WHERE user_id = $1 AND (server_id IS NULL OR server_id = $2) \
         UNION \
         SELECT rp.permission FROM role_permissions rp \
           JOIN granted g ON g.id = rp.role_id \
          WHERE rp.server_id IS NULL OR rp.server_id = $2",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Полностью заменяет каталог узлов сервера тем, что прислал агент.
///
/// Именно замена, а не добавление: мод убрали со сборки — его узлы должны
/// исчезнуть из подсказок, иначе список за пару месяцев зарастёт мусором.
pub async fn replace_permission_nodes(
    pool: &PgPool,
    server_id: Uuid,
    nodes: &[String],
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM permission_nodes WHERE server_id = $1")
        .bind(server_id)
        .execute(&mut *tx)
        .await?;
    for node in nodes {
        sqlx::query(
            "INSERT INTO permission_nodes (server_id, node) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(server_id)
        .bind(node)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn permission_nodes(pool: &PgPool, server_id: Uuid) -> Result<Vec<String>> {
    Ok(
        sqlx::query_scalar("SELECT node FROM permission_nodes WHERE server_id = $1 ORDER BY node")
            .bind(server_id)
            .fetch_all(pool)
            .await?,
    )
}

/// Сборки, у которых есть опциональные моды — источник прав `noro.optional.*`.
pub async fn builds_with_optional_mods(pool: &PgPool) -> Result<Vec<BuildRow>> {
    Ok(sqlx::query_as::<_, BuildRow>(
        "SELECT * FROM builds WHERE jsonb_array_length(optional_mods) > 0",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn role_with_perms(pool: &PgPool, r: RoleRow) -> Result<Role> {
    let grants: Vec<(String, Option<Uuid>)> =
        sqlx::query_as("SELECT permission, server_id FROM role_permissions WHERE role_id = $1")
            .bind(r.id)
            .fetch_all(pool)
            .await?;
    let permissions = grants.iter().map(|(p, _)| p.clone()).collect();
    let permission_grants = grants
        .into_iter()
        .map(|(permission, server_id)| PermissionGrant {
            permission,
            server_id,
        })
        .collect();
    let inherited_permissions = inherited_role_permissions(pool, r.id).await?;
    Ok(Role {
        id: r.id,
        name: r.name,
        display_name: r.display_name,
        color: r.color,
        permissions,
        is_default: r.is_default,
        sort_order: r.sort_order,
        lp_group: r.lp_group,
        icon: r.icon,
        permission_grants,
        parent_id: r.parent_id,
        inherited_permissions,
    })
}

/// Права всех предков роли — родителя, его родителя и так далее.
///
/// Свои права роли сюда не попадают: их спрашивают отдельно, и админке нужно
/// видеть границу между «выдано здесь» и «пришло сверху».
///
/// UNION, а не UNION ALL: на UNION рекурсия останавливается, когда новых строк
/// больше нет, поэтому замкнутая цепочка ролей даёт конечный ответ, а не висящий
/// запрос. Циклы запрещает API, но резолвер прав не должен зависеть от того,
/// что данные в базе непременно правильные.
pub async fn inherited_role_permissions(pool: &PgPool, role_id: Uuid) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar(
        "WITH RECURSIVE ancestors AS (
             SELECT parent_id AS id FROM roles WHERE id = $1 AND parent_id IS NOT NULL
           UNION
             SELECT r.parent_id FROM roles r
               JOIN ancestors a ON r.id = a.id
              WHERE r.parent_id IS NOT NULL
         )
         SELECT DISTINCT rp.permission FROM role_permissions rp
           JOIN ancestors a ON a.id = rp.role_id",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?)
}

/// Пары «роль → её родитель» целиком. Таблица ролей маленькая и читается
/// одним запросом, поэтому проверять цикл проще в памяти, чем ещё одним
/// рекурсивным SQL.
pub async fn role_parent_links(pool: &PgPool) -> Result<Vec<(Uuid, Option<Uuid>)>> {
    Ok(sqlx::query_as("SELECT id, parent_id FROM roles")
        .fetch_all(pool)
        .await?)
}

pub async fn user_by_access_token(pool: &PgPool, token: Uuid) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT u.* FROM users u JOIN oauth_sessions s ON s.user_id = u.id
         WHERE s.access_token = $1 AND s.expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_users(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<UserRow>> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?)
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>> {
    Ok(
        sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn set_user_ban(
    pool: &PgPool,
    id: Uuid,
    banned: bool,
    reason: Option<&str>,
) -> Result<()> {
    sqlx::query("UPDATE users SET banned = $2, ban_reason = $3 WHERE id = $1")
        .bind(id)
        .bind(banned)
        .bind(reason)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_username(pool: &PgPool, id: Uuid, username: &str) -> Result<()> {
    sqlx::query("UPDATE users SET mc_username = $2 WHERE id = $1")
        .bind(id)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_skin(pool: &PgPool, id: Uuid, skin_url: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE users SET skin_url = $2 WHERE id = $1")
        .bind(id)
        .bind(skin_url)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_user_role(
    pool: &PgPool,
    user_id: Uuid,
    role_id: Uuid,
    by: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id, granted_by) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(role_id)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_user_role(pool: &PgPool, user_id: Uuid, role_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// `server_id = None` — право действует везде, иначе только на этой сборке.
pub async fn add_user_permission(
    pool: &PgPool,
    user_id: Uuid,
    perm: &str,
    server_id: Option<Uuid>,
    by: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_permissions (user_id, permission, server_id, granted_by) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(perm)
    .bind(server_id)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Снимает право ровно в одном контексте: без `server_id` — только глобальное.
/// Иначе снятие права «на этой сборке» тихо убирало бы и глобальное.
pub async fn remove_user_permission(
    pool: &PgPool,
    user_id: Uuid,
    perm: &str,
    server_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM user_permissions WHERE user_id = $1 AND permission = $2 \
           AND server_id IS NOT DISTINCT FROM $3",
    )
    .bind(user_id)
    .bind(perm)
    .bind(server_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Сессии
// ---------------------------------------------------------------------------

pub struct NewSession {
    pub access_token: Uuid,
    pub refresh_token: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// Выдать лаунчеру одноразовый код, по которому он заберёт токены сессии.
///
/// Живёт минуты: код едет в URL loopback-редиректа, и чем короче окно, тем
/// меньше стоит его перехват.
pub async fn create_launcher_code(
    pool: &PgPool,
    user_id: Uuid,
    session: &NewSession,
) -> Result<Uuid> {
    let code = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO launcher_auth_codes (code, user_id, access_token, refresh_token, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(code)
    .bind(user_id)
    .bind(session.access_token)
    .bind(session.refresh_token)
    .bind(Utc::now() + Duration::minutes(5))
    .execute(pool)
    .await?;
    Ok(code)
}

/// Забрать токены по коду. Код одноразовый: DELETE ... RETURNING не даст двум
/// параллельным запросам получить одну и ту же сессию.
pub async fn take_launcher_code(pool: &PgPool, code: Uuid) -> Result<Option<(Uuid, Uuid, Uuid)>> {
    Ok(sqlx::query_as(
        "DELETE FROM launcher_auth_codes WHERE code = $1 AND expires_at > NOW()
         RETURNING user_id, access_token, refresh_token",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    scope: &str,
    ttl: Duration,
) -> Result<NewSession> {
    let expires_at = Utc::now() + ttl;
    let row: (Uuid, Uuid) = sqlx::query_as(
        "INSERT INTO oauth_sessions (user_id, scope, expires_at) VALUES ($1, $2, $3)
         RETURNING access_token, refresh_token",
    )
    .bind(user_id)
    .bind(scope)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(NewSession {
        access_token: row.0,
        refresh_token: row.1,
        expires_at,
    })
}

pub async fn refresh_session(
    pool: &PgPool,
    refresh_token: Uuid,
    ttl: Duration,
) -> Result<Option<NewSession>> {
    let expires_at = Utc::now() + ttl;
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "UPDATE oauth_sessions SET access_token = gen_random_uuid(), expires_at = $2
         WHERE refresh_token = $1 RETURNING access_token, refresh_token",
    )
    .bind(refresh_token)
    .bind(expires_at)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(a, r)| NewSession {
        access_token: a,
        refresh_token: r,
        expires_at,
    }))
}

pub async fn delete_session(pool: &PgPool, access_token: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM oauth_sessions WHERE access_token = $1")
        .bind(access_token)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Роли (админка)
// ---------------------------------------------------------------------------

pub async fn list_roles(pool: &PgPool) -> Result<Vec<Role>> {
    let rows = sqlx::query_as::<_, RoleRow>("SELECT * FROM roles ORDER BY sort_order DESC")
        .fetch_all(pool)
        .await?;
    let mut out = Vec::new();
    for r in rows {
        out.push(role_with_perms(pool, r).await?);
    }
    Ok(out)
}

pub async fn create_role(
    pool: &PgPool,
    name: &str,
    display_name: &str,
    color: Option<&str>,
    is_default: bool,
    sort_order: i32,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO roles (name, display_name, color, is_default, sort_order)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(name)
    .bind(display_name)
    .bind(color)
    .bind(is_default)
    .bind(sort_order)
    .fetch_one(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_role(
    pool: &PgPool,
    id: Uuid,
    display_name: &str,
    color: Option<&str>,
    is_default: bool,
    sort_order: i32,
    lp_group: Option<&str>,
    icon: Option<&str>,
    parent_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "UPDATE roles SET display_name=$2, color=$3, is_default=$4, sort_order=$5,
         lp_group=$6, icon=$7, parent_id=$8 WHERE id=$1",
    )
    .bind(id)
    .bind(display_name)
    .bind(color)
    .bind(is_default)
    .bind(sort_order)
    .bind(lp_group.filter(|s| !s.trim().is_empty()))
    .bind(icon.filter(|s| !s.trim().is_empty()))
    .bind(parent_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_role(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_role_permission(
    pool: &PgPool,
    role_id: Uuid,
    perm: &str,
    server_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission, server_id) \
         VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(role_id)
    .bind(perm)
    .bind(server_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_role_permission(
    pool: &PgPool,
    role_id: Uuid,
    perm: &str,
    server_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM role_permissions WHERE role_id = $1 AND permission = $2 \
           AND server_id IS NOT DISTINCT FROM $3",
    )
    .bind(role_id)
    .bind(perm)
    .bind(server_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Серверы
// ---------------------------------------------------------------------------

pub async fn list_servers(pool: &PgPool, only_active: bool) -> Result<Vec<ServerRow>> {
    let sql = if only_active {
        "SELECT * FROM servers WHERE active = TRUE ORDER BY sort_order, name"
    } else {
        "SELECT * FROM servers ORDER BY sort_order, name"
    };
    Ok(sqlx::query_as::<_, ServerRow>(sql).fetch_all(pool).await?)
}

pub async fn get_server(pool: &PgPool, id: Uuid) -> Result<Option<ServerRow>> {
    Ok(
        sqlx::query_as::<_, ServerRow>("SELECT * FROM servers WHERE id=$1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Суммарный онлайн сборки.
///
/// Прокси и бэкенды видят одних и тех же игроков, поэтому складывать всё
/// подряд нельзя — получилось бы вдвое. Считают бэкенды; прокси идут в дело,
/// только если бэкендов не завели вовсе.
///
/// `None` — ни один агент не на связи: это «неизвестно», а не «ноль игроков».
fn sum_online(servers: &[schema::GameServerEntry]) -> (Option<u32>, Option<u32>) {
    let backends: Vec<_> = servers.iter().filter(|g| g.live && !g.proxy).collect();
    let counted = if backends.is_empty() {
        servers.iter().filter(|g| g.live).collect()
    } else {
        backends
    };
    if counted.is_empty() {
        return (None, None);
    }
    (
        Some(counted.iter().map(|g| g.online).sum()),
        Some(counted.iter().map(|g| g.max_online).sum()),
    )
}

/// Все сборки сервера, свежие сверху: и опубликованные, и черновики.
///
/// Отбор по правам делается выше — здесь не хватает данных о пользователе.
pub async fn list_server_builds(
    pool: &PgPool,
    server_id: Uuid,
) -> Result<Vec<schema::BuildOption>> {
    let rows: Vec<(Uuid, String, bool)> = sqlx::query_as(
        "SELECT id, version, published FROM builds WHERE server_id = $1 ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, version, published)| schema::BuildOption {
            id,
            version,
            published,
        })
        .collect())
}

/// Клиентский загрузчик сервера.
///
/// В `servers.modloader` попадает и серверное ПО: агент и враппер знают `paper`,
/// `spigot`, `velocity` и остальных. Клиент у них ванильный — это факт
/// предметной области, а не подстановка на всякий случай. Всё прочее, чего мы не
/// знаем, — ошибка данных: раньше любая опечатка тихо становилась `vanilla`, и
/// сборка с Fabric уезжала игроку без загрузчика.
pub fn client_modloader(raw: &str) -> Result<Modloader> {
    match raw.to_ascii_lowercase().as_str() {
        "paper" | "bukkit" | "spigot" | "purpur" | "folia" | "velocity" | "bungeecord" => {
            Ok(Modloader::Vanilla)
        }
        other => Modloader::from_str(other).map_err(anyhow::Error::msg),
    }
}

/// Сервер + актуальная опубликованная сборка → ServerEntry для лаунчера.
pub async fn server_entry(pool: &PgPool, s: &ServerRow) -> Result<ServerEntry> {
    let build: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, version FROM builds WHERE server_id = $1 AND published = TRUE
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(s.id)
    .fetch_optional(pool)
    .await?;

    let game_servers: Vec<_> = crate::db::list_game_servers(pool, s.id)
        .await?
        .iter()
        .map(|g| g.to_entry())
        .collect();
    let (online, max_online) = sum_online(&game_servers);
    // Точка входа: прокси, если он заведён, иначе первый сервер по порядку.
    // Своего адреса у сборки нет — он дублировал бы этот.
    let entry_point = game_servers
        .iter()
        .find(|g| g.proxy)
        .or_else(|| game_servers.first());

    Ok(ServerEntry {
        id: s.id,
        name: s.name.clone(),
        description: s.description.clone(),
        icon_url: s.icon_url.clone(),
        background_url: s.background_url.clone(),
        mc_host: entry_point.map(|g| g.mc_host.clone()),
        mc_port: entry_point.map(|g| g.mc_port),
        modloader: client_modloader(&s.modloader).with_context(|| format!("сервер {}", s.name))?,
        mc_version: s.mc_version.clone(),
        current_build_id: build.as_ref().map(|b| b.0),
        current_version: build.map(|b| b.1),
        limited: s.limited,
        sort_order: s.sort_order,
        game_servers,
        available_builds: list_server_builds(pool, s.id).await?,
        online,
        max_online,
    })
}

pub async fn delete_server(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM servers WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Сборки
// ---------------------------------------------------------------------------

pub async fn list_builds(pool: &PgPool, server_id: Uuid) -> Result<Vec<BuildRow>> {
    Ok(sqlx::query_as::<_, BuildRow>(
        "SELECT * FROM builds WHERE server_id=$1 ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_build(pool: &PgPool, id: Uuid) -> Result<Option<BuildRow>> {
    Ok(
        sqlx::query_as::<_, BuildRow>("SELECT * FROM builds WHERE id=$1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn latest_published_build(pool: &PgPool, server_id: Uuid) -> Result<Option<BuildRow>> {
    Ok(sqlx::query_as::<_, BuildRow>(
        "SELECT * FROM builds WHERE server_id=$1 AND published=TRUE ORDER BY created_at DESC LIMIT 1",
    )
    .bind(server_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_build(
    pool: &PgPool,
    server_id: Uuid,
    version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
    mc_version: &str,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO builds (server_id, version, modloader, modloader_version, mc_version)
         VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(server_id)
    .bind(version)
    .bind(modloader)
    .bind(modloader_version)
    .bind(mc_version)
    .fetch_one(pool)
    .await?)
}

pub async fn set_build_published(pool: &PgPool, id: Uuid, published: bool) -> Result<()> {
    sqlx::query("UPDATE builds SET published=$2 WHERE id=$1")
        .bind(id)
        .bind(published)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_build(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM builds WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Обновить вычисленные при bootstrap поля манифеста.
#[allow(clippy::too_many_arguments)]
pub async fn update_build_manifest_meta(
    pool: &PgPool,
    id: Uuid,
    main_class: &str,
    jvm_args: &serde_json::Value,
    game_args: &serde_json::Value,
    assets_index_name: &str,
    signature: &[u8],
) -> Result<()> {
    sqlx::query(
        "UPDATE builds SET main_class=$2, jvm_args=$3, game_args=$4, assets_index_name=$5,
         manifest_signature=$6 WHERE id=$1",
    )
    .bind(id)
    .bind(main_class)
    .bind(jvm_args)
    .bind(game_args)
    .bind(assets_index_name)
    .bind(signature)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn build_files(pool: &PgPool, build_id: Uuid) -> Result<Vec<BuildFileRow>> {
    Ok(sqlx::query_as::<_, BuildFileRow>(
        "SELECT * FROM build_files WHERE build_id=$1 ORDER BY path",
    )
    .bind(build_id)
    .fetch_all(pool)
    .await?)
}

pub async fn upsert_build_file(
    pool: &PgPool,
    build_id: Uuid,
    path: &str,
    sha1: &str,
    size: i64,
    side: &str,
    kind: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO build_files (build_id, path, sha1, size, side, kind)
         VALUES ($1,$2,$3,$4,$5,$6)
         ON CONFLICT (build_id, path) DO UPDATE SET sha1=$3, size=$4, side=$5, kind=$6",
    )
    .bind(build_id)
    .bind(path)
    .bind(sha1)
    .bind(size)
    .bind(side)
    .bind(kind)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_build_file(pool: &PgPool, file_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM build_files WHERE id=$1")
        .bind(file_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mojang-артефакты (кеш)
// ---------------------------------------------------------------------------

pub async fn artifact_by_path(pool: &PgPool, path: &str) -> Result<Option<MojangArtifactRow>> {
    Ok(
        sqlx::query_as::<_, MojangArtifactRow>("SELECT * FROM mojang_artifacts WHERE path=$1")
            .bind(path)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn upsert_artifact(
    pool: &PgPool,
    artifact_type: &str,
    mc_version: Option<&str>,
    platform: Option<&str>,
    path: &str,
    sha1: &str,
    size: i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO mojang_artifacts (artifact_type, mc_version, platform, path, sha1, size)
         VALUES ($1,$2,$3,$4,$5,$6)
         ON CONFLICT (path) DO UPDATE SET sha1=$5, size=$6",
    )
    .bind(artifact_type)
    .bind(mc_version)
    .bind(platform)
    .bind(path)
    .bind(sha1)
    .bind(size)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Base Builds (Глобальный кеш ванильных/loader файлов)
// ---------------------------------------------------------------------------

pub async fn get_base_build(
    pool: &PgPool,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
) -> Result<Option<BaseBuildRow>> {
    Ok(sqlx::query_as::<_, BaseBuildRow>(
        "SELECT * FROM base_builds WHERE mc_version=$1 AND modloader=$2 AND modloader_version IS NOT DISTINCT FROM $3"
    )
    .bind(mc_version)
    .bind(modloader)
    .bind(modloader_version)
    .fetch_optional(pool)
    .await?)
}

pub async fn upsert_base_build(
    pool: &PgPool,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
    main_class: &str,
    jvm_args: &serde_json::Value,
    game_args: &serde_json::Value,
    assets_index_name: &str,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO base_builds (mc_version, modloader, modloader_version, main_class, jvm_args, game_args, assets_index_name)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT (mc_version, modloader, modloader_version) DO UPDATE SET
            main_class=$4, jvm_args=$5, game_args=$6, assets_index_name=$7
         RETURNING id"
    )
    .bind(mc_version)
    .bind(modloader)
    .bind(modloader_version)
    .bind(main_class)
    .bind(jvm_args)
    .bind(game_args)
    .bind(assets_index_name)
    .fetch_one(pool)
    .await?)
}

pub async fn upsert_base_build_file(
    pool: &PgPool,
    base_build_id: Uuid,
    path: &str,
    sha1: &str,
    size: i64,
    side: &str,
    kind: &str,
    platform: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO base_build_files (base_build_id, path, sha1, size, side, kind, platform)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT (base_build_id, path)
         DO UPDATE SET sha1=$3, size=$4, side=$5, kind=$6, platform=$7",
    )
    .bind(base_build_id)
    .bind(path)
    .bind(sha1)
    .bind(size)
    .bind(side)
    .bind(kind)
    .bind(platform)
    .execute(pool)
    .await?;
    Ok(())
}

/// Сборка разложена до мультиплатформенности?
///
/// Признак — java-файлы без платформы: раньше рантайм клали под ОС мастера и
/// путь платформы не содержал, так что Windows получал чужой JRE.
pub async fn has_legacy_java(pool: &PgPool, base_build_id: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM base_build_files
          WHERE base_build_id = $1 AND kind = 'java' AND platform IS NULL)",
    )
    .bind(base_build_id)
    .fetch_one(pool)
    .await?)
}

/// Снести base build целиком. `base_build_files` уходят каскадом, блобы в сторе
/// остаются: они адресуются по содержимому и делятся между сборками.
pub async fn delete_base_build(
    pool: &PgPool,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
) -> Result<u64> {
    Ok(sqlx::query(
        "DELETE FROM base_builds
          WHERE mc_version = $1 AND modloader = $2
            AND modloader_version IS NOT DISTINCT FROM $3",
    )
    .bind(mc_version)
    .bind(modloader)
    .bind(modloader_version)
    .execute(pool)
    .await?
    .rows_affected())
}

/// Убрать файлы вида: пути java сменились, и старые записи иначе остались бы
/// висеть — клиент считал бы их нужными всем.
pub async fn delete_base_build_files_kind(
    pool: &PgPool,
    base_build_id: Uuid,
    kind: &str,
) -> Result<u64> {
    Ok(
        sqlx::query("DELETE FROM base_build_files WHERE base_build_id = $1 AND kind = $2")
            .bind(base_build_id)
            .bind(kind)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

pub async fn base_build_files(pool: &PgPool, base_build_id: Uuid) -> Result<Vec<BaseBuildFileRow>> {
    Ok(sqlx::query_as::<_, BaseBuildFileRow>(
        "SELECT * FROM base_build_files WHERE base_build_id=$1 ORDER BY path",
    )
    .bind(base_build_id)
    .fetch_all(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Ядра серверов
// ---------------------------------------------------------------------------

pub async fn list_server_cores(
    pool: &PgPool,
    server_id: Option<Uuid>,
) -> Result<Vec<ServerCoreRow>> {
    if let Some(server_id) = server_id {
        Ok(sqlx::query_as::<_, ServerCoreRow>(
            "SELECT * FROM server_cores WHERE server_id=$1 ORDER BY uploaded_at DESC",
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?)
    } else {
        Ok(sqlx::query_as::<_, ServerCoreRow>(
            "SELECT * FROM server_cores ORDER BY uploaded_at DESC",
        )
        .fetch_all(pool)
        .await?)
    }
}

pub async fn insert_server_core(
    pool: &PgPool,
    server_id: Uuid,
    version: &str,
    sha256: &str,
    file_sha1: &str,
    size: i64,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO server_cores (server_id, version, sha256, file_sha1, size)
         VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(server_id)
    .bind(version)
    .bind(sha256)
    .bind(file_sha1)
    .bind(size)
    .fetch_one(pool)
    .await?)
}

pub async fn activate_server_core(pool: &PgPool, id: Uuid) -> Result<Option<ServerCoreRow>> {
    let row = sqlx::query_as::<_, ServerCoreRow>("SELECT * FROM server_cores WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    if let Some(ref core) = row {
        sqlx::query("UPDATE server_cores SET active=FALSE WHERE server_id=$1")
            .bind(core.server_id)
            .execute(pool)
            .await?;
        sqlx::query("UPDATE server_cores SET active=TRUE WHERE id=$1")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(row)
}

pub async fn delete_server_core(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM server_cores WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Новости
// ---------------------------------------------------------------------------

pub async fn list_news(pool: &PgPool, limit: i64) -> Result<Vec<NewsRow>> {
    Ok(sqlx::query_as::<_, NewsRow>(
        "SELECT * FROM news ORDER BY pinned DESC, published_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn create_news(
    pool: &PgPool,
    title: &str,
    body: &str,
    preview: Option<&str>,
    author: Option<Uuid>,
    pinned: bool,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO news (title, body, preview_img_url, author_id, pinned)
         VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(title)
    .bind(body)
    .bind(preview)
    .bind(author)
    .bind(pinned)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_news(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM news WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Версии лаунчера
// ---------------------------------------------------------------------------

pub async fn list_launcher_versions(pool: &PgPool) -> Result<Vec<LauncherVersionRow>> {
    Ok(sqlx::query_as::<_, LauncherVersionRow>(
        "SELECT * FROM launcher_versions ORDER BY built_at DESC",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn current_launcher_version(
    pool: &PgPool,
    platform: &str,
) -> Result<Option<LauncherVersionRow>> {
    Ok(sqlx::query_as::<_, LauncherVersionRow>(
        "SELECT * FROM launcher_versions
         WHERE platform=$1 AND kind='core' AND is_current=TRUE LIMIT 1",
    )
    .bind(platform)
    .fetch_optional(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_launcher_version(
    pool: &PgPool,
    version: &str,
    platform: &str,
    sha256: &str,
    file_sha1: &str,
    size: i64,
    signature: &str,
    kind: &str,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO launcher_versions (version, platform, sha256, file_sha1, size, signature, kind)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT (version, platform, kind)
         DO UPDATE SET sha256=$3, file_sha1=$4, size=$5, signature=$6
         RETURNING id",
    )
    .bind(version)
    .bind(platform)
    .bind(sha256)
    .bind(file_sha1)
    .bind(size)
    .bind(signature)
    .bind(kind)
    .fetch_one(pool)
    .await?)
}

pub async fn set_current_launcher_version(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<LauncherVersionRow>> {
    let row =
        sqlx::query_as::<_, LauncherVersionRow>("SELECT * FROM launcher_versions WHERE id=$1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    if let Some(ref r) = row {
        // Гасим только ту же разновидность на той же платформе. Core и
        // установщик переезжают порознь намеренно: репутация SmartScreen
        // привязана к хешу .exe, и подменять установщик ради обновления, которое
        // целиком живёт в core, значит обнулять её на ровном месте.
        //
        // Гашение по одной платформе целиком (как было раньше) утаскивало за
        // собой установщик и убирало кнопку скачивания с сайта — отсюда парный
        // подъём, который и лишал выбора.
        sqlx::query("UPDATE launcher_versions SET is_current=FALSE WHERE platform=$1 AND kind=$2")
            .bind(&r.platform)
            .bind(&r.kind)
            .execute(pool)
            .await?;
        sqlx::query("UPDATE launcher_versions SET is_current=TRUE WHERE id=$1")
            .bind(r.id)
            .execute(pool)
            .await?;
    }
    Ok(row)
}

// ---------------------------------------------------------------------------
// Admin-токены
// ---------------------------------------------------------------------------

/// Найти токен по селектору. Владение секретом проверяется отдельно —
/// `last_used_at` здесь не трогается, иначе отметка ставилась бы и на неудачные
/// попытки.
pub async fn admin_token_by_lookup(pool: &PgPool, lookup: &str) -> Result<Option<AdminTokenRow>> {
    Ok(
        sqlx::query_as::<_, AdminTokenRow>("SELECT * FROM admin_tokens WHERE token_lookup=$1")
            .bind(lookup)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn touch_admin_token(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE admin_tokens SET last_used_at=NOW() WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Досчитать argon2 для токена, доставшегося от старой схемы.
pub async fn upgrade_admin_token_hash(pool: &PgPool, id: Uuid, phc: &str) -> Result<()> {
    sqlx::query("UPDATE admin_tokens SET token_hash=$2 WHERE id=$1 AND token_hash IS NULL")
        .bind(id)
        .bind(phc)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_admin_tokens(pool: &PgPool) -> Result<Vec<AdminTokenRow>> {
    Ok(
        sqlx::query_as::<_, AdminTokenRow>("SELECT * FROM admin_tokens ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn create_admin_token(
    pool: &PgPool,
    name: &str,
    lookup: &str,
    token_hash: &str,
    permissions: &[String],
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO admin_tokens (name, token_lookup, token_hash, permissions)
         VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(name)
    .bind(lookup)
    .bind(token_hash)
    .bind(permissions)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_admin_token(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM admin_tokens WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Игровые сессии / статистика
// ---------------------------------------------------------------------------

pub async fn record_play_start(pool: &PgPool, user_id: Uuid, server_id: Uuid) -> Result<()> {
    sqlx::query("INSERT INTO play_sessions (user_id, server_id) VALUES ($1,$2)")
        .bind(user_id)
        .bind(server_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn record_play_stop(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Uuid,
    secs: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE play_sessions SET playtime_secs=$3
         WHERE id = (SELECT id FROM play_sessions WHERE user_id=$1 AND server_id=$2
                     ORDER BY started_at DESC LIMIT 1)",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(secs)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct Stats {
    pub users: i64,
    pub servers: i64,
    pub builds: i64,
    pub total_playtime_secs: i64,
}

pub async fn stats(pool: &PgPool) -> Result<Stats> {
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let servers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM servers")
        .fetch_one(pool)
        .await?;
    let builds: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM builds")
        .fetch_one(pool)
        .await?;
    let total_playtime_secs: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(playtime_secs), 0)::BIGINT FROM play_sessions")
            .fetch_one(pool)
            .await?;
    Ok(Stats {
        users,
        servers,
        builds,
        total_playtime_secs,
    })
}

// ── Переводы ────────────────────────────────────────────────────────────────

/// Каталог языка: (ftl, sha1). `None`, если для языка ничего не залито.
pub async fn get_translation(
    pool: &PgPool,
    locale: &str,
) -> sqlx::Result<Option<(String, String)>> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT ftl, sha1 FROM translations WHERE locale = $1")
            .bind(locale)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

/// Список залитых языков с их хешами — лаунчер по нему решает, качать ли.
pub async fn list_translations(pool: &PgPool) -> sqlx::Result<Vec<(String, String)>> {
    sqlx::query_as("SELECT locale, sha1 FROM translations ORDER BY locale")
        .fetch_all(pool)
        .await
}

pub async fn upsert_translation(
    pool: &PgPool,
    locale: &str,
    ftl: &str,
    sha1: &str,
    by: Option<uuid::Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO translations (locale, ftl, sha1, updated_by, updated_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (locale) DO UPDATE
         SET ftl = $2, sha1 = $3, updated_by = $4, updated_at = NOW()",
    )
    .bind(locale)
    .bind(ftl)
    .bind(sha1)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_translation(pool: &PgPool, locale: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM translations WHERE locale = $1")
        .bind(locale)
        .execute(pool)
        .await?;
    Ok(())
}

/// Пользователь по MC UUID — как его знает игровой сервер.
pub async fn user_by_mc_uuid(pool: &PgPool, mc_uuid: Uuid) -> Result<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE mc_uuid = $1")
        .bind(mc_uuid)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// Актуальные установщики по платформам — то, что раздаётся с сайта.
pub async fn current_bootstrappers(pool: &PgPool) -> Result<Vec<LauncherVersionRow>> {
    Ok(sqlx::query_as::<_, LauncherVersionRow>(
        "SELECT * FROM launcher_versions
         WHERE kind='bootstrapper' AND is_current=TRUE
         ORDER BY platform",
    )
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod client_modloader_tests {
    use super::*;

    /// В `servers.modloader` живут и серверные платформы. Строгий разбор ломал
    /// список серверов у живого `paper`-сервера — эта проверка держит границу
    /// между «серверное ПО с ванильным клиентом» и «мы не знаем, что это».
    #[test]
    fn server_software_means_a_vanilla_client() {
        for raw in ["paper", "Spigot", "purpur", "velocity", "bungeecord"] {
            assert_eq!(client_modloader(raw).unwrap(), Modloader::Vanilla, "{raw}");
        }
    }

    #[test]
    fn client_loaders_pass_through() {
        assert_eq!(client_modloader("fabric").unwrap(), Modloader::Fabric);
        assert_eq!(client_modloader("neoforge").unwrap(), Modloader::NeoForge);
        assert_eq!(client_modloader("vanilla").unwrap(), Modloader::Vanilla);
    }

    /// Опечатка обязана быть ошибкой, а не молчаливой ваниллой: сборка с Fabric
    /// уехала бы игроку без загрузчика, и игра упала бы у него, а не у нас.
    #[test]
    fn an_unknown_value_is_an_error() {
        assert!(client_modloader("fabrci").is_err());
        assert!(client_modloader("").is_err());
    }
}
