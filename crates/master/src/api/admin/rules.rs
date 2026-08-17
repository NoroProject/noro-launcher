//! Админ: свод правил — разделы, правила и их порядок.

use crate::api::auth::AdminAuth;
use crate::db::rules::{
    CategoryInput, CategoryTranslationRow, RuleCategoryRow, RuleInput, RuleRow, RuleSanctionRow,
    RuleTranslationRow, SanctionInput, TranslationInput,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::{PERM_RULES_DELETE, PERM_RULES_EDIT, PERM_RULES_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Виды наказаний, которые можно заложить в правило. Тот же список, что и у
/// формы модерации: вариант должен подставляться в неё без перевода.
const KINDS: [&str; 4] = ["warn", "mute", "ban", "server_ban"];

#[derive(Deserialize)]
pub struct CategoryReq {
    #[serde(default)]
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub server_id: Option<Uuid>,
    /// Переводы названия и вступления по локалям.
    #[serde(default)]
    pub translations: Vec<TranslationInput>,
}

#[derive(Deserialize)]
pub struct RuleReq {
    pub code: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Формулировка, которую игрок увидит в наказании по этому пункту.
    #[serde(default)]
    pub punish_reason: String,
    #[serde(default)]
    pub category_id: Option<Uuid>,
    #[serde(default)]
    pub server_id: Option<Uuid>,
    /// Допустимые наказания. Пустой список — правило ничего не предписывает,
    /// и модератор выбирает срок сам.
    #[serde(default)]
    pub sanctions: Vec<SanctionInput>,
    /// Переводы заголовка и текста правила по локалям.
    #[serde(default)]
    pub translations: Vec<TranslationInput>,
}

#[derive(Deserialize)]
pub struct ReorderReq {
    pub order: Vec<Uuid>,
}

pub async fn list_categories(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<RuleCategoryRow>>> {
    admin.require(PERM_RULES_VIEW)?;
    Ok(Json(crate::db::list_all_rule_categories(&state.db).await?))
}

pub async fn create_category(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CategoryReq>,
) -> AppResult<Json<RuleCategoryRow>> {
    admin.require(PERM_RULES_EDIT)?;
    let name = required(&req.name, "name")?;
    let saved = crate::db::create_rule_category(
        &state.db,
        CategoryInput {
            code: req.code.trim(),
            name,
            description: req.description.trim(),
            parent_id: req.parent_id,
            server_id: req.server_id,
        },
    )
    .await?;
    crate::db::replace_category_translations(&state.db, saved.id, &req.translations).await?;
    Ok(Json(saved))
}

pub async fn update_category(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CategoryReq>,
) -> AppResult<Json<RuleCategoryRow>> {
    admin.require(PERM_RULES_EDIT)?;
    let name = required(&req.name, "name")?;
    if let Some(parent) = req.parent_id {
        if parent == id || crate::db::category_has_ancestor(&state.db, id, parent).await? {
            return Err(AppError::BadRequest(
                "a section cannot be nested into itself".into(),
            ));
        }
    }
    let saved = crate::db::update_rule_category(
        &state.db,
        id,
        CategoryInput {
            code: req.code.trim(),
            name,
            description: req.description.trim(),
            parent_id: req.parent_id,
            server_id: req.server_id,
        },
    )
    .await?;
    crate::db::replace_category_translations(&state.db, saved.id, &req.translations).await?;
    Ok(Json(saved))
}

/// GET /api/admin/rules/categories/{id}/translations — языки одного раздела.
pub async fn category_translations(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<CategoryTranslationRow>>> {
    admin.require(PERM_RULES_VIEW)?;
    Ok(Json(
        crate::db::translations_of_category(&state.db, id).await?,
    ))
}

/// Удаление раздела не трогает правила: они всплывают в «без раздела» и видны
/// на той же странице. Молча исчезнувшее правило админ заметил бы позже всех.
pub async fn delete_category(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_RULES_DELETE)?;
    crate::db::delete_rule_category(&state.db, id).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn reorder_categories(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<ReorderReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_RULES_EDIT)?;
    crate::db::reorder_rule_categories(&state.db, &req.order).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(serde::Serialize)]
pub struct AdminRules {
    pub rules: Vec<RuleRow>,
    pub sanctions: Vec<RuleSanctionRow>,
}

pub async fn list_rules(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<AdminRules>> {
    admin.require(PERM_RULES_VIEW)?;
    Ok(Json(AdminRules {
        rules: crate::db::list_all_rules(&state.db).await?,
        sanctions: crate::db::list_all_rule_sanctions(&state.db).await?,
    }))
}

pub async fn create_rule(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<RuleReq>,
) -> AppResult<Json<RuleRow>> {
    admin.require(PERM_RULES_EDIT)?;
    let input = rule_input(&req)?;
    let rule = crate::db::create_rule(&state.db, input)
        .await
        .map_err(|e| code_conflict(e, &req.code))?;
    crate::db::replace_rule_sanctions(&state.db, rule.id, &req.sanctions).await?;
    crate::db::replace_rule_translations(&state.db, rule.id, &req.translations).await?;
    Ok(Json(rule))
}

pub async fn update_rule(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<RuleReq>,
) -> AppResult<Json<RuleRow>> {
    admin.require(PERM_RULES_EDIT)?;
    let input = rule_input(&req)?;
    let rule = crate::db::update_rule(&state.db, id, input)
        .await
        .map_err(|e| code_conflict(e, &req.code))?;
    crate::db::replace_rule_sanctions(&state.db, rule.id, &req.sanctions).await?;
    crate::db::replace_rule_translations(&state.db, rule.id, &req.translations).await?;
    Ok(Json(rule))
}

/// GET /api/admin/rules/{id}/translations — языки одного правила для редактора.
pub async fn rule_translations(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<RuleTranslationRow>>> {
    admin.require(PERM_RULES_VIEW)?;
    Ok(Json(crate::db::translations_of_rule(&state.db, id).await?))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_RULES_DELETE)?;
    crate::db::delete_rule(&state.db, id).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn reorder_rules(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<ReorderReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_RULES_EDIT)?;
    crate::db::reorder_rules(&state.db, &req.order).await?;
    Ok(Json(json!({ "ok": true })))
}

fn required<'a>(value: &'a str, field: &str) -> AppResult<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    Ok(trimmed)
}

fn rule_input(req: &RuleReq) -> AppResult<RuleInput<'_>> {
    let code = required(&req.code, "code")?;
    let title = required(&req.title, "title")?;
    for sanction in &req.sanctions {
        if !KINDS.contains(&sanction.kind.as_str()) {
            return Err(AppError::BadRequest(format!(
                "unknown punishment {}",
                sanction.kind
            )));
        }
        if sanction.min_minutes.is_some_and(|m| m <= 0)
            || sanction.max_minutes.is_some_and(|m| m <= 0)
        {
            return Err(AppError::BadRequest("duration must be positive".into()));
        }
        // Перевёрнутая вилка означала бы правило, по которому нельзя выдать
        // вообще ничего, — и обнаружилось бы это в момент выдачи бана.
        if let (Some(min), Some(max)) = (sanction.min_minutes, sanction.max_minutes) {
            if max < min {
                return Err(AppError::BadRequest(
                    "the upper limit is below the lower one".into(),
                ));
            }
        }
    }
    Ok(RuleInput {
        code,
        title,
        description: req.description.trim(),
        punish_reason: req.punish_reason.trim(),
        category_id: req.category_id,
        server_id: req.server_id,
    })
}

/// Дубль кода — ошибка админа, а не сбой: у свода не может быть двух «1.1».
fn code_conflict(err: anyhow::Error, code: &str) -> AppError {
    let unique = err
        .downcast_ref::<sqlx::Error>()
        .and_then(|e| e.as_database_error())
        .is_some_and(|e| e.is_unique_violation());
    if unique {
        return AppError::Conflict(format!("rule {code} already exists"));
    }
    AppError::Other(err)
}
