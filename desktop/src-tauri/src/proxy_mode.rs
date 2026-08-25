//! Windows system proxy configuration.
//!
//! WinINET keeps its proxy settings in the current user's registry, and every
//! browser plus most CLI tools read them from there. Writing the keys is easy; the
//! parts that matter are restoring exactly what the user had before, and telling
//! WinINET the settings changed so open applications pick them up without a
//! restart.
//!
//! `ProxyServer` is written as `socks=host:port`. That form is what WinINET
//! understands for a SOCKS proxy — the `socks5://` URL syntax used by curl and
//! friends is silently ignored here.

use std::io;

#[cfg(windows)]
use std::net::SocketAddr;

/// Registry path holding the per-user WinINET settings.
#[cfg(windows)]
const SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

/// Hosts that should never go through the tunnel. Local and private ranges are
/// excluded so LAN devices, routers and dev servers stay reachable.
#[cfg(windows)]
const DEFAULT_BYPASS: &str = "<local>;localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;\
172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;\
172.28.*;172.29.*;172.30.*;172.31.*;192.168.*";

/// The user's proxy settings as they were before we touched anything.
///
/// The fields are only read by the Windows implementation below; on other targets
/// this is an inert placeholder so the rest of the app still compiles.
#[derive(Debug, Clone, Default)]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct SavedProxy {
    enabled: Option<u32>,
    server: Option<String>,
    bypass: Option<String>,
    /// True when we actually applied a change and therefore owe a restore.
    applied: bool,
}

/// Point the system proxy at our SOCKS5 listener, returning the previous state.
///
/// Extra bypass entries from the user's settings are appended to the private-range
/// defaults, so "bypass the tunnel for these" works in proxy mode too.
#[cfg(windows)]
pub fn apply(socks: SocketAddr, extra_bypass: &str) -> io::Result<SavedProxy> {
    use windows_registry::CURRENT_USER;

    let key = CURRENT_USER
        .create(SETTINGS_KEY)
        .map_err(|e| io::Error::other(format!("could not open the proxy settings key: {e}")))?;

    // Read the old values first; without these a restore would be guesswork.
    let saved = SavedProxy {
        enabled: key.get_u32("ProxyEnable").ok(),
        server: key.get_string("ProxyServer").ok(),
        bypass: key.get_string("ProxyOverride").ok(),
        applied: false,
    };

    let bypass = build_bypass(extra_bypass);

    key.set_u32("ProxyEnable", 1)
        .map_err(|e| io::Error::other(format!("could not enable the proxy: {e}")))?;
    key.set_string("ProxyServer", format!("socks={socks}"))
        .map_err(|e| io::Error::other(format!("could not set the proxy address: {e}")))?;
    key.set_string("ProxyOverride", &bypass)
        .map_err(|e| io::Error::other(format!("could not set the proxy bypass list: {e}")))?;

    notify_wininet();

    log::info!("[+] system proxy set to socks={socks}");

    Ok(SavedProxy {
        applied: true,
        ..saved
    })
}

/// Put the user's original settings back.
///
/// Every step is best-effort: leaving the proxy pointed at a dead listener would
/// break the user's internet, so we push through failures rather than aborting on
/// the first one.
#[cfg(windows)]
pub fn restore(saved: &mut SavedProxy) {
    use windows_registry::CURRENT_USER;

    if !saved.applied {
        return;
    }
    saved.applied = false;

    let Ok(key) = CURRENT_USER.create(SETTINGS_KEY) else {
        log::error!("[-] could not reopen the proxy settings key; the system proxy is still set");
        return;
    };

    match saved.enabled {
        // Restore whatever the flag was, including an explicit 0.
        Some(value) => {
            if let Err(e) = key.set_u32("ProxyEnable", value) {
                log::warn!("[!] could not restore ProxyEnable: {e}");
            }
        }
        // The value did not exist before, so turning it off is the honest
        // equivalent — removing it entirely also reads as "disabled" to WinINET.
        None => {
            if let Err(e) = key.set_u32("ProxyEnable", 0) {
                log::warn!("[!] could not clear ProxyEnable: {e}");
            }
        }
    }

    restore_string(&key, "ProxyServer", saved.server.as_deref());
    restore_string(&key, "ProxyOverride", saved.bypass.as_deref());

    notify_wininet();
    log::info!("[+] system proxy restored");
}

#[cfg(windows)]
fn restore_string(key: &windows_registry::Key, name: &str, value: Option<&str>) {
    let result = match value {
        Some(previous) => key.set_string(name, previous),
        None => key.remove_value(name),
    };
    if let Err(e) = result {
        log::warn!("[!] could not restore {name}: {e}");
    }
}

/// Merge the private-range defaults with the user's own bypass entries.
#[cfg(windows)]
fn build_bypass(extra: &str) -> String {
    let mut bypass = String::from(DEFAULT_BYPASS);

    for entry in extra.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        // `private` is Aether's own shorthand for the ranges already covered.
        if entry.eq_ignore_ascii_case("private") {
            continue;
        }
        bypass.push(';');
        bypass.push_str(entry);
    }

    bypass
}

/// Tell WinINET its settings changed, so running applications reload them.
///
/// Both calls are needed: `SETTINGS_CHANGED` invalidates the cached config and
/// `REFRESH` makes WinINET re-read the registry. Skipping this leaves already-open
/// browsers using the old proxy until they restart.
#[cfg(windows)]
fn notify_wininet() {
    use windows_sys::Win32::Networking::WinInet::{
        InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
    };

    unsafe {
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null_mut(),
            0,
        );
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_REFRESH,
            std::ptr::null_mut(),
            0,
        );
    }
}

#[cfg(not(windows))]
pub fn apply(_socks: std::net::SocketAddr, _extra_bypass: &str) -> io::Result<SavedProxy> {
    Err(io::Error::other(
        "System Proxy mode is only implemented on Windows",
    ))
}

#[cfg(not(windows))]
pub fn restore(_saved: &mut SavedProxy) {}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn user_bypass_entries_are_appended_to_the_defaults() {
        let bypass = build_bypass("*.ir, bank.example.com");
        assert!(bypass.starts_with("<local>"));
        assert!(bypass.ends_with(";*.ir;bank.example.com"));
    }

    #[test]
    fn the_private_shorthand_is_not_duplicated() {
        let bypass = build_bypass("private");
        assert_eq!(bypass, DEFAULT_BYPASS);
    }

    #[test]
    fn empty_entries_are_ignored() {
        assert_eq!(build_bypass(" , ,"), DEFAULT_BYPASS);
    }
}
