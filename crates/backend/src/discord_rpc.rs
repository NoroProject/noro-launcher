// Over 150 lines: the client thread and the activity it sends; Discord may come
// and go, and each update reconnects first when it has to.
//! Discord Rich Presence.
//!
//! Discord may not be running, so every connection failure is non-fatal and the
//! loop keeps retrying on a timer.

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{debug, warn};

const DEFAULT_DISCORD_APP_ID: &str = "1512048650258219089";

#[derive(Debug, Clone, PartialEq)]
pub enum DiscordRpcState {
    Launcher {
        server_name: Option<String>,
    },
    GameLoading {
        server_name: String,
    },
    GameMenu {
        server_name: String,
        start_timestamp: u64,
    },
    GamePlaying {
        server_name: String,
        online_current: Option<u32>,
        online_max: Option<u32>,
        start_timestamp: u64,
    },
}

enum Command {
    State(DiscordRpcState),
    Enable(bool),
}

#[derive(Clone)]
pub struct DiscordRpc {
    tx: mpsc::UnboundedSender<Command>,
}

impl DiscordRpc {
    pub fn update(&self, state: DiscordRpcState) {
        let _ = self.tx.send(Command::State(state));
    }

    /// Off clears what Discord shows and stops connecting until switched on.
    pub fn set_enabled(&self, enabled: bool) {
        let _ = self.tx.send(Command::Enable(enabled));
    }
}

pub fn spawn_discord_rpc(enabled: bool) -> DiscordRpc {
    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();

    tokio::spawn(async move {
        let app_id =
            std::env::var("DISCORD_APP_ID").unwrap_or_else(|_| DEFAULT_DISCORD_APP_ID.to_string());

        let mut client: Option<DiscordIpcClient> = None;
        let mut connected = false;
        let mut enabled = enabled;
        let mut current_state = DiscordRpcState::Launcher { server_name: None };

        let launcher_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut check_interval = tokio::time::interval(Duration::from_secs(4));

        loop {
            tokio::select! {
                Some(command) = rx.recv() => {
                    let new_state = match command {
                        Command::State(state) => state,
                        Command::Enable(on) => {
                            enabled = on;
                            if !on && connected {
                                if let Some(ref mut c) = client {
                                    let _ = c.clear_activity();
                                    let _ = c.close();
                                }
                                connected = false;
                            }
                            continue;
                        }
                    };
                    let changed = current_state != new_state;
                    current_state = new_state;

                    if enabled && (connected || changed) {
                        if !connected {
                            try_connect(&app_id, &mut client, &mut connected);
                        }
                        if connected {
                            if let Some(ref mut c) = client {
                                if let Err(e) = update_activity(c, &current_state, launcher_start_time) {
                                    warn!("Discord RPC update failed: {e}");
                                    connected = false;
                                }
                            }
                        }
                    }
                }
                _ = check_interval.tick() => {
                    if enabled && !connected {
                        try_connect(&app_id, &mut client, &mut connected);
                        if connected {
                            if let Some(ref mut c) = client {
                                if let Err(e) = update_activity(c, &current_state, launcher_start_time) {
                                    warn!("Discord RPC initial activity failed: {e}");
                                    connected = false;
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    DiscordRpc { tx }
}

fn try_connect(app_id: &str, client: &mut Option<DiscordIpcClient>, connected: &mut bool) {
    if client.is_none() {
        if let Ok(c) = DiscordIpcClient::new(app_id) {
            *client = Some(c);
        }
    }
    if let Some(ref mut c) = client {
        match c.connect() {
            Ok(_) => {
                *connected = true;
                debug!("Discord RPC connected");
            }
            Err(_) => {
                *connected = false;
            }
        }
    }
}

fn update_activity(
    client: &mut DiscordIpcClient,
    state: &DiscordRpcState,
    launcher_start_time: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut act = activity::Activity::new();
    let assets = activity::Assets::new()
        .large_image("logo")
        .large_text("NORO Launcher");

    // In the player's language: friends reading the status mostly share it.
    // The catalog is the process-wide one the window switches.
    let named = |key: &str, name: &str| {
        let mut args = i18n::FluentArgs::new();
        args.set("name", name.to_string());
        i18n::t_args(key, &args)
    };
    let (details, state_str, start_ts) = match state {
        DiscordRpcState::Launcher { server_name } => {
            let details = i18n::t("rpc-in-launcher");
            let state_str = match server_name {
                Some(name) => named("rpc-server", name),
                None => i18n::t("rpc-picking-server"),
            };
            (details, state_str, launcher_start_time)
        }
        DiscordRpcState::GameLoading { server_name } => (
            named("rpc-starting", server_name),
            i18n::t("rpc-loading"),
            launcher_start_time,
        ),
        DiscordRpcState::GameMenu {
            server_name,
            start_timestamp,
        } => (
            i18n::t("rpc-main-menu"),
            server_name.clone(),
            *start_timestamp as i64,
        ),
        DiscordRpcState::GamePlaying {
            server_name,
            online_current,
            online_max,
            start_timestamp,
        } => {
            let details = named("rpc-playing", server_name);
            let state_str = match (online_current, online_max) {
                (Some(cur), Some(max)) => {
                    let mut args = i18n::FluentArgs::new();
                    args.set("current", cur.to_string());
                    args.set("max", max.to_string());
                    i18n::t_args("rpc-online", &args)
                }
                _ => i18n::t("rpc-on-server"),
            };
            (details, state_str, *start_timestamp as i64)
        }
    };

    act = act
        .details(&details)
        .state(&state_str)
        .assets(assets)
        .timestamps(activity::Timestamps::new().start(start_ts));

    client.set_activity(act)?;
    Ok(())
}
