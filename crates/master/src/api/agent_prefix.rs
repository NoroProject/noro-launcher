//! Префикс роли как право — в том виде, в каком его хранит LuckPerms.

use super::agent::AgentRole;

/// Префикс в формате LuckPerms: `prefix.<вес>.<значение>`.
///
/// Держать его правом, а не отдельным полем, — не наша прихоть: часть модов
/// читает меты именно из списка прав, и своя выдумка там просто не нашлась бы.
pub fn prefix_nodes(roles: &[AgentRole]) -> Vec<String> {
    roles
        .iter()
        .filter_map(|role| {
            let icon = role.icon.as_deref()?;
            let color = role.color.as_deref().unwrap_or("");
            Some(format!("prefix.{}.{}", role.sort_order, legacy_prefix(color, icon)))
        })
        .collect()
}

/// `#rrggbb` + иконка → `§x§r§r§g§g§b§b<иконка>§r`. Тот же вид, что собирает агент.
fn legacy_prefix(color: &str, icon: &str) -> String {
    let mut out = String::new();
    if color.len() == 7 && color.starts_with('#') {
        out.push_str("§x");
        for ch in color[1..].chars() {
            out.push('§');
            out.push(ch.to_ascii_lowercase());
        }
    }
    out.push_str(icon);
    out.push_str("§r");
    out
}
