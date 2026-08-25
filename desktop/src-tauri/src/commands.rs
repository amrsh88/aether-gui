//! Application state and the Tauri command surface.
//!
//! One mutex guards the whole connection state. That is deliberate: connect and
//! disconnect are user-driven and seconds apart, and every subtle bug in this kind
//! of code comes from a half-applied state — the proxy set but the tunnel down, or
//! routes installed with nothing behind them. Serialising the transitions makes
//! those states unrepresentable.
//!
//! Teardown always runs in reverse order of setup, and always runs to completion
//! even when a step fails, so a failed connect never leaves the machine's
//! networking altered.

use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::engine::{self, Reporter, ScanLog, Session};
use crate::model::{events, PeerInfo, Phase, ProgressEvent, ScannedPeer, Settings, TunnelMode};
use crate::tun::{self, Tunnel};
use crate::{autostart, proxy_mode, relay, settings_store};

/// Whatever is currently active. All fields absent when disconnected.
#[derive(Default)]
struct Active {
    session: Option<Session>,
    tunnel: Option<Tunnel>,
    proxy: Option<proxy_mode::SavedProxy>,
    relay: Option<relay::Relay>,
    stats_cancel: Option<CancellationToken>,
    watchdog: Option<tokio::task::JoinHandle<()>>,
}

/// Shared behind an `Arc` rather than reached through `State` so background tasks
/// can own a handle without borrowing from the `AppHandle`.
pub struct Shared {
    active: Mutex<Active>,
    scan_log: Arc<ScanLog>,
    /// Held for the duration of a connect attempt, so a second one is refused
    /// rather than queued behind it.
    busy: Mutex<()>,
}

pub struct AppState(Arc<Shared>);

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(Shared {
            active: Mutex::new(Active::default()),
            scan_log: Arc::new(ScanLog::default()),
            busy: Mutex::new(()),
        }))
    }

    fn shared(&self) -> Arc<Shared> {
        self.0.clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Forwards engine progress to the frontend.
struct EventReporter {
    app: AppHandle,
    scan_log: Arc<ScanLog>,
}

impl Reporter for EventReporter {
    fn progress(&self, phase: Phase, detail: String) {
        let _ = self
            .app
            .emit(events::PROGRESS, ProgressEvent::new(phase, detail));
    }

    fn peer_found(&self, address: std::net::SocketAddr, rtt_ms: u64, ok: bool) {
        self.scan_log.record(address, rtt_ms, ok);
    }

    fn log(&self, line: String) {
        log::info!("{line}");
        let _ = self.app.emit(events::LOG, line);
    }
}

fn emit_progress(app: &AppHandle, event: ProgressEvent) {
    let _ = app.emit(events::PROGRESS, event);
}

fn emit_log(app: &AppHandle, line: impl Into<String>) {
    let line = line.into();
    log::info!("{line}");
    let _ = app.emit(events::LOG, line);
}

#[tauri::command]
pub async fn connect(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    let shared = state.shared();

    // Reject an overlapping attempt instead of queueing it; a queued connect would
    // fire after the user had already given up and pressed cancel.
    let _busy = shared
        .busy
        .try_lock()
        .map_err(|_| "A connection attempt is already running".to_string())?;

    if shared.active.lock().await.session.is_some() {
        return Err("Already connected".into());
    }

    // Fail before touching anything if the chosen mode is unavailable.
    if matches!(settings.mode, TunnelMode::Tun) && !tun::is_elevated() {
        return Err(
            "Full Tunnel mode needs administrator rights. Restart Aether GUI as administrator, \
             or switch to System Proxy mode."
                .into(),
        );
    }

    let reporter: Arc<dyn Reporter> = Arc::new(EventReporter {
        app: app.clone(),
        scan_log: shared.scan_log.clone(),
    });

    // 1. Core tunnel. Everything else layers on top, so a failure here needs no
    //    cleanup at all.
    let session = engine::connect(&settings, reporter, shared.scan_log.clone())
        .await
        .inspect_err(|e| {
            emit_progress(&app, ProgressEvent::new(Phase::Error, e.clone()));
        })?;

    let socks = session.socks_addr();
    let peer = session.peer_addr();
    let protocol = session.protocol();
    let rtt_ms = session.rtt_ms();

    let mut active = shared.active.lock().await;

    // 2. System integration. From here on a failure must roll the whole thing back:
    //    a live tunnel nobody is routed through is just a hidden leak.
    emit_progress(
        &app,
        ProgressEvent::new(Phase::Routing, "Applying system configuration"),
    );

    let counters = match settings.mode {
        TunnelMode::Tun => {
            let dns = parse_dns(&settings);
            match Tunnel::start(socks, peer, &dns, settings.kill_switch).await {
                Ok(tunnel) => {
                    let counters = tunnel.counters();
                    active.tunnel = Some(tunnel);
                    counters
                }
                Err(e) => {
                    let message = format!("Could not start the full tunnel: {e}");
                    session.shutdown().await;
                    emit_progress(&app, ProgressEvent::new(Phase::Error, message.clone()));
                    return Err(message);
                }
            }
        }

        TunnelMode::Proxy => {
            // A counting relay goes in front of the core and Windows is pointed at
            // the relay, so proxy mode reports real throughput instead of nothing.
            let relay_listen = std::net::SocketAddr::new(socks.ip(), 0);
            let relay = match relay::Relay::start(relay_listen, socks).await {
                Ok(relay) => relay,
                Err(e) => {
                    let message = format!("Could not start the proxy relay: {e}");
                    session.shutdown().await;
                    emit_progress(&app, ProgressEvent::new(Phase::Error, message.clone()));
                    return Err(message);
                }
            };

            match proxy_mode::apply(relay.listen_addr(), &settings.route_direct) {
                Ok(saved) => {
                    let counters = relay.counters();
                    active.proxy = Some(saved);
                    active.relay = Some(relay);
                    counters
                }
                Err(e) => {
                    let message = format!("Could not set the system proxy: {e}");
                    relay.shutdown().await;
                    session.shutdown().await;
                    emit_progress(&app, ProgressEvent::new(Phase::Error, message.clone()));
                    return Err(message);
                }
            }
        }
    };

    // 3. Meters and the watchdog.
    let stats_cancel = CancellationToken::new();
    crate::stats::spawn(app.clone(), counters, stats_cancel.clone());
    active.stats_cancel = Some(stats_cancel);

    active.session = Some(session);
    active.watchdog = Some(spawn_watchdog(app.clone(), shared.clone()));

    drop(active);

    if let Err(e) = settings_store::save(&settings) {
        log::warn!("[!] could not save settings: {e}");
    }
    if let Err(e) = autostart::set(settings.start_with_windows) {
        log::warn!("[!] could not update the start-with-Windows entry: {e}");
    }

    emit_progress(
        &app,
        ProgressEvent::new(Phase::Connected, "Tunnel is up").with_peer(PeerInfo {
            address: peer.to_string(),
            rtt_ms,
            protocol,
        }),
    );
    emit_log(&app, format!("[+] connected through {peer}"));

    Ok(())
}

#[tauri::command]
pub async fn disconnect(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    emit_progress(
        &app,
        ProgressEvent::new(Phase::Stopping, "Restoring system settings"),
    );

    let shared = state.shared();
    teardown(&shared).await;

    emit_log(&app, "[+] disconnected");
    emit_progress(&app, ProgressEvent::new(Phase::Idle, String::new()));
    Ok(())
}

/// Undo everything, in reverse order of setup, ignoring individual failures.
///
/// System integration comes off before the tunnel, so traffic is never pointed at a
/// listener that has already gone away.
async fn teardown(shared: &Shared) {
    let mut active = shared.active.lock().await;
    teardown_locked(&mut active).await;
}

/// The teardown body, for callers that already hold the lock.
async fn teardown_locked(active: &mut Active) {
    if let Some(watchdog) = active.watchdog.take() {
        watchdog.abort();
    }
    if let Some(cancel) = active.stats_cancel.take() {
        cancel.cancel();
    }
    if let Some(mut proxy) = active.proxy.take() {
        proxy_mode::restore(&mut proxy);
    }
    if let Some(relay) = active.relay.take() {
        relay.shutdown().await;
    }
    if let Some(tunnel) = active.tunnel.take() {
        tunnel.shutdown().await;
    }
    if let Some(session) = active.session.take() {
        session.shutdown().await;
    }
}

/// Watch for the tunnel dying on its own, and clean up if it does.
///
/// Without this a dropped tunnel would leave the system proxy or the route table
/// pointing at a dead listener while the UI still claimed to be connected.
fn spawn_watchdog(app: AppHandle, shared: Arc<Shared>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(2);

        loop {
            tokio::time::sleep(interval).await;

            let mut active = shared.active.lock().await;

            let session_dead = active.session.as_ref().is_some_and(|s| !s.is_running());
            let tunnel_dead = active.tunnel.as_ref().is_some_and(|t| !t.is_running());

            if !session_dead && !tunnel_dead {
                continue;
            }

            let reason = if session_dead {
                "the tunnel dropped"
            } else {
                "packet forwarding stopped"
            };
            log::warn!("[!] {reason}; cleaning up");

            // Clear our own handle first so teardown does not abort this task while
            // it is still running.
            active.watchdog = None;
            teardown_locked(&mut active).await;
            drop(active);

            emit_progress(
                &app,
                ProgressEvent::new(Phase::Error, format!("Disconnected because {reason}")),
            );
            break;
        }
    })
}

fn parse_dns(settings: &Settings) -> Vec<std::net::IpAddr> {
    settings
        .dns_list()
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect()
}

#[tauri::command]
pub fn load_settings() -> Option<Settings> {
    let mut settings = settings_store::load()?;

    // The registry is the truth for this one: the user may have removed the entry
    // from Task Manager's Startup tab, and the toggle should reflect that rather
    // than whatever we last wrote to disk.
    settings.start_with_windows = autostart::is_enabled();

    Some(settings)
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<(), String> {
    settings_store::save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scanned_peers(state: State<'_, AppState>) -> Vec<ScannedPeer> {
    state.0.scan_log.snapshot()
}

#[tauri::command]
pub fn is_elevated() -> bool {
    tun::is_elevated()
}

#[tauri::command]
pub fn core_version() -> String {
    engine::core_version().to_string()
}

/// Open one of the project's own links in the user's browser.
///
/// Only the two URLs the About page shows are permitted. Handing an arbitrary
/// frontend-supplied string to the shell would be a way to launch anything at all
/// if the webview were ever compromised.
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    const ALLOWED: [&str; 2] = [
        "https://github.com/CluvexStudio/Aether",
        "https://t.me/net_republic",
    ];

    if !ALLOWED.contains(&url.as_str()) {
        return Err("That link is not allowed".into());
    }

    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn win_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

/// Hide to the tray rather than exiting.
///
/// Closing the window while connected would strand the system proxy and the route
/// table, so the close button hides instead. Quitting goes through the tray menu,
/// which tears everything down first.
#[tauri::command]
pub fn win_hide(window: tauri::Window) {
    let _ = window.hide();
}

/// Restore the machine's networking. Called from the tray before quitting.
pub async fn shutdown_everything(state: &AppState) {
    teardown(&state.0).await;
}
