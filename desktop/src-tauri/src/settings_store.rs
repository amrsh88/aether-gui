//! Settings persistence.
//!
//! Stored as JSON next to the Aether identity files. Unknown or missing fields fall
//! back to their defaults, so a settings file written by an older build never stops
//! the app from starting — the alternative is a user who has to delete a file they
//! do not know exists.

use std::io;
use std::path::PathBuf;

use crate::model::Settings;

fn settings_path() -> Option<PathBuf> {
    crate::engine::config_dir().map(|dir| dir.join("settings.json"))
}

/// Read the stored settings, or `None` when there are none to read.
pub fn load() -> Option<Settings> {
    let path = settings_path()?;
    let raw = std::fs::read_to_string(&path).ok()?;

    match serde_json::from_str(&raw) {
        Ok(settings) => Some(settings),
        Err(e) => {
            // Report and start fresh rather than refusing to launch.
            log::warn!(
                "[!] ignoring unreadable settings at {}: {e}",
                path.display()
            );
            None
        }
    }
}

/// Write the settings, creating the directory if needed.
pub fn save(settings: &Settings) -> io::Result<()> {
    let path =
        settings_path().ok_or_else(|| io::Error::other("could not resolve a config directory"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| io::Error::other(format!("could not serialise settings: {e}")))?;

    // Write to a temporary file and rename, so a crash mid-write cannot leave a
    // truncated file that fails to parse on next launch.
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, json)?;
    std::fs::rename(&temporary, &path)?;

    Ok(())
}
