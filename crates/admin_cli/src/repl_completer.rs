//! Intelligent Tab autocompletion with human-readable labels & UUID replacement.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct CachedItem {
    pub id: String,
    pub label: String,
}

#[derive(Default, Clone)]
pub struct DynamicCache {
    pub server_items: Vec<CachedItem>,
    pub user_items: Vec<CachedItem>,
    pub build_items: Vec<CachedItem>,
    pub role_items: Vec<CachedItem>,
}

#[derive(Clone)]
pub struct ReplHelper {
    top_commands: Vec<String>,
    subcommands: HashMap<&'static str, Vec<&'static str>>,
    pub cache: Arc<RwLock<DynamicCache>>,
}

impl ReplHelper {
    pub fn new(cache: Arc<RwLock<DynamicCache>>) -> Self {
        let top_commands = vec![
            "server", "build", "file", "mod", "role", "user", "news", "core",
            "token", "launcher", "cape", "info", "audit", "backup", "punishment",
            "chat-filter", "blocklist", "oauth", "auth-methods", "case", "settings", "game",
            "exec", "console", "power", "kick", "tell", "announce",
            "connect", "set-server", "set-token", "status", "clear", "cls", "help", "exit", "quit",
            "servers", "builds", "users", "stats", "agents"
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut subcommands: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        subcommands.insert("server", vec!["list", "get", "create", "edit", "delete", "reorder", "upload-icon", "upload-bg", "game-server"]);
        subcommands.insert("build", vec!["list", "get", "create", "publish", "unpublish", "rebuild", "rebuild-clean", "delete", "set-versions", "set-paths", "set-settings", "import-mrpack", "import-cf", "import-zip", "import-progress", "optional-mods"]);
        subcommands.insert("file", vec!["list", "upload", "download", "delete", "move", "edit"]);
        subcommands.insert("mod", vec!["search", "install", "install-url", "install-cf", "install-mr", "list", "remove"]);
        subcommands.insert("role", vec!["list", "create", "edit", "delete", "set-badge", "set-permissions"]);
        subcommands.insert("user", vec!["list", "get", "ban", "unban", "add-role", "set-role", "remove-role", "add-perm", "set-perm", "remove-perm", "set-cape"]);
        subcommands.insert("news", vec!["list", "create", "edit", "delete"]);
        subcommands.insert("core", vec!["list", "upload", "delete"]);
        subcommands.insert("token", vec!["list", "create", "revoke"]);
        subcommands.insert("launcher", vec!["list", "build", "publish", "delete"]);
        subcommands.insert("cape", vec!["list", "upload", "delete"]);
        subcommands.insert("info", vec!["stats", "agents", "permissions", "mc-versions", "loader-versions"]);
        subcommands.insert("audit", vec!["list"]);
        subcommands.insert("backup", vec!["list", "create", "restore"]);
        subcommands.insert("punishment", vec!["list", "revoke"]);
        subcommands.insert("chat-filter", vec!["list", "add", "delete"]);
        subcommands.insert("blocklist", vec!["list", "add", "delete"]);
        subcommands.insert("oauth", vec!["list", "create", "delete"]);
        subcommands.insert("auth-methods", vec!["list", "set"]);
        subcommands.insert("case", vec!["list", "get", "close"]);
        subcommands.insert("settings", vec!["get", "maintenance", "rebuild-prefixes"]);
        subcommands.insert("game", vec!["exec", "console", "power", "kick", "tell", "announce"]);
        subcommands.insert("power", vec!["start", "stop", "restart", "kill"]);

        Self { top_commands, subcommands, cache }
    }
}

impl Helper for ReplHelper {}
impl Hinter for ReplHelper { type Hint = String; }
impl Highlighter for ReplHelper {}
impl Validator for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_till_pos = &line[..pos];
        let words: Vec<&str> = line_till_pos.split_whitespace().collect();
        let ends_with_space = line_till_pos.ends_with(' ');

        if words.is_empty() {
            let pairs = self.top_commands.iter().map(|s| Pair { display: s.clone(), replacement: s.clone() }).collect();
            return Ok((pos, pairs));
        }

        if words.len() == 1 && ends_with_space {
            let cmd = words[0].to_lowercase();
            if let Some(subs) = self.subcommands.get(cmd.as_str()) {
                let pairs = subs.iter().map(|s| Pair { display: (*s).into(), replacement: (*s).into() }).collect();
                return Ok((pos, pairs));
            }
        }

        if words.len() == 1 && !ends_with_space {
            let last = words[0];
            let start = pos.saturating_sub(last.len());
            let matches: Vec<Pair> = self
                .top_commands
                .iter()
                .filter(|c| c.starts_with(last))
                .map(|c| Pair { display: c.clone(), replacement: c.clone() })
                .collect();
            return Ok((start, matches));
        }

        let first = words[0].to_lowercase();
        let sub = words.get(1).map(|s| s.to_lowercase()).unwrap_or_default();

        if words.len() == 2 && !ends_with_space {
            let last = words[1];
            let start = pos.saturating_sub(last.len());
            if let Some(subs) = self.subcommands.get(first.as_str()) {
                let matches: Vec<Pair> = subs
                    .iter()
                    .filter(|s| s.starts_with(last))
                    .map(|s| Pair { display: (*s).into(), replacement: (*s).into() })
                    .collect();
                if !matches.is_empty() {
                    return Ok((start, matches));
                }
            }
        }

        let filter = if ends_with_space { "" } else { words.last().copied().unwrap_or("") };
        let start = if ends_with_space { pos } else { pos.saturating_sub(filter.len()) };

        if let Ok(c) = self.cache.read() {
            let target_items = if first == "user" && (sub == "add-role" || sub == "set-role" || sub == "remove-role") && (words.len() == 3 || (words.len() == 2 && ends_with_space)) {
                if words.len() == 3 || (words.len() == 2 && !ends_with_space) { &c.role_items } else { &c.user_items }
            } else {
                match first.as_str() {
                    "exec" | "console" | "power" | "logs" => &c.server_items,
                    "tell" | "kick" | "user" => &c.user_items,
                    "server" => &c.server_items,
                    "build" | "file" | "mod" => &c.build_items,
                    _ => &vec![],
                }
            };

            let matches: Vec<Pair> = target_items
                .iter()
                .filter(|item| item.id.starts_with(filter) || item.label.to_lowercase().starts_with(&filter.to_lowercase()))
                .map(|item| Pair {
                    display: format!("{} ({})", item.id, item.label),
                    replacement: item.id.clone(),
                })
                .collect();
            if !matches.is_empty() {
                return Ok((start, matches));
            }
        }

        Ok((pos, vec![]))
    }
}
