//! Правила: чтение сводом — сайтом, лаунчером и агентом.
//!
//! Публичное: свод правил и есть публичный документ, на который ссылается
//! каждый бан. Прятать его за входом означало бы, что забаненный не может
//! прочитать, за что именно.

use crate::api::auth::AgentAuth;
use crate::db::rules::{RuleCategoryRow, RuleRow, RuleSanctionRow};
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct RulesResponse {
    /// Свод, который вернули: `None` — общий, иначе сервер-дополнение.
    pub server_id: Option<Uuid>,
    pub categories: Vec<RuleCategoryRow>,
    pub rules: Vec<RuleRow>,
    /// Допустимые наказания, разложенные по правилам. Отдельным списком, а не
    /// внутри правила: страница и форма модерации сопоставляют их по `rule_id`,
    /// а запрос на каждое правило превратил бы открытие свода в сотню обращений.
    pub sanctions: Vec<RuleSanctionRow>,
}

#[derive(Serialize)]
pub struct RuleScope {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct RulesQuery {
    /// Сервер, дополнение которого показать вместе с общим сводом.
    pub server_id: Option<Uuid>,
    /// Язык свода. Без перевода пункт остаётся на исходном языке.
    pub locale: Option<String>,
}

async fn load(
    state: &AppState,
    server_id: Option<Uuid>,
    locale: Option<&str>,
) -> AppResult<Json<RulesResponse>> {
    let mut categories = crate::db::list_rule_categories(&state.db, server_id).await?;
    let mut rules = crate::db::list_rules(&state.db, server_id).await?;
    let ids: Vec<_> = rules.iter().map(|r| r.id).collect();
    let mut sanctions = crate::db::list_rule_sanctions(&state.db, &ids).await?;

    if let Some(locale) = locale.filter(|l| !l.is_empty()) {
        let cat_tr = crate::db::category_translations(&state.db, locale).await?;
        let rule_tr = crate::db::rule_translations(&state.db, locale).await?;
        let sanction_tr = crate::db::sanction_translations(&state.db, locale).await?;
        crate::db::apply_locale(
            &mut categories,
            &mut rules,
            &mut sanctions,
            &cat_tr,
            &rule_tr,
            &sanction_tr,
        );
    }
    Ok(Json(RulesResponse {
        server_id,
        categories,
        rules,
        sanctions,
    }))
}

/// GET /api/rules — общий свод, с `?server_id=` ещё и дополнение сервера.
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<RulesQuery>,
) -> AppResult<Json<RulesResponse>> {
    load(&state, query.server_id, query.locale.as_deref()).await
}

/// GET /api/rules/servers/{server_id} — тот же свод адресом, пригодным для
/// ссылки из игры.
pub async fn by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Query(query): Query<RulesQuery>,
) -> AppResult<Json<RulesResponse>> {
    load(&state, Some(server_id), query.locale.as_deref()).await
}

/// GET /api/rules/scopes — серверы, у которых свод отличается от общего.
pub async fn scopes(State(state): State<AppState>) -> AppResult<Json<Vec<RuleScope>>> {
    let rows = crate::db::list_rule_scopes(&state.db).await?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, name)| RuleScope { id, name })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct AgentRulesQuery {
    pub lang: Option<String>,
    pub locale: Option<String>,
}

/// GET /api/agent/rules — свод своего сервера: плагину он нужен для
/// автодополнения кодов в командах модерации.
pub async fn agent_list(
    State(state): State<AppState>,
    agent: AgentAuth,
    Query(query): Query<AgentRulesQuery>,
) -> AppResult<Json<RulesResponse>> {
    let lang = query.lang.or(query.locale);
    load(&state, Some(agent.game_server.server_id), lang.as_deref()).await
}

#[derive(Serialize)]
pub struct RuleWithSanctions {
    #[serde(flatten)]
    pub rule: RuleRow,
    pub sanctions: Vec<RuleSanctionRow>,
}

/// GET /api/agent/rules/{code} — одно правило по коду вместе с допустимыми
/// наказаниями: по ним плагин заполняет команду бана и проверяет срок.
pub async fn agent_by_code(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(code): Path<String>,
    Query(query): Query<AgentRulesQuery>,
) -> AppResult<Json<RuleWithSanctions>> {
    let server_id = agent.game_server.server_id;
    let mut rule = crate::db::rule_by_code(&state.db, &code, Some(server_id))
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("rule {code} not found")))?;
    let mut sanctions = crate::db::sanctions_of_rule(&state.db, rule.id).await?;

    let lang = query.lang.or(query.locale);
    if let Some(locale) = lang.filter(|l| !l.is_empty()) {
        let rule_tr = crate::db::rule_translations(&state.db, &locale).await?;
        let sanction_tr = crate::db::sanction_translations(&state.db, &locale).await?;
        let mut dummy_cats = vec![];
        let mut dummy_rules = vec![rule.clone()];
        crate::db::apply_locale(
            &mut dummy_cats,
            &mut dummy_rules,
            &mut sanctions,
            &[],
            &rule_tr,
            &sanction_tr,
        );
        rule = dummy_rules.remove(0);
    }

    Ok(Json(RuleWithSanctions { rule, sanctions }))
}
