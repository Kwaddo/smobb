use smobb_core::save::SaveState;
use smobb_tui_core::app::App;
use std::fs;
use std::path::PathBuf;

const SAVE_FILE: &str = "smobb-save.json";

/// Per-user data path for the desktop save, e.g.
/// `~/Library/Application Support/smobb/smobb-save.json` on macOS. A bundled
/// `.app` has no meaningful cwd, so we never write relative to it.
pub fn save_path() -> PathBuf {
    let dir = dirs::data_dir()
        .map(|d| d.join("smobb"))
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join(SAVE_FILE)
}

pub fn load_save() -> Option<SaveState> {
    let txt = fs::read_to_string(save_path()).ok()?;
    SaveState::from_json(&txt)
}

pub fn write_save(app: &App) {
    let _ = fs::write(save_path(), app.snapshot().to_json());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_path_is_under_smobb_dir_and_named() {
        let p = save_path();
        assert!(p.ends_with("smobb/smobb-save.json"));
    }
}
