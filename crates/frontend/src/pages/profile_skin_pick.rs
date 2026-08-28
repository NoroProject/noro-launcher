//! Выбор файла скина через нативный диалог платформы.
//!
//! Раньше здесь запускался `osascript` с `choose file`, и это ломалось трижды:
//! на Windows и Linux такого бинарника нет вовсе; на macOS панель принадлежала
//! стороннему процессу и всплывала позади окна лаунчера; а `Command::output()`
//! держал поток отрисовки GPUI до закрытия диалога. Все три случая выглядели
//! одинаково — «кнопка не работает», — потому что любой из них сводился к
//! одному и тому же тосту «No valid skin selected».

use crate::state::{LauncherUI, SavedSkinPreset, Toast};
use gpui::{AppContext, ClickEvent, Context, PathPromptOptions, Window};
use i18n::t;
use schema::NotifLevel;
use std::path::Path;

/// Тот же потолок, что и у мастера: скин крупнее он не примет.
const MAX_SKIN_BYTES: usize = 256 * 1024;
const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub fn on_upload_click(
    _this: &mut LauncherUI,
    _e: &ClickEvent,
    _w: &mut Window,
    cx: &mut Context<LauncherUI>,
) {
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Select skin".into()),
    });

    cx.spawn(async move |this, cx| {
        let path = match picked.await {
            Ok(Ok(Some(paths))) => match paths.into_iter().next() {
                Some(path) => path,
                // Диалог отдал пустой список — выбирать нечего, и это не сбой.
                None => return,
            },
            // Игрок закрыл диалог. Это решение игрока, а не ошибка: молчим.
            Ok(Ok(None)) | Err(_) => return,
            Ok(Err(e)) => {
                report(
                    &this,
                    cx,
                    format!("{}: {e}", t("profile-skin-picker-failed")),
                );
                return;
            }
        };

        // Чтение уводим с потока отрисовки: файл может лежать на сетевом диске.
        let loaded = cx.background_spawn(async move { read_skin(&path) }).await;

        match loaded {
            Ok(skin) => {
                let _ = this.update(cx, |this, cx| {
                    this.upload_skin(skin.bytes.clone());
                    this.custom_presets.push(SavedSkinPreset {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: skin.name,
                        bytes: skin.bytes,
                        preview: this.skin_preview.clone(),
                    });
                    cx.notify();
                });
            }
            Err(message) => report(&this, cx, message),
        }
    })
    .detach();
}

#[derive(Debug)]
struct Skin {
    name: String,
    bytes: Vec<u8>,
}

/// Каждый отказ называет свою причину.
///
/// Один общий «No valid skin selected» на все случаи не давал игроку понять,
/// что делать: не тот файл, слишком большой или мы вовсе не смогли его открыть.
fn read_skin(path: &Path) -> Result<Skin, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("{}: {e}", t("profile-skin-unreadable")))?;

    if bytes.len() < PNG_MAGIC.len() || &bytes[..PNG_MAGIC.len()] != PNG_MAGIC {
        return Err(t("profile-skin-not-png"));
    }
    if bytes.len() > MAX_SKIN_BYTES {
        return Err(t("profile-skin-too-large"));
    }

    Ok(Skin {
        name: skin_name(path),
        bytes,
    })
}

fn skin_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| t("profile-skin-untitled"))
}

fn report(this: &gpui::WeakEntity<LauncherUI>, cx: &mut gpui::AsyncApp, text: String) {
    let _ = this.update(cx, |this, cx| {
        this.toast = Some(Toast {
            text,
            level: NotifLevel::Warning,
        });
        cx.notify();
    });
}

/// Расширение подсказывает, но не решает: содержимое проверяется по сигнатуре.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_a_png_is_named_as_such() {
        let dir = std::env::temp_dir().join("noro-skin-pick-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not-a-skin.png");
        std::fs::write(&path, b"just text, but the extension lies").unwrap();

        let err = read_skin(&path).expect_err("не PNG");
        assert_eq!(err, t("profile-skin-not-png"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_missing_file_reports_reading_and_not_format() {
        let err = read_skin(Path::new("/nope/definitely/missing.png")).expect_err("нет файла");
        assert!(
            err.starts_with(&t("profile-skin-unreadable")),
            "причина должна быть про чтение: {err}"
        );
    }

    #[test]
    fn an_oversized_png_is_rejected_by_size() {
        let dir = std::env::temp_dir().join("noro-skin-pick-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("huge.png");
        let mut bytes = PNG_MAGIC.to_vec();
        bytes.resize(MAX_SKIN_BYTES + 1, 0);
        std::fs::write(&path, &bytes).unwrap();

        assert_eq!(
            read_skin(&path).expect_err("слишком большой"),
            t("profile-skin-too-large")
        );
        std::fs::remove_file(&path).ok();
    }
}
