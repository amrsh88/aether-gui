//! Aether core driver.
//!
//! The core is linked in as a library, not spawned as a child process. That choice
//! is what makes the rest of the app honest: identity provisioning, scanning,
//! verification and the tunnel itself all run in-process, so every progress event
//! the UI shows comes from the code that actually did the work rather than from
//! parsing log lines out of a pipe.
//!
//! The core still reads a lot of its configuration from environment variables, so
//! `configure` translates the GUI's settings into the variables Aether's own CLI
//! would have set. That keeps us on the same code path the upstream binary uses
//! instead of a private one only this app exercises.

use std::net::SocketAddr;
use std::sync::Arc;

use aether::api::{self, Cancel, ProvisionRequest, ScanRequest, Transport, TunnelSpec};
use parking_lot::Mutex;

use crate::model::{Phase, Protocol, ScannedPeer, Settings};

/// Everything a connection attempt reports back as it progresses.
///
/// The engine is deliberately ignorant of Tauri: it emits through this trait so it
/// can be driven from a test or a CLI harness without a window.
pub trait Reporter: Send + Sync + 'static {
    fn progress(&self, phase: Phase, detail: String);
    fn peer_found(&self, address: SocketAddr, rtt_ms: u64, ok: bool);
    fn log(&self, line: String);
}

/// A live Aether session.
pub struct Session {
    cancel: Cancel,
    /// Address applications should point at. This is the core's own listener.
    socks: SocketAddr,
    /// The endpoint we connected through, needed to exclude it from the tunnel.
    peer: SocketAddr,
    /// Round-trip time measured during the scan, shown on the gateway chip.
    rtt_ms: u64,
    protocol: Protocol,
    task: tokio::task::JoinHandle<()>,
}

impl Session {
    pub fn socks_addr(&self) -> SocketAddr {
        self.socks
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer
    }

    pub fn rtt_ms(&self) -> u64 {
        self.rtt_ms
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }

    /// Ask the core to stop and wait briefly for it to unwind.
    pub async fn shutdown(self) {
        self.cancel.cancel();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), self.task).await;
    }
}

/// Peers seen during the most recent scan, for the Stats page.
#[derive(Default)]
pub struct ScanLog(Mutex<Vec<ScannedPeer>>);

impl ScanLog {
    pub fn clear(&self) {
        self.0.lock().clear();
    }

    pub fn record(&self, address: SocketAddr, rtt_ms: u64, ok: bool) {
        let mut peers = self.0.lock();
        let address = address.to_string();
        // A rescan of the same endpoint should update the row, not add a second.
        if let Some(existing) = peers.iter_mut().find(|p| p.address == address) {
            existing.rtt_ms = rtt_ms;
            existing.ok = ok;
            return;
        }
        peers.push(ScannedPeer {
            address,
            rtt_ms,
            ok,
        });
    }

    pub fn snapshot(&self) -> Vec<ScannedPeer> {
        self.0.lock().clone()
    }
}

/// Translate GUI settings into the environment the core reads.
///
/// Aether resolves most options through `std::env`, and its own CLI works by
/// setting exactly these variables before calling into the library. Doing the same
/// keeps behaviour identical to the upstream binary.
fn configure(settings: &Settings) {
    let set = |key: &str, value: &str| std::env::set_var(key, value);

    set("AETHER_SOCKS", &settings.socks_addr().to_string());
    set("AETHER_PROTOCOL", settings.protocol.as_env());
    set("AETHER_SCAN", settings.scan.as_env());
    set("AETHER_NOIZE", settings.obfuscation.as_env());
    set("AETHER_IP", settings.ip.as_env());
    set(
        "AETHER_QUICK_RECONNECT",
        if settings.quick_reconnect { "1" } else { "0" },
    );

    let dns = settings.dns_list();
    if !dns.is_empty() {
        set("AETHER_DNS", &dns.join(","));
    }

    match settings.http_addr() {
        Some(http) => set("AETHER_HTTP_PROXY", &http.to_string()),
        None => std::env::remove_var("AETHER_HTTP_PROXY"),
    }

    // Routing rules are matched by the core from the TLS server name, so they keep
    // working behind the TUN front end where the packet has no app identity left.
    let direct = settings.route_direct.trim();
    if direct.is_empty() {
        std::env::remove_var("AETHER_ROUTE_DIRECT");
    } else {
        set("AETHER_ROUTE_DIRECT", direct);
    }

    let block = settings.route_block.trim();
    if block.is_empty() {
        std::env::remove_var("AETHER_ROUTE_BLOCK");
    } else {
        set("AETHER_ROUTE_BLOCK", block);
    }

    // Identity files live beside the app's own config, not in the working
    // directory, so running from Program Files does not try to write there.
    if let Some(dir) = config_dir() {
        let _ = std::fs::create_dir_all(&dir);
        set("AETHER_CONFIG", &dir.join("aether.toml").to_string_lossy());
    }
}

/// Per-user directory for identities and settings.
pub fn config_dir() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("AetherGUI"))
}

/// The core version this GUI was linked against.
///
/// Read out of the linked crate's own metadata at build time, so it cannot drift
/// from the code that is actually running.
pub fn core_version() -> &'static str {
    // `aether` exposes no VERSION constant, and `env!` in this crate would report
    // the GUI's version. `build.rs` reads it out of core/aether/Cargo.toml.
    env!("AETHER_CORE_VERSION")
}

fn transport_of(protocol: Protocol) -> Transport {
    match protocol {
        // `gool` is WireGuard nested in WireGuard; both layers are WireGuard as
        // far as identity provisioning and endpoint scanning are concerned.
        Protocol::Wireguard | Protocol::Gool => Transport::WireGuard,
        Protocol::Masque => Transport::Masque,
    }
}

fn ip_scan(settings: &Settings) -> aether::prober::IpScan {
    use crate::model::IpVersion;
    match settings.ip {
        IpVersion::V4 => aether::prober::IpScan::V4,
        IpVersion::V6 => aether::prober::IpScan::V6,
        IpVersion::Dual => aether::prober::IpScan::Both,
    }
}

/// Provision or load an identity, scan for a gateway, verify it, and bring the
/// tunnel up.
///
/// Returns once the tunnel is carrying traffic; the tunnel itself keeps running in
/// a background task until the returned `Session` is shut down.
pub async fn connect(
    settings: &Settings,
    reporter: Arc<dyn Reporter>,
    scan_log: Arc<ScanLog>,
) -> Result<Session, String> {
    configure(settings);

    let transport = transport_of(settings.protocol);
    let cancel = Cancel::new();

    reporter.progress(Phase::Starting, "Loading identity".into());

    // 1. Identity. Provisioned once and cached on disk from then on.
    let config_path = api::identity_path(
        &std::env::var("AETHER_CONFIG").unwrap_or_else(|_| "aether.toml".into()),
        transport,
        None,
    );
    let request = ProvisionRequest::for_transport(transport);
    let identity = api::open_identity(&config_path, &request)
        .await
        .map_err(|e| format!("Could not prepare an identity: {e}"))?;

    reporter.log(format!(
        "[+] identity ready: device={} ipv4={}",
        identity.device_id, identity.ipv4
    ));

    // 2. ECH, when the transport can use it. Failure here is not fatal: the core
    //    falls back to a cleartext SNI, which still works on most networks.
    let ech = if matches!(transport, Transport::Masque) {
        api::fetch_ech_config().await
    } else {
        None
    };

    // 3. Scan. This is the slow part, so the UI gets told before it starts.
    reporter.progress(
        Phase::Scanning,
        format!("Probing gateways ({})", settings.scan.as_env()),
    );
    scan_log.clear();

    let scan_request = ScanRequest::for_transport(transport)
        .with_mode(settings.scan.as_env())
        .with_ip(ip_scan(settings))
        .with_profile(settings.obfuscation.as_env());

    let endpoint = api::scan(&identity, &scan_request, &cancel)
        .await
        .map_err(|e| match e {
            aether::error::AetherError::Cancelled => "Cancelled".to_string(),
            other => format!("No reachable gateway found: {other}"),
        })?;

    let peer = endpoint.socket();
    scan_log.record(peer, endpoint.rtt_ms, true);
    reporter.peer_found(peer, endpoint.rtt_ms, true);
    reporter.log(format!("[+] selected {peer} (rtt {}ms)", endpoint.rtt_ms));

    // 4. Verify before committing. Without this a gateway that completes a
    //    handshake but drops payload would be reported as connected, and the user
    //    would be left with a tunnel that carries nothing.
    reporter.progress(Phase::Verifying, "Validating the data plane".into());

    let mut spec = TunnelSpec::for_transport(transport)
        .with_socks(settings.socks_addr())
        .with_profile(settings.obfuscation.as_env());
    spec.ech = ech;
    if let Some(http) = settings.http_addr() {
        spec = spec.with_http(http);
    }

    let verified = api::verify_endpoint(&identity, peer, &spec, &cancel)
        .await
        .map_err(|e| match e {
            aether::error::AetherError::Cancelled => "Cancelled".to_string(),
            other => format!("Verification failed: {other}"),
        })?;

    if !verified {
        scan_log.record(peer, endpoint.rtt_ms, false);
        reporter.peer_found(peer, endpoint.rtt_ms, false);
        return Err(format!("{peer} answered but would not pass traffic"));
    }

    // 5. Run the tunnel. `api::connect` only returns when the tunnel stops, so it
    //    owns a task for the rest of the session's life.
    let socks = settings.socks_addr();
    let task = {
        let identity = identity.clone();
        let spec = spec.clone();
        let cancel = cancel.clone();
        let reporter = reporter.clone();

        tokio::spawn(async move {
            match api::connect(&identity, peer, &spec, &cancel).await {
                Ok(()) => {
                    reporter.log("[+] tunnel closed cleanly".into());
                    reporter.progress(Phase::Idle, String::new());
                }
                Err(aether::error::AetherError::Cancelled) => {
                    reporter.progress(Phase::Idle, String::new());
                }
                Err(e) => {
                    reporter.log(format!("[-] tunnel exited: {e}"));
                    reporter.progress(Phase::Error, format!("Tunnel dropped: {e}"));
                }
            }
        })
    };

    // Give the listener a moment to bind before anything is pointed at it. Without
    // this the first connection after "Connected" can be refused.
    wait_for_listener(socks).await?;

    Ok(Session {
        cancel,
        socks,
        peer,
        rtt_ms: endpoint.rtt_ms,
        protocol: settings.protocol,
        task,
    })
}

/// Poll until Aether's SOCKS5 listener accepts connections.
async fn wait_for_listener(socks: SocketAddr) -> Result<(), String> {
    for _ in 0..60 {
        if tokio::net::TcpStream::connect(socks).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(format!(
        "the tunnel came up but nothing is listening on {socks}"
    ))
}
