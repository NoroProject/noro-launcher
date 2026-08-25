//! Админ: управление пользователями.

use crate::api::auth::AdminAuth;
use crate::api::paging::{Page, PageQuery};
use crate::audit::{self, target};
use crate::db::identities::UnlinkResult;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use schema::{
    UserProfile, PERM_PUNISH_BAN, PERM_USERS_CAPES, PERM_USERS_EDIT, PERM_USERS_PERMISSIONS,
    PERM_USERS_ROLES, PERM_USERS_SKIN, PERM_USERS_VIEW,
};

use serde::Deserialize;
use uuid::Uuid;

/// `GET /api/admin/users` — страница игроков.
///
/// `q` ищет по нику, Discord и обоим идентификаторам. Без поиска в базе клиенту
/// оставалось бы тянуть список целиком и фильтровать у себя — что спотлайт и
/// делал, находя только тех, кто попал в первую страницу.
#[derive(Deserialize)]
pub struct UsersQuery {
    // Те же три поля, что у `PageQuery`, объявлены здесь, а не подмешаны
    // флаттеном: так serde разбирает их напрямую, без буферизации.
    pub q: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::api::paging::flexible_i64::deserialize"
    )]
    pub limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "crate::api::paging::flexible_i64::deserialize"
    )]
    pub offset: Option<i64>,
    /// Только забаненные или только активные. Фильтр серверный: на клиенте он
    /// отсеивал бы лишь текущую страницу и врал о числе найденного.
    pub banned: Option<bool>,
    pub role: Option<String>,
}

impl UsersQuery {
    fn paging(&self) -> PageQuery {
        PageQuery::from_parts(self.q.clone(), self.limit, self.offset)
    }
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<UsersQuery>,
) -> AppResult<Json<Page<UserProfile>>> {
    admin.require(PERM_USERS_VIEW)?;
    let page = q.paging();
    let like = page.like();
    let filter = crate::db::UserFilter {
        like: like.as_deref(),
        banned: q.banned,
        role: q.role.as_deref(),
    };
    let (rows, total) =
        crate::db::list_users(&state.db, &filter, page.limit(), page.offset()).await?;
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        items.push(crate::db::profile_from_row(&state.db, r).await?);
    }
    Ok(Json(Page::new(items, total)))
}

pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_VIEW)?;
    let row = crate::db::get_user(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("user".into()))?;
    Ok(Json(crate::db::profile_from_row(&state.db, row).await?))
}

#[derive(Deserialize)]
pub struct LookupIdentityQuery {
    pub provider: String,
    pub provider_user_id: String,
}

/// `GET /api/admin/users/by-identity?provider=telegram&provider_user_id=730545443`
///
/// Найти профиль игрока по привязанной платформе и её ID.
pub async fn get_by_identity(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<LookupIdentityQuery>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_VIEW)?;
    let user_id = crate::db::identities::find_user(&state.db, &q.provider, &q.provider_user_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "user with identity {}/{} not found",
                q.provider, q.provider_user_id
            ))
        })?;
    let profile = crate::db::load_profile(&state.db, user_id).await?;
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct BanReq {
    pub banned: bool,
    pub reason: Option<String>,
}

pub async fn ban(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<BanReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_PUNISH_BAN)?;
    // Инстанс, оставшийся без операторского входа, чинится только руками в БД.
    if req.banned && crate::db::is_root_user(&state.db, id).await? {
        return Err(AppError::Forbidden("root cannot be banned".into()));
    }
    crate::db::set_user_ban(&state.db, id, req.banned, req.reason.as_deref()).await?;
    audit::record(
        &state,
        &admin.actor,
        if req.banned {
            crate::audit::actions::USER_BAN
        } else {
            crate::audit::actions::USER_UNBAN
        },
        target("user", id),
        serde_json::json!({ "reason": req.reason }),
    )
    .await;
    let profile = crate::db::load_profile(&state.db, id).await?;
    state.ws.send_to_user(
        id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}

/// `DELETE /api/admin/users/{id}/identities/{provider}` — снять привязку.
///
/// Нужна, когда игрок потерял доступ к платформе и сам отвязать её уже не
/// может. Платформу регистрации не снимаем и здесь: из неё выведен mc_uuid, и
/// без неё аккаунт остаётся с UUID, который ни на что не ссылается.
pub async fn unlink_identity(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, provider)): Path<(Uuid, String)>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_EDIT)?;
    let provider = crate::api::auth::oauth::flow::parse_provider(&provider)?;
    match crate::db::identities::unlink(&state.db, id, provider.slug()).await? {
        UnlinkResult::Removed => {}
        UnlinkResult::NotLinked => {
            return Err(AppError::NotFound(format!(
                "{} is not linked to this account",
                provider.display_name()
            )))
        }
        UnlinkResult::Primary => {
            return Err(AppError::BadRequest(format!(
                "{} is the platform this account was created with — it cannot be unlinked",
                provider.display_name()
            )))
        }
    }

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::USER_IDENTITY_UNLINK,
        target("user", id),
        serde_json::json!({ "provider": provider.slug() }),
    )
    .await;
    Ok(Json(crate::db::load_profile(&state.db, id).await?))
}

pub async fn add_role(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_ROLES)?;
    crate::db::add_user_role(&state.db, id, role_id, admin.user_id()).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::USER_ROLE_ADD,
        target("user", id),
        serde_json::json!({ "role_id": role_id }),
    )
    .await;
    notify_user(&state, id).await
}

pub async fn remove_role(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_ROLES)?;
    crate::db::remove_user_role(&state.db, id, role_id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::USER_ROLE_REMOVE,
        target("user", id),
        serde_json::json!({ "role_id": role_id }),
    )
    .await;
    notify_user(&state, id).await
}
#[derive(Deserialize)]
pub struct PermReq {
    pub permission: String,
    /// Контекст: `None` — везде, иначе только на этой сборке.
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

/// Тот же контекст, но в query — у DELETE тела нет.
#[derive(Deserialize)]
pub struct PermScope {
    #[serde(default)]
    pub server_id: Option<Uuid>,
}
pub async fn add_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<PermReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_PERMISSIONS)?;
    crate::db::add_user_permission(
        &state.db,
        id,
        &req.permission,
        req.server_id,
        admin.user_id(),
    )
    .await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::USER_PERM_ADD,
        target("user", id),
        serde_json::json!({ "permission": req.permission, "server_id": req.server_id }),
    )
    .await;
    notify_user(&state, id).await
}

pub async fn remove_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, perm)): Path<(Uuid, String)>,
    Query(scope): Query<PermScope>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_PERMISSIONS)?;
    crate::db::remove_user_permission(&state.db, id, &perm, scope.server_id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::USER_PERM_REMOVE,
        target("user", id),
        serde_json::json!({ "permission": perm, "server_id": scope.server_id }),
    )
    .await;
    notify_user(&state, id).await
}
#[derive(Deserialize)]
pub struct CapeReq {
    pub cape_id: Option<Uuid>,
}

pub async fn set_cape(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CapeReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_CAPES)?;
    let cape_url = match req.cape_id {
        Some(cape_id) => Some(
            crate::db::get_cape_url(&state.db, cape_id)
                .await?
                .ok_or_else(|| AppError::NotFound("cape".into()))?,
        ),
        None => None,
    };
    crate::db::set_user_cape(&state.db, id, cape_url.as_deref()).await?;
    notify_user(&state, id).await
}

pub async fn get_capes(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<schema::UserCapesData>> {
    admin.require(PERM_USERS_CAPES)?;
    let granted_cape_ids = crate::db::list_user_granted_cape_ids(&state.db, id).await?;
    Ok(Json(schema::UserCapesData { granted_cape_ids }))
}

pub async fn set_granted_capes(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<schema::SetUserCapesReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_CAPES)?;
    crate::db::set_user_granted_capes(&state.db, id, &req.granted_cape_ids).await?;
    let active_cape_url = match req.active_cape_id {
        Some(cape_id) => crate::db::get_cape_url(&state.db, cape_id).await?,
        None => None,
    };
    crate::db::set_user_cape(&state.db, id, active_cape_url.as_deref()).await?;
    notify_user(&state, id).await
}

pub async fn upload_skin_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_SKIN)?;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("skin") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
                return Err(AppError::bad(
                    crate::error_codes::UPLOAD_BAD_FORMAT,
                    "PNG expected",
                ));
            }
            if data.len() > 256 * 1024 {
                return Err(AppError::bad(
                    crate::error_codes::UPLOAD_TOO_LARGE,
                    "skin is too large",
                ));
            }
            let stored = state
                .files
                .put_bytes(&data)
                .await
                .map_err(AppError::Other)?;
            let url = state.config.file_url(&stored.sha1);
            let slim = crate::api::skin_model::detect_slim(&data);
            crate::db::set_skin(&state.db, id, Some(&url), slim).await?;
            let user_name = crate::db::load_profile(&state.db, id)
                .await
                .map(|p| p.username)
                .unwrap_or_else(|_| "Preset".into());
            let _ = sqlx::query(
                "INSERT INTO user_skin_presets (user_id, name, skin_url, skin_slim) VALUES ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(user_name)
            .bind(&url)
            .bind(slim)
            .execute(&state.db)
            .await;
            return notify_user(&state, id).await;
        }
    }
    Err(AppError::bad(
        crate::error_codes::UPLOAD_FIELD_MISSING,
        "missing skin field",
    ))
}

#[derive(Deserialize)]
pub struct AddSkinPresetReq {
    pub name: Option<String>,
    pub skin_url: Option<String>,
    #[serde(default)]
    pub slim: Option<bool>,
}

pub async fn add_skin_preset_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<AddSkinPresetReq>,
) -> AppResult<Json<SkinPresetItem>> {
    admin.require(PERM_USERS_SKIN)?;
    let profile = crate::db::load_profile(&state.db, id).await?;
    let skin_url = match req.skin_url {
        Some(u) => u,
        None => profile
            .skin_url
            .ok_or_else(|| AppError::BadRequest("User has no current skin".into()))?,
    };
    let slim = req.slim.unwrap_or(profile.skin_slim);
    let name = req.name.unwrap_or(profile.username);

    let row = sqlx::query_as::<_, SkinPresetItem>(
        "INSERT INTO user_skin_presets (user_id, name, skin_url, skin_slim)
         VALUES ($1, $2, $3, $4) RETURNING id, name, skin_url",
    )
    .bind(id)
    .bind(name)
    .bind(skin_url)
    .bind(slim)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(row))
}

pub async fn delete_skin_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_SKIN)?;
    crate::db::set_skin(&state.db, id, None, false).await?;
    notify_user(&state, id).await
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SkinPresetItem {
    pub id: Uuid,
    pub name: String,
    pub skin_url: String,
}

pub async fn list_skin_presets_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Page<SkinPresetItem>>> {
    admin.require(PERM_USERS_SKIN)?;
    let rows = sqlx::query_as::<_, SkinPresetItem>(
        "SELECT id, name, skin_url FROM user_skin_presets WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(Page::whole(rows)))
}

#[derive(Deserialize)]
pub struct SelectSkinPresetReq {
    pub skin_url: String,
    /// Тонкая модель (Алекс). По умолчанию классическая.
    #[serde(default)]
    pub slim: bool,
}

pub async fn select_skin_preset_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<SelectSkinPresetReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_USERS_SKIN)?;
    crate::db::set_skin(&state.db, id, Some(&req.skin_url), req.slim).await?;
    notify_user(&state, id).await
}

pub async fn delete_skin_preset_for_user(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, preset_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<()>> {
    admin.require(PERM_USERS_SKIN)?;
    sqlx::query("DELETE FROM user_skin_presets WHERE id = $1 AND user_id = $2")
        .bind(preset_id)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(()))
}

async fn notify_user(state: &AppState, id: Uuid) -> AppResult<Json<UserProfile>> {
    let profile = crate::db::load_profile(&state.db, id).await?;
    state.ws.send_to_user(
        id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    // И в игру. Иначе выданная здесь роль не значила бы там ничего до тех пор,
    // пока игрок не перезайдёт, — а он не знает, что должен.
    crate::agent_link::notify::profile_changed(state, Some(profile.uuid));
    Ok(Json(profile))
}
