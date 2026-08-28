//! Evaluating `rules` from version.json, client-side.
//!
//! One manifest serves every platform, so conditional arguments reach us
//! untouched and are resolved here. They have to be: `-XstartOnFirstThread` is
//! required on macOS and stops the JVM from starting anywhere else, so a
//! manifest resolved on the machine that built it would be wrong for everyone
//! running a different OS.

/// The values this argument contributes on this machine — empty if its rules
/// don't match.
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

/// Rules are read in order and the last matching one wins — that's how Mojang
/// expresses "everyone except macOS" as `allow` for all plus `disallow` for one.
/// No rules means allowed; any rules means denied unless one matches.
fn allow(rules: &[schema::ManifestRule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        // Feature rules cover demo mode, custom window size and quick play. We
        // enable none of those, so a feature rule never matches and `--demo`,
        // `--width` and friends stay out of the command line.
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

/// Architecture in Mojang's spelling. In practice manifests only ever name
/// `x86`, so the last arm is really "not 32-bit".
fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86" => "x86",
        _ => "x86_64",
    }
}

/// Matches the OS version against the rule's regex (`^10\.` for Windows 10). A
/// pattern that won't compile counts as no match — skipping one `-Dos.name`
/// argument beats failing a launch over a regex.
fn version_matches(pattern: &str) -> bool {
    let Ok(re) = regex::Regex::new(pattern) else {
        return false;
    };
    re.is_match(&os_version())
}

#[cfg(target_os = "windows")]
fn os_version() -> String {
    // `cmd /c ver` prints "Microsoft Windows [Version 10.0.19045.5011]".
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .args(["/c", "ver"])
        // Without this a console window flashes up mid-launch.
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

/// Only Windows arguments carry a `version` rule, and those fail on `os.name`
/// here anyway, so there is nothing to look up.
#[cfg(not(target_os = "windows"))]
fn os_version() -> String {
    String::new()
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
