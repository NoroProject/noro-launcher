//! Discord Rich Presence модуль для NORO Launcher.

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

#[derive(Clone)]
pub struct DiscordRpc {
    tx: mpsc::UnboundedSender<DiscordRpcState>,
}

impl DiscordRpc {
    pub fn update(&self, state: DiscordRpcState) {
        let _ = self.tx.send(state);
    }
}

pub fn spawn_discord_rpc() -> DiscordRpc {
    let (tx, mut rx) = mpsc::unbounded_channel::<DiscordRpcState>();

    tokio::spawn(async move {
        let app_id =
            std::env::var("DISCORD_APP_ID").unwrap_or_else(|_| DEFAULT_DISCORD_APP_ID.to_string());

        let mut client: Option<DiscordIpcClient> = None;
        let mut connected = false;
        let mut current_state = DiscordRpcState::Launcher { server_name: None };

        let launcher_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut check_interval = tokio::time::interval(Duration::from_secs(4));

        loop {
            tokio::select! {
                Some(new_state) = rx.recv() => {
                    let changed = current_state != new_state;
                    current_state = new_state;

                    if connected || changed {
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
                    if !connected {
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

    let (details, state_str, start_ts) = match state {
        DiscordRpcState::Launcher { server_name } => {
            let details = "В лаунчере NORO".to_string();
            let state_str = match server_name {
                Some(name) => format!("Сервер: {name}"),
                None => "Выбирает сервер".to_string(),
            };
            (details, state_str, launcher_start_time)
        }
        DiscordRpcState::GameLoading { server_name } => (
            format!("Запуск: {server_name}"),
            "Загрузка ресурсов...".to_string(),
            launcher_start_time,
        ),
        DiscordRpcState::GameMenu {
            server_name,
            start_timestamp,
        } => (
            "В главном меню".to_string(),
            server_name.clone(),
            *start_timestamp as i64,
        ),
        DiscordRpcState::GamePlaying {
            server_name,
            online_current,
            online_max,
            start_timestamp,
        } => {
            let details = format!("Играет на {server_name}");
            let state_str = match (online_current, online_max) {
                (Some(cur), Some(max)) => format!("Онлайн: {cur}/{max}"),
                _ => "На сервере".to_string(),
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
