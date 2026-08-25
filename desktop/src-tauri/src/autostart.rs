//! Launch-at-sign-in, via the per-user `Run` key.
//!
//! The `Run` key is used rather than a scheduled task because it needs no elevation
//! to write. The trade-off is that Windows starts `Run` entries unelevated, so a
//! Full Tunnel session still has to be started manually or the app has to be
//! elevated by other means — the UI says so rather than pretending otherwise.

use std::io;

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(windows)]
const VALUE_NAME: &str = "AetherGUI";

/// Add or remove the sign-in entry.
#[cfg(windows)]
pub fn set(enabled: bool) -> io::Result<()> {
    use windows_registry::CURRENT_USER;

    let key = CURRENT_USER
        .create(RUN_KEY)
        .map_err(|e| io::Error::other(format!("could not open the Run key: {e}")))?;

    if !enabled {
        // A missing value is the desired end state, so absence is not an error.
        let _ = key.remove_value(VALUE_NAME);
        log::info!("[+] start-with-Windows disabled");
        return Ok(());
    }

    let exe = std::env::current_exe()
        .map_err(|e| io::Error::other(format!("could not resolve our own path: {e}")))?;

    // Quoted because Program Files contains a space, and `--minimized` so the app
    // starts in the tray instead of stealing focus at sign-in.
    let command = format!("\"{}\" --minimized", exe.display());

    key.set_string(VALUE_NAME, &command)
        .map_err(|e| io::Error::other(format!("could not write the Run entry: {e}")))?;

    log::info!("[+] start-with-Windows enabled");
    Ok(())
}

/// Whether the sign-in entry is currently present.
#[cfg(windows)]
pub fn is_enabled() -> bool {
    use windows_registry::CURRENT_USER;

    CURRENT_USER
        .open(RUN_KEY)
        .and_then(|key| key.get_string(VALUE_NAME))
        .is_ok()
}

#[cfg(not(windows))]
pub fn set(_enabled: bool) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn is_enabled() -> bool {
    false
}
