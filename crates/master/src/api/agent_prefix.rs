//! Префикс и суффикс роли как права — в том виде, в каком их хранит LuckPerms.

use super::agent::types::AgentRole;

/// Меты в формате LuckPerms: `prefix.<вес>.<значение>` и `suffix.<вес>.<…>`.
///
/// Держать их правами, а не отдельными полями, — не наша прихоть: часть модов
/// читает меты именно из списка прав, и своя выдумка там просто не нашлась бы.
pub fn meta_nodes(roles: &[AgentRole]) -> Vec<String> {
    let mut nodes = Vec::new();
    for role in roles {
        if let Some(prefix) = shown_prefix(role) {
            nodes.push(format!("prefix.{}.{}", role.sort_order, prefix));
        }
        if let Some(suffix) = role.suffix.as_deref().filter(|s| !s.is_empty()) {
            nodes.push(format!("suffix.{}.{}", role.sort_order, legacy(suffix)));
        }
    }
    nodes
}

/// Заданный префикс роли, иначе — иконка в цвете роли.
///
/// Запасной вариант нужен ради тех сборок, что жили до появления поля: там
/// префиксом работала иконка, и молча снять её обновлением значило бы стереть
/// оформление чата на живом сервере.
fn shown_prefix(role: &AgentRole) -> Option<String> {
    if let Some(prefix) = role.prefix.as_deref().filter(|s| !s.is_empty()) {
        return Some(legacy(prefix));
    }
    let icon = role.icon.as_deref()?;
    let color = role.color.as_deref().unwrap_or("");
    Some(format!("{}{icon}§r", hex_color(color)))
}

/// `&c` → `§c`: в админке цвета набирают амперсандом, в игре они живут только
/// со знаком параграфа.
fn legacy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        let code = ch == '&'
            && chars
                .peek()
                .is_some_and(|next| next.is_ascii_hexdigit() || "klmnorKLMNOR".contains(*next));
        out.push(if code { '§' } else { ch });
    }
    out
}

/// `#rrggbb` → `§x§r§r§g§g§b§b`. Пустая строка, если цвет не задан.
fn hex_color(color: &str) -> String {
    if color.len() != 7 || !color.starts_with('#') {
        return String::new();
    }
    let mut out = String::from("§x");
    for ch in color[1..].chars() {
        out.push('§');
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(prefix: Option<&str>, suffix: Option<&str>, icon: Option<&str>) -> AgentRole {
        AgentRole {
            name: "admin".into(),
            display_name: "Админ".into(),
            lp_group: None,
            color: Some("#ff8c82".into()),
            icon: icon.map(str::to_string),
            prefix: prefix.map(str::to_string),
            suffix: suffix.map(str::to_string),
            sort_order: 100,
        }
    }

    #[test]
    fn prefix_wins_over_icon() {
        let nodes = meta_nodes(&[role(Some("&8[&cADMIN&8] "), Some(" &7★"), Some("★"))]);
        assert_eq!(
            nodes,
            vec![
                "prefix.100.§8[§cADMIN§8] ".to_string(),
                "suffix.100. §7★".to_string(),
            ]
        );
    }

    /// Роль без префикса продолжает жить иконкой: обновление не должно снимать
    /// оформление, которое уже настроено на сервере.
    #[test]
    fn icon_stays_the_fallback() {
        let nodes = meta_nodes(&[role(None, None, Some("★"))]);
        assert_eq!(nodes, vec!["prefix.100.§x§f§f§8§c§8§2★§r".to_string()]);
    }

    /// Одинокий амперсанд — просто амперсанд: `Tom & Jerry` не цветовой код.
    #[test]
    fn lone_ampersand_survives() {
        let nodes = meta_nodes(&[role(Some("Tom & Jerry "), None, None)]);
        assert_eq!(nodes, vec!["prefix.100.Tom & Jerry ".to_string()]);
    }
}
