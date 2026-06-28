//! Serializable snapshot of player progress (journal + settings).
//!
//! The engine's progress lives in memory; [`crate::engine::Engine::snapshot`]
//! exports it into a `SaveState` and [`crate::engine::Engine::restore`] loads one
//! back. Frontends persist this between sessions (the desktop app to a file, the
//! web app to `localStorage`) so the notebook survives a restart.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn default_interval() -> u32 {
    crate::engine::DEFAULT_INTERVAL_MS
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveState {
    /// Journal "Clues" — recap entries, in order.
    #[serde(default)]
    pub recap: Vec<String>,
    /// Discovered passwords.
    #[serde(default)]
    pub discovered: Vec<String>,
    /// Topics asked, grouped by location id.
    #[serde(default)]
    pub asked: BTreeMap<String, Vec<String>>,
    /// Decoded hex -> plaintext pairs.
    #[serde(default)]
    pub decoded: Vec<(String, String)>,
    /// People (locations) met.
    #[serde(default)]
    pub visited: Vec<String>,
    /// Skip-mode (instant text) setting.
    #[serde(default)]
    pub skip: bool,
    /// Reveal pacing in ms.
    #[serde(default = "default_interval")]
    pub interval_ms: u32,
    /// Whether the player has learned Jaffar's lines are hex (auto-translate on).
    #[serde(default)]
    pub decoding_unlocked: bool,
    /// Accessibility: disable motion-heavy effects (flash/glitch).
    #[serde(default)]
    pub reduced_effects: bool,
}

impl Default for SaveState {
    fn default() -> Self {
        SaveState {
            recap: Vec::new(),
            discovered: Vec::new(),
            asked: BTreeMap::new(),
            decoded: Vec::new(),
            visited: Vec::new(),
            skip: false,
            interval_ms: default_interval(), // match the serde default (not u32's 0)
            decoding_unlocked: false,
            reduced_effects: false,
        }
    }
}

impl SaveState {
    /// Serialize to pretty JSON for on-disk / localStorage storage.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Parse from JSON; returns `None` on malformed input (treated as no save).
    pub fn from_json(s: &str) -> Option<SaveState> {
        serde_json::from_str(s).ok()
    }
}
