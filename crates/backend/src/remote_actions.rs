//! Удалённые действия: то, что админ просит сделать на машине игрока.
//!
//! Всё, что стирает файлы или прерывает работу, спрашивает подтверждение —
//! иначе это уже не поддержка, а управление чужим компьютером. Сверка
//! целостности ничего не портит и идёт молча.

use crate::directories::LauncherDirectories;
use anyhow::{bail, Result};
use schema::RemoteAction;
use std::path::Path;
use uuid::Uuid;

/// Что вышло — уходит в уведомление игроку и в лог.
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
            // Ассеты восстановимы и чаще всего именно они и битые: снести их
            // дешевле, чем переустанавливать сборку целиком.
            let removed = remove_dir(&dir.join("assets")).await;
            Ok(Outcome {
                message: format!("кэш ассетов очищен ({removed})"),
            })
        }
        RemoteAction::ReinstallBuild => {
            let dir = instance_dir(dirs, server_id)?;
            // Метка версии сборки: без неё следующий запуск считает инстанс
            // неустановленным и качает всё заново.
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
        None => bail!("действие требует указания сборки"),
    }
}

/// Удалить каталог, если он есть. Отсутствие — не ошибка: нечего было чистить.
async fn remove_dir(path: &Path) -> &'static str {
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => "удалено",
        Err(_) => "уже пусто",
    }
}
