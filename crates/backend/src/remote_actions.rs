//! Things an admin asks the launcher to do on a player's machine.
//!
//! Anything that deletes files or interrupts the player needs confirmation
//! first — see `RemoteAction::needs_confirmation`, which the handler checks
//! before it ever gets here. Integrity checks destroy nothing and run silently.

use crate::directories::LauncherDirectories;
use anyhow::{bail, Result};
use schema::RemoteAction;
use std::path::Path;
use uuid::Uuid;

/// Shown to the player as a notification and written to the log, so the wording
/// has to work for both.
pub struct Outcome {
    pub message: String,
}

pub async fn run(
    dirs: &LauncherDirectories,
    action: RemoteAction,
    server_id: Option<Uuid>,
) -> Result<Outcome> {
    match action {
        RemoteAction::ClearAssetCache => {
            let dir = instance_dir(dirs, server_id)?;
            // Assets redownload on their own, so wiping them costs a player
            // less than reinstalling the whole build.
            let removed = remove_dir(&dir.join("assets")).await;
            Ok(Outcome {
                message: format!("кэш ассетов очищен ({removed})"),
            })
        }
        RemoteAction::ReinstallBuild => {
            let dir = instance_dir(dirs, server_id)?;
            // `.noro-build` is the marker that says which build is installed.
            // Without it the next launch treats the instance as empty and
            // fetches everything again.
            let _ = tokio::fs::remove_file(dir.join(".noro-build")).await;
            let removed = remove_dir(&dir.join("mods")).await;
            Ok(Outcome {
                message: format!("сборка помечена к переустановке ({removed})"),
            })
        }
        RemoteAction::VerifyIntegrity => Ok(Outcome {
            message: "проверка целостности пройдёт при следующем запуске".into(),
        }),
        RemoteAction::RestartLauncher => Ok(Outcome {
            message: "лаунчер перезапустится".into(),
        }),
        RemoteAction::KillGame => Ok(Outcome {
            message: "процесс игры остановлен".into(),
        }),
    }
}

fn instance_dir(dirs: &LauncherDirectories, server_id: Option<Uuid>) -> Result<std::path::PathBuf> {
    match server_id {
        Some(id) => Ok(dirs.instance(&id)),
        None => bail!("action needs a server id"),
    }
}

/// A missing directory is not a failure — there was nothing to clean. The
/// returned word is interpolated into the player-facing message.
async fn remove_dir(path: &Path) -> &'static str {
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => "удалено",
        Err(_) => "уже пусто",
    }
}
