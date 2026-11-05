//! What streamer mode does to a console line.
//!
//! Lines reach the window already redacted (`schema::redact`): tokens are gone,
//! the home directory is masked, an IPv4 address keeps its first three parts so
//! a support log still shows which network the player was on. On stream that
//! is too much, so the whole address goes, and IPv6 and the host names of this
//! network's servers with it.

use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

pub const MASK: &str = "***";

static RULES: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
    [
        // A whole IPv4, or what redaction left of one (`10.0.0.***`).
        (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.(?:\d{1,3}\b|\*{3})", MASK),
        // IPv6, all eight groups or shortened with `::`. Fewer groups without
        // `::` would take a time of day (`12:30:15`) for an address.
        (
            r"(?i)\b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b|\b(?:[0-9a-f]{1,4}:)+:(?:[0-9a-f]{1,4}(?::[0-9a-f]{1,4})*)?",
            MASK,
        ),
        // Home directories, including what redaction already masked and the
        // Windows form with forward slashes it does not look for.
        (r"(?i)([A-Z]:[\\/]+Users[\\/]+)[^\\/\s]+", "${1}***"),
        (r"(/(?:Users|home)/)[^/\s]+", "${1}***"),
    ]
    .into_iter()
    .map(|(pattern, replacement)| {
        (
            Regex::new(pattern).expect("streamer mask pattern"),
            replacement,
        )
    })
    .collect()
});

/// The line with addresses and home directories masked, and each of `hosts`
/// replaced by what may be shown instead. Those are this network's servers: a
/// plain host name has no shape a pattern could find, so the window names them.
pub fn mask<'a>(text: &'a str, hosts: &[(String, String)]) -> Cow<'a, str> {
    let mut out = Cow::Borrowed(text);
    for (rule, replacement) in RULES.iter() {
        if let Cow::Owned(changed) = rule.replace_all(&out, *replacement) {
            out = Cow::Owned(changed);
        }
    }
    for (host, shown) in hosts.iter().filter(|(h, _)| !h.is_empty()) {
        if out.contains(host.as_str()) {
            out = Cow::Owned(out.replace(host.as_str(), shown));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::mask;

    fn m(text: &str) -> String {
        mask(text, &[]).into_owned()
    }

    #[test]
    fn addresses_go_whole() {
        assert_eq!(
            m("Connecting to 203.0.113.7, 25565"),
            "Connecting to ***, 25565"
        );
        assert_eq!(m("from 10.0.0.*** joined"), "from *** joined");
        assert_eq!(m("peer 2001:db8::1 left"), "peer *** left");
    }

    #[test]
    fn times_and_versions_stay() {
        assert_eq!(m("[12:30:15] [main/INFO]"), "[12:30:15] [main/INFO]");
        assert_eq!(m("Minecraft 1.21.1 loaded"), "Minecraft 1.21.1 loaded");
    }

    #[test]
    fn home_directories_lose_the_name() {
        assert_eq!(m(r"C:\Users\alice\AppData"), r"C:\Users\***\AppData");
        assert_eq!(m("C:/Users/alice/AppData"), "C:/Users/***/AppData");
        assert_eq!(m("/home/alice/.minecraft"), "/home/***/.minecraft");
        assert_eq!(m("/Users/*****/Library"), "/Users/***/Library");
    }

    #[test]
    fn known_hosts_are_masked() {
        let hosts = vec![("play.example.com".to_string(), "***".to_string())];
        assert_eq!(
            mask("Connecting to play.example.com, 25565", &hosts),
            "Connecting to ***, 25565"
        );
    }
}
