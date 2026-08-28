//! Вычисление `rules` из version.json — на стороне клиента, а не мастера.
//!
//! Мастер раздаёт один манифест на все платформы и не знает, куда он уедет,
//! поэтому условные аргументы доходят до нас нетронутыми: `-XstartOnFirstThread`
//! нужен только macOS, а на Linux и Windows JVM от него просто не стартует.
//! Раньше мастер сворачивал правила под свою собственную ОС — и сборка,
//! собранная на Linux-сервере, ломала клиентов на macOS, и наоборот.

/// Значения аргумента, годные для этой машины. Пусто — аргумент не наш.
pub fn arg_values(arg: &schema::ManifestArg) -> &[String] {
    match arg {
        schema::ManifestArg::String(s) => std::slice::from_ref(s),
        schema::ManifestArg::Conditional { rules, value } => {
            if allow(rules) {
                value
            } else {
                &[]
            }
        }
    }
}

/// Разрешают ли правила то, к чему они прицеплены.
///
/// Правила читаются по порядку, и последнее подошедшее решает: так у Mojang
/// `allow` для всех и `disallow` для одной ОС даёт «всем, кроме неё».
/// Без правил разрешено, с правилами по умолчанию запрещено.
fn allow(rules: &[schema::ManifestRule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        // Feature-правила — про demo-режим, свой размер окна и quick play.
        // Ничего этого мы не включаем, поэтому такие правила не подходят
        // никогда: и `--demo`, и `--width` остаются за бортом.
        if rule.features.is_some() {
            continue;
        }
        if os_matches(rule.os.as_ref()) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn os_matches(os: Option<&schema::ManifestRuleOs>) -> bool {
    let Some(os) = os else {
        return true;
    };
    let name_ok = os.name.as_deref().is_none_or(|n| n == os_name());
    let arch_ok = os.arch.as_deref().is_none_or(|a| a == arch());
    let version_ok = os.version.as_deref().is_none_or(version_matches);
    name_ok && arch_ok && version_ok
}

fn os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    }
}

/// Архитектура в обозначениях Mojang.
///
/// На практике в манифестах 1.8–1.21 встречается единственное значение —
/// `x86`, то есть 32 бита, и только у `-Xss1M`. Ни `arm64`, ни `x86_64` там
/// не попадаются, так что ветка ниже почти всегда просто «не x86».
fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86" => "x86",
        _ => "x86_64",
    }
}

/// Сходится ли версия ОС с regex из правила (`^10\.` у Windows 10).
///
/// Правило с непонятным regex считаем не подошедшим: пропустить лишний
/// `-Dos.name=Windows 10` безопаснее, чем уронить запуск на разборе.
fn version_matches(pattern: &str) -> bool {
    let Ok(re) = regex::Regex::new(pattern) else {
        return false;
    };
    re.is_match(&os_version())
}

#[cfg(target_os = "windows")]
fn os_version() -> String {
    // `cmd /c ver` отдаёт "Microsoft Windows [Version 10.0.19045.5011]".
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .args(["/c", "ver"])
        // Без флага на секунду мигает консольное окно — прямо при запуске игры.
        .creation_flags(super::CREATE_NO_WINDOW)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            let start = s.find("Version ")? + "Version ".len();
            Some(s[start..].split(']').next()?.trim().to_string())
        })
        .unwrap_or_default()
}

/// Правила с `version` в манифестах есть только у Windows-аргументов, так что
/// на остальных системах спрашивать нечего — они и по `os.name` не подойдут.
#[cfg(not(target_os = "windows"))]
fn os_version() -> String {
    String::new()
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
