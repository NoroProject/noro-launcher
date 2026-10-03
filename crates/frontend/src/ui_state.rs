//! What the window remembers between runs: where it stood, how big it was,
//! and which server was open. It used to open centred on the first server
//! every time, whatever the player had arranged.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct UiState {
    #[serde(default)]
    pub window: Option<SavedBounds>,
    #[serde(default)]
    pub maximized: bool,
    #[serde(default)]
    pub last_server: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct SavedBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn path() -> Option<PathBuf> {
    Some(
        dirs::data_dir()?
            .join(schema::launcher_dir_name())
            .join("ui-state.json"),
    )
}

/// Defaults when there is nothing saved or it doesn't read: none of this is
/// worth an error.
pub fn load() -> UiState {
    path()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(state: &UiState) {
    let (Some(path), Ok(bytes)) = (path(), serde_json::to_vec_pretty(state)) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, bytes).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_file_with_fewer_fields_still_reads() {
        let state: UiState = serde_json::from_str(r#"{"maximized": true}"#).unwrap();
        assert!(state.maximized);
        assert_eq!(state.window, None);
        assert_eq!(state.last_server, None);
    }
}
