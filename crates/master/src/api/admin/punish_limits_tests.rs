use super::*;
use schema::{PERM_PUNISH_BYPASS, PERM_PUNISH_PERMANENT};
use uuid::Uuid;

fn sanction(kind: &str, min: Option<i64>, max: Option<i64>) -> RuleSanctionRow {
    RuleSanctionRow {
        id: Uuid::nil(),
        rule_id: Uuid::nil(),
        kind: kind.into(),
        label: String::new(),
        min_minutes: min,
        max_minutes: max,
        sort_order: 0,
    }
}

fn helper(perm: &str) -> bool {
    ["noro.mod.punish.mute", "noro.mod.punish.warn"].contains(&perm)
}

fn senior(perm: &str) -> bool {
    perm.starts_with("noro.mod.punish.")
}

fn request(kind: &str, minutes: Option<i64>) -> Request<'_> {
    Request {
        kind,
        minutes,
        rule_cited: true,
    }
}

/// Ради этого всё и затевалось: хелпер выдаёт мут ровно в рамках правила.
#[test]
fn a_helper_stays_inside_the_rule() {
    let rule = [sanction("mute", Some(30), Some(120))];
    assert!(check(&request("mute", Some(60)), &rule, helper).is_ok());
    assert!(check(&request("mute", Some(600)), &rule, helper).is_err());
}

/// Вид наказания — отдельное право: мут есть, бана нет.
#[test]
fn a_kind_without_a_permission_is_refused() {
    let rule = [sanction("ban", Some(1440), Some(10080))];
    let err = check(&request("ban", Some(1440)), &rule, helper).unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
}

/// Байпас снимает рамки, но не заменяет право на сам вид наказания.
#[test]
fn bypass_lifts_the_range_but_not_the_kind() {
    let rule = [sanction("mute", Some(30), Some(120))];
    let bypassing = |perm: &str| helper(perm) || perm == PERM_PUNISH_BYPASS;
    assert!(check(&request("mute", Some(100_000)), &rule, bypassing).is_ok());
    assert!(check(&request("ban", Some(60)), &rule, bypassing).is_err());
}

/// Без байпаса наказание обязано ссылаться на правило: иначе рамки обходятся
/// простым «не выбирать пункт».
#[test]
fn a_punishment_without_a_rule_needs_bypass() {
    let free = Request {
        kind: "mute",
        minutes: Some(60),
        rule_cited: false,
    };
    assert!(check(&free, &[], helper).is_err());
    assert!(check(&free, &[], senior).is_ok());
}

/// Правило без санкций ничего не разрешает само по себе.
#[test]
fn a_rule_without_sanctions_allows_nothing() {
    assert!(check(&request("mute", Some(60)), &[], helper).is_err());
}

/// «Навсегда» требует своего права, даже когда правило это позволяет.
#[test]
fn forever_needs_its_own_permission() {
    let rule = [sanction("mute", Some(30), None)];
    assert!(check(&request("mute", None), &rule, helper).is_err());

    let with_permanent = |perm: &str| helper(perm) || perm == PERM_PUNISH_PERMANENT;
    assert!(check(&request("mute", None), &rule, with_permanent).is_ok());
}

/// Предупреждение срока не имеет, и вечным его никто не считает.
#[test]
fn a_warning_is_not_a_permanent_punishment() {
    let rule = [sanction("warn", None, None)];
    assert!(check(&request("warn", None), &rule, helper).is_ok());
}

/// Текст отказа перечисляет допустимое: по нему видно, что исправить.
#[test]
fn the_refusal_lists_what_is_allowed() {
    let rule = [
        sanction("mute", Some(30), Some(120)),
        sanction("ban", Some(10080), None),
    ];
    assert_eq!(describe(&rule), "mute 30m–2h, ban from 7d");
}
