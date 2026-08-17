use super::*;
use chrono::Utc;

fn rule(title: &str, description: &str) -> RuleRow {
    RuleRow {
        id: Uuid::nil(),
        category_id: None,
        server_id: None,
        code: "1.1".into(),
        title: title.into(),
        description: description.into(),
        punish_reason: "Гриферство постройки".into(),
        sort_order: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn translation(title: &str, description: &str) -> RuleTranslationRow {
    translation_with_reason(title, description, "")
}

fn translation_with_reason(
    title: &str,
    description: &str,
    punish_reason: &str,
) -> RuleTranslationRow {
    RuleTranslationRow {
        rule_id: Uuid::nil(),
        locale: "en".into(),
        title: title.into(),
        description: description.into(),
        punish_reason: punish_reason.into(),
    }
}

/// Формулировка наказания переводится вместе со сводом: её читает игрок в бане,
/// а не администратор.
#[test]
fn a_translation_replaces_the_punish_reason() {
    let mut rules = [rule("Гриферство", "Ломать чужое нельзя")];
    apply_locale(
        &mut [],
        &mut rules,
        &mut [],
        &[],
        &[translation_with_reason("Griefing", "", "Griefing builds")],
        &[],
    );
    assert_eq!(rules[0].punish_reason, "Griefing builds");
}

/// Пустой перевод формулировки оставляет исходную: пустая причина в бане — это
/// наказание без объяснения.
#[test]
fn an_empty_translated_reason_keeps_the_original() {
    let mut rules = [rule("Гриферство", "Ломать чужое нельзя")];
    apply_locale(
        &mut [],
        &mut rules,
        &mut [],
        &[],
        &[translation("Griefing", "")],
        &[],
    );
    assert_eq!(rules[0].punish_reason, "Гриферство постройки");
}

/// Перевод замещает исходный текст — ради этого всё и заводилось.
#[test]
fn a_translation_replaces_the_original() {
    let mut rules = [rule("Гриферство", "Ломать чужое нельзя")];
    apply_locale(
        &mut [],
        &mut rules,
        &mut [],
        &[],
        &[translation("Griefing", "Do not break what is not yours")],
        &[],
    );
    assert_eq!(rules[0].title, "Griefing");
    assert_eq!(rules[0].description, "Do not break what is not yours");
}

/// Непереведённый пункт остаётся на исходном языке: пустое место читалось бы
/// как «правила нет», и сослаться на него было бы нельзя.
#[test]
fn a_missing_translation_falls_back_to_the_original() {
    let mut rules = [rule("Гриферство", "Ломать чужое нельзя")];
    apply_locale(&mut [], &mut rules, &mut [], &[], &[], &[]);
    assert_eq!(rules[0].title, "Гриферство");
}

/// Переведён заголовок, но не текст — текст остаётся исходным, а не пустым.
#[test]
fn an_empty_description_does_not_erase_the_original() {
    let mut rules = [rule("Гриферство", "Ломать чужое нельзя")];
    apply_locale(
        &mut [],
        &mut rules,
        &mut [],
        &[],
        &[translation("Griefing", "")],
        &[],
    );
    assert_eq!(rules[0].title, "Griefing");
    assert_eq!(rules[0].description, "Ломать чужое нельзя");
}
