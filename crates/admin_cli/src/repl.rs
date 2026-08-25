//! Interactive REPL console for noro-admin.

use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};
use std::sync::{Arc, RwLock};

use crate::client::Client;
use crate::repl_completer::{CachedItem, DynamicCache, ReplHelper};
use crate::{execute_command, Cli};

pub async fn run_repl(initial_server: String, initial_token: String) -> Result<()> {
    let mut server = if initial_server == "http://localhost:8080" {
        std::env::var("NORO_MASTER_URL").unwrap_or(initial_server)
    } else {
        initial_server
    };
    let mut token = if initial_token.is_empty() {
        std::env::var("NORO_ADMIN_TOKEN").unwrap_or(initial_token)
    } else {
        initial_token
    };

    let cache = Arc::new(RwLock::new(DynamicCache::default()));
    let helper = ReplHelper::new(cache.clone());

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let history_path = dirs::home_dir().map(|h| h.join(".noro_admin_history"));
    let mut rl = Editor::<ReplHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("\x1b[1;36m✦ noro-admin Interactive Console ✦\x1b[0m");
    println!("Server: \x1b[33m{}\x1b[0m", server);
    println!(
        "Token:  \x1b[33m{}\x1b[0m",
        if token.is_empty() { "<none>" } else { "<set>" }
    );
    println!("Type \x1b[32m'help'\x1b[0m for commands, \x1b[32m'connect <URL> [TOKEN]'\x1b[0m to switch server, \x1b[32m<Tab>\x1b[0m for autocomplete.\n");

    let client_init = Client::new(server.clone(), token.clone());
    let cache_init = cache.clone();
    tokio::spawn(async move { refresh_cache(&client_init, cache_init).await });

    loop {
        let prompt = format!("\x1b[1;34mnoro-admin ({})\x1b[0m> ", server);
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                let mut words = match shell_words::split(trimmed) {
                    Ok(w) => w,
                    Err(err) => {
                        println!("\x1b[31mCommand parse error:\x1b[0m {err}");
                        continue;
                    }
                };

                let cmd_name = words[0].to_lowercase();
                match cmd_name.as_str() {
                    "exit" | "quit" | "q" => break,
                    "clear" | "cls" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    "status" => {
                        println!("Server: {server}");
                        println!(
                            "Token:  {}",
                            if token.is_empty() { "<none>" } else { "<set>" }
                        );
                        let c = Client::new(server.clone(), token.clone());
                        match c.get("/health").await {
                            Ok(_) => println!("Health: \x1b[32mOK\x1b[0m"),
                            Err(e) => println!("Health check failed: \x1b[31m{e}\x1b[0m"),
                        }
                        refresh_cache(&c, cache.clone()).await;
                        continue;
                    }
                    "connect" | "set-server" => {
                        if words.len() > 1 {
                            server = words[1].clone();
                            if words.len() > 2 {
                                token = words[2].clone();
                            }
                            println!("Connected to \x1b[33m{server}\x1b[0m");
                            let c = Client::new(server.clone(), token.clone());
                            let cache_clone = cache.clone();
                            tokio::spawn(async move { refresh_cache(&c, cache_clone).await });
                        } else {
                            println!("Usage: connect <URL> [TOKEN]");
                        }
                        continue;
                    }
                    "set-token" => {
                        if words.len() > 1 {
                            token = words[1].clone();
                            println!("Token updated.");
                            let c = Client::new(server.clone(), token.clone());
                            let cache_clone = cache.clone();
                            tokio::spawn(async move { refresh_cache(&c, cache_clone).await });
                        } else {
                            println!("Usage: set-token <TOKEN>");
                        }
                        continue;
                    }
                    "help" | "?" => {
                        print_repl_help();
                        continue;
                    }
                    "servers" | "ps" => {
                        words = vec!["server".into(), "list".into()];
                    }
                    "stats" => {
                        words = vec!["info".into(), "stats".into()];
                    }
                    "agents" => {
                        words = vec!["info".into(), "agents".into()];
                    }
                    "users" => {
                        words = vec!["user".into(), "list".into()];
                    }
                    "logs" => {
                        if words.len() > 1 {
                            words[0] = "console".into();
                        } else {
                            println!("Usage: logs <game_server_id>");
                            continue;
                        }
                    }
                    _ => {}
                }

                let mut args = vec![
                    "noro-admin".to_string(),
                    "--server".to_string(),
                    server.clone(),
                ];
                if !token.is_empty() {
                    args.push("--token".to_string());
                    args.push(token.clone());
                }
                args.extend(words);

                match Cli::try_parse_from(args) {
                    Ok(parsed) => {
                        let c = Client::new(server.clone(), token.clone());
                        if let Some(cmd) = parsed.command {
                            if let Err(err) = execute_command(&c, cmd).await {
                                println!("\x1b[31mError:\x1b[0m {err}");
                            }
                        }
                    }
                    Err(err) => println!("{}", err.render()),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error reading input: {:?}", err);
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }
    println!("Bye!");
    Ok(())
}

async fn refresh_cache(c: &Client, cache: Arc<RwLock<DynamicCache>>) {
    let mut server_items = Vec::new();
    let mut build_items = Vec::new();
    let mut user_items = Vec::new();
    let mut role_items = Vec::new();

    if let Ok(v) = c.get("/api/admin/servers").await {
        if let Some(arr) = v
            .as_array()
            .or_else(|| v.get("items").and_then(|i| i.as_array()))
        {
            for item in arr {
                let name = item
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("server");
                if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                    server_items.push(CachedItem {
                        id: id.to_string(),
                        label: name.to_string(),
                    });
                }
                if let Some(gs_arr) = item.get("game_servers").and_then(|g| g.as_array()) {
                    for gs in gs_arr {
                        let gs_name = gs.get("name").and_then(|s| s.as_str()).unwrap_or(name);
                        if let Some(gs_id) = gs.get("id").and_then(|s| s.as_str()) {
                            server_items.push(CachedItem {
                                id: gs_id.to_string(),
                                label: gs_name.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if let Ok(v) = c.get("/api/admin/users").await {
        if let Some(arr) = v
            .as_array()
            .or_else(|| v.get("items").and_then(|i| i.as_array()))
        {
            for item in arr {
                let name = item
                    .get("username")
                    .and_then(|s| s.as_str())
                    .unwrap_or("user");
                if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                    user_items.push(CachedItem {
                        id: id.to_string(),
                        label: name.to_string(),
                    });
                }
            }
        }
    }

    if let Ok(v) = c.get("/api/admin/builds").await {
        if let Some(arr) = v
            .as_array()
            .or_else(|| v.get("items").and_then(|i| i.as_array()))
        {
            for item in arr {
                let ver = item
                    .get("version")
                    .and_then(|s| s.as_str())
                    .unwrap_or("build");
                let mc = item
                    .get("mc_version")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let label = format!("{ver} ({mc})");
                if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                    build_items.push(CachedItem {
                        id: id.to_string(),
                        label,
                    });
                }
            }
        }
    }

    if let Ok(v) = c.get("/api/admin/roles").await {
        if let Some(arr) = v
            .as_array()
            .or_else(|| v.get("items").and_then(|i| i.as_array()))
        {
            for item in arr {
                let name = item.get("name").and_then(|s| s.as_str()).unwrap_or("role");
                if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                    role_items.push(CachedItem {
                        id: id.to_string(),
                        label: name.to_string(),
                    });
                }
            }
        }
    }

    if let Ok(mut lock) = cache.write() {
        lock.server_items = server_items;
        lock.user_items = user_items;
        lock.build_items = build_items;
        lock.role_items = role_items;
    }
}

fn print_repl_help() {
    println!("\x1b[1mAvailable Noro Admin Commands:\x1b[0m");
    println!("  \x1b[36mserver\x1b[0m       List, inspect, create, start, stop, edit game servers");
    println!("  \x1b[36mexec\x1b[0m         Send console command to server (e.g. exec <id> gamemode 1 Alex)");
    println!("  \x1b[36mconsole\x1b[0m      View live console logs of a game server");
    println!("  \x1b[36mpower\x1b[0m        Control server power state (start|stop|restart|kill)");
    println!("  \x1b[36mbuild\x1b[0m        List, inspect, create, publish, delete builds");
    println!(
        "  \x1b[36mfile\x1b[0m         Manage files in builds (upload, download, move, delete)"
    );
    println!(
        "  \x1b[36mmod\x1b[0m          Search Modrinth/CurseForge and install mods into builds"
    );
    println!("  \x1b[36mrole / user\x1b[0m  Manage roles, users, bans, permissions, capes");
    println!("  \x1b[36mnews / core\x1b[0m  Manage news posts and server core JARs");
    println!("  \x1b[36mtoken / cape\x1b[0m Manage admin tokens and player capes");
    println!("  \x1b[36minfo / audit\x1b[0m Stats, agents, MC/loader versions, audit logs");
    println!("  \x1b[36mbackup\x1b[0m       List, create, and restore server backups");
    println!("  \x1b[36mpunishment\x1b[0m   List and revoke user bans/mutes");
    println!("  \x1b[36mchat-filter\x1b[0m  Manage chat filter rules");
    println!("  \x1b[36mblocklist\x1b[0m    Manage blocked IPs and hardware IDs");
    println!("  \x1b[36moauth / case\x1b[0m OAuth2 apps and moderation cases");
    println!("  \x1b[36msettings\x1b[0m     Global settings, maintenance mode, prefix packs");
    println!("\x1b[1mREPL Controls & Dynamic Autocompletion:\x1b[0m");
    println!("  \x1b[32m<Tab>\x1b[0m                 Auto-completes commands, subcommands, server IDs, players & builds");
    println!("  \x1b[32mservers / ps\x1b[0m          Shortcut for 'server list'");
    println!("  \x1b[32mstats / agents\x1b[0m        Shortcut for 'info stats' / 'info agents'");
    println!("  \x1b[32mconnect <URL> [TOKEN]\x1b[0m  Switch server URL and optional token");
    println!("  \x1b[32mstatus\x1b[0m                 Show server connection and health");
    println!("  \x1b[32mexit / quit\x1b[0m            Exit REPL mode");
}
