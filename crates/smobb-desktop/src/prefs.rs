//! Per-user window preferences (font size, fullscreen), persisted next to the
//! game save. Loading is total: any IO/parse failure falls back to defaults so
//! startup never blocks or panics.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::settings::FONT_DEFAULT;

const PREFS_FILE: &str = "smobb-window.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPrefs {
    pub font_px: u32,
    pub fullscreen: bool,
}

impl Default for WindowPrefs {
    fn default() -> Self {
        Self {
            font_px: FONT_DEFAULT,
            fullscreen: false,
        }
    }
}

/// `~/Library/Application Support/smobb/smobb-window.json` on macOS — same data
/// dir as the save (a bundled `.app` has no meaningful cwd).
pub fn prefs_path() -> PathBuf {
    let dir = dirs::data_dir()
        .map(|d| d.join("smobb"))
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join(PREFS_FILE)
}

pub fn load() -> WindowPrefs {
    fs::read_to_string(prefs_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(p: &WindowPrefs) {
    if let Ok(t) = serde_json::to_string_pretty(p) {
        let _ = fs::write(prefs_path(), t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefs_path_is_named_under_smobb_dir() {
        assert!(prefs_path().ends_with("smobb/smobb-window.json"));
    }

    #[test]
    fn window_prefs_json_round_trips() {
        let p = WindowPrefs {
            font_px: 24,
            fullscreen: true,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: WindowPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn default_is_sane() {
        let d = WindowPrefs::default();
        assert_eq!(d.font_px, FONT_DEFAULT);
        assert!(!d.fullscreen);
    }
}
