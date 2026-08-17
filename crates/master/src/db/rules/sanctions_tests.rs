use super::*;

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

/// Рамки — это и есть ограничение хелпера: час при вилке «30м–2ч» проходит,
/// неделя нет.
#[test]
fn a_term_inside_the_range_passes_and_outside_does_not() {
    let mute = sanction("mute", Some(30), Some(120));
    assert!(mute.allows(Some(30)));
    assert!(mute.allows(Some(60)));
    assert!(mute.allows(Some(120)));
    assert!(!mute.allows(Some(29)));
    assert!(!mute.allows(Some(10080)));
}

/// «Навсегда» проходит только там, где верхняя граница не задана: иначе вилка
/// «7д–30д» молча превращалась бы в вечный бан.
#[test]
fn forever_needs_an_open_upper_bound() {
    assert!(!sanction("ban", Some(10080), Some(43200)).allows(None));
    assert!(sanction("ban", Some(10080), None).allows(None));
}

/// У предупреждения срока нет, и рамки к нему не применяются.
#[test]
fn a_warning_has_no_term() {
    let warn = sanction("warn", None, None);
    assert!(warn.allows(None));
    assert!(warn.allows(Some(60)));
}

/// Открытая нижняя граница разрешает любой короткий срок — так задают
/// «мут до суток» одним значением сверху.
#[test]
fn an_open_lower_bound_allows_short_terms() {
    let mute = sanction("mute", None, Some(1440));
    assert!(mute.allows(Some(1)));
    assert!(mute.allows(Some(1440)));
    assert!(!mute.allows(Some(1441)));
    assert!(!mute.allows(None));
}
