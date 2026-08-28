//! Picking a skin file through the platform's native dialog.

use crate::state::{LauncherUI, SavedSkinPreset, Toast};
use gpui::{AppContext, ClickEvent, Context, PathPromptOptions, Window};
use i18n::t;
use schema::NotifLevel;
use std::path::Path;

/// Same ceiling as the master, which rejects anything larger.
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
                // An empty list from the dialog: nothing was picked.
                None => return,
            },
            // Cancelled by the player, or the prompt was dropped. Say nothing.
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

        // Off the render thread: the file may sit on a network drive.
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

/// Each rejection names its own reason: unreadable, not a PNG, or too large.
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

/// The extension is a hint; the content is what gets checked, by signature.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_a_png_is_named_as_such() {
        let dir = std::env::temp_dir().join("noro-skin-pick-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not-a-skin.png");
        std::fs::write(&path, b"just text, but the extension lies").unwrap();

        let err = read_skin(&path).expect_err("not a PNG");
        assert_eq!(err, t("profile-skin-not-png"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_missing_file_reports_reading_and_not_format() {
        let err = read_skin(Path::new("/nope/definitely/missing.png")).expect_err("no such file");
        assert!(
            err.starts_with(&t("profile-skin-unreadable")),
            "the reason should be about reading: {err}"
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
            read_skin(&path).expect_err("too large"),
            t("profile-skin-too-large")
        );
        std::fs::remove_file(&path).ok();
    }
}
