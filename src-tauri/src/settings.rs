use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Persistent app settings. Stored as JSON in the OS app-config directory so a
/// user's choice (auto-connect flag + monitored game executables) survives
/// between launches.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub auto_connect_on_game: bool,
    #[serde(default)]
    pub monitored_games: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_cues: Option<bool>,
}

impl AppSettings {
    /// Load settings from disk, falling back to defaults if the file is missing
    /// or unreadable (so a corrupt file never prevents the app from running).
    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(e) => {
                    println!("settings: failed to parse {path:?}: {e}; using defaults");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Persist the current settings to disk. Failures are logged but not fatal.
    pub fn save(&self, path: &std::path::Path) {
        let json = match serde_json::to_string_pretty(self) {
            Ok(j) => j,
            Err(e) => {
                println!("settings: failed to serialize: {e}");
                return;
            }
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, json) {
            println!("settings: failed to save {path:?}: {e}");
        }
    }
}

/// Normalize a game name for matching against process names (lowercase, strip a
/// trailing `.exe`).
pub fn normalize_game(name: &str) -> String {
    let trimmed = name.trim().to_lowercase();
    trimmed.trim_end_matches(".exe").to_string()
}

/// Shared, thread-safe holder for the app settings. Guarded in a mutex so both
/// the Tauri command handlers and the background watcher can read/update it.
pub type SharedSettings = Arc<Mutex<AppSettings>>;
