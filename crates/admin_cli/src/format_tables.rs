//! Specialized table formatters for lists in noro-admin.

use serde_json::Value;

pub fn print_users_table(items: &[Value]) {
    println!(
        "\x1b[1;36m{:<38} {:<16} {:<14} {:<8} {:<10}\x1b[0m",
        "USER ID", "USERNAME", "ROLES", "ROOT", "CAN PLAY"
    );
    println!("{}", "─".repeat(90));
    for u in items {
        let id = u.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        let name = u.get("username").and_then(|v| v.as_str()).unwrap_or("-");
        let root = if u.get("is_root").and_then(|v| v.as_bool()).unwrap_or(false) {
            "\x1b[32mYES\x1b[0m"
        } else {
            "NO"
        };
        let can_play = if u.get("can_play").and_then(|v| v.as_bool()).unwrap_or(false) {
            "\x1b[32mYES\x1b[0m"
        } else {
            "NO"
        };
        let roles: Vec<&str> = u
            .get("roles")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r.get("name").and_then(|n| n.as_str()))
                    .collect()
            })
            .unwrap_or_default();
        let roles_str = if roles.is_empty() {
            "-".into()
        } else {
            roles.join(",")
        };
        println!(
            "{:<38} \x1b[1;33m{:<16}\x1b[0m {:<14} {:<8} {:<10}",
            id, name, roles_str, root, can_play
        );
    }
    println!("{}", "─".repeat(90));
    println!("Total: \x1b[1m{}\x1b[0m users", items.len());
}

pub fn print_servers_table(items: &[Value]) {
    println!(
        "\x1b[1;36m{:<38} {:<20} {:<12} {:<12} {:<8}\x1b[0m",
        "SERVER ID", "NAME", "MODLOADER", "MC VERSION", "ACTIVE"
    );
    println!("{}", "─".repeat(90));
    for s in items {
        let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("-");
        let loader = s.get("modloader").and_then(|v| v.as_str()).unwrap_or("-");
        let mc = s.get("mc_version").and_then(|v| v.as_str()).unwrap_or("-");
        let active = if s.get("active").and_then(|v| v.as_bool()).unwrap_or(true) {
            "\x1b[32mYES\x1b[0m"
        } else {
            "\x1b[31mNO\x1b[0m"
        };
        println!(
            "{:<38} \x1b[1;33m{:<20}\x1b[0m {:<12} {:<12} {:<8}",
            id, name, loader, mc, active
        );
    }
    println!("{}", "─".repeat(90));
    println!("Total: \x1b[1m{}\x1b[0m servers", items.len());
}

pub fn print_builds_table(items: &[Value]) {
    println!(
        "\x1b[1;36m{:<38} {:<12} {:<12} {:<12} {:<10}\x1b[0m",
        "BUILD ID", "VERSION", "MODLOADER", "MC VERSION", "PUBLISHED"
    );
    println!("{}", "─".repeat(90));
    for b in items {
        let id = b.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        let ver = b.get("version").and_then(|v| v.as_str()).unwrap_or("-");
        let loader = b.get("modloader").and_then(|v| v.as_str()).unwrap_or("-");
        let mc = b.get("mc_version").and_then(|v| v.as_str()).unwrap_or("-");
        let publ = if b
            .get("published")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            "\x1b[32mYES\x1b[0m"
        } else {
            "NO"
        };
        println!(
            "{:<38} \x1b[1;33m{:<12}\x1b[0m {:<12} {:<12} {:<10}",
            id, ver, loader, mc, publ
        );
    }
    println!("{}", "─".repeat(90));
    println!("Total: \x1b[1m{}\x1b[0m builds", items.len());
}

pub fn print_files_table(items: &[Value]) {
    println!(
        "\x1b[1;36m{:<40} {:<12} {:<10}\x1b[0m",
        "PATH / NAME", "SIZE", "TYPE"
    );
    println!("{}", "─".repeat(65));
    for f in items {
        let path = f
            .get("path")
            .or_else(|| f.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let size = f
            .get("size")
            .and_then(|v| v.as_u64())
            .map(|b| format!("{} KB", b / 1024))
            .unwrap_or_else(|| "-".into());
        let kind = f.get("type").and_then(|v| v.as_str()).unwrap_or("file");
        println!("{:<40} {:<12} {:<10}", path, size, kind);
    }
    println!("{}", "─".repeat(65));
    println!("Total: \x1b[1m{}\x1b[0m files", items.len());
}

pub fn print_generic_table(items: &[Value]) {
    let keys: Vec<&str> = items[0]
        .as_object()
        .map(|m| m.keys().take(6).map(|k| k.as_str()).collect())
        .unwrap_or_default();
    if keys.is_empty() {
        return;
    }
    for k in &keys {
        print!("\x1b[1;36m{:<18}\x1b[0m ", k.to_uppercase());
    }
    println!();
    println!("{}", "─".repeat(keys.len() * 20));
    for item in items {
        for k in &keys {
            let val_str = item
                .get(*k)
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => {
                        if *b {
                            "\x1b[32mYES\x1b[0m".into()
                        } else {
                            "NO".into()
                        }
                    }
                    Value::Null => "-".into(),
                    _ => v.to_string(),
                })
                .unwrap_or_else(|| "-".into());
            print!("{:<18} ", val_str);
        }
        println!();
    }
    println!("{}", "─".repeat(keys.len() * 20));
    println!("Total: \x1b[1m{}\x1b[0m items", items.len());
}
