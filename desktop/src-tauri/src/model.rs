//! Types that cross the IPC boundary.
//!
//! Field names are camelCase on the wire to match `src/lib/types.ts`.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Masque,
    Wireguard,
    Gool,
}

impl Protocol {
    /// The value Aether's own CLI/env layer expects in `AETHER_PROTOCOL`.
    pub fn as_env(self) -> &'static str {
        match self {
            Protocol::Masque => "masque",
            Protocol::Wireguard => "wg",
            Protocol::Gool => "gool",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode {
    Turbo,
    Balanced,
    Thorough,
    Stealth,
    Ironclad,
}

impl ScanMode {
    pub fn as_env(self) -> &'static str {
        match self {
            ScanMode::Turbo => "turbo",
            ScanMode::Balanced => "balanced",
            ScanMode::Thorough => "thorough",
            ScanMode::Stealth => "stealth",
            ScanMode::Ironclad => "ironclad",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Obfuscation {
    Off,
    Light,
    Balanced,
    Gfw,
}

impl Obfuscation {
    /// Aether names these profiles slightly differently than the UI does.
    pub fn as_env(self) -> &'static str {
        match self {
            Obfuscation::Off => "off",
            Obfuscation::Light => "firewall",
            Obfuscation::Balanced => "balanced",
            Obfuscation::Gfw => "gfw",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpVersion {
    V4,
    V6,
    Dual,
}

impl IpVersion {
    pub fn as_env(self) -> &'static str {
        match self {
            IpVersion::V4 => "v4",
            IpVersion::V6 => "v6",
            IpVersion::Dual => "both",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelMode {
    /// Set the Windows system proxy to our SOCKS5 listener.
    Proxy,
    /// Create a TUN adapter and route every packet through the tunnel.
    Tun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub mode: TunnelMode,
    pub protocol: Protocol,
    pub scan: ScanMode,
    pub obfuscation: Obfuscation,
    pub bind: String,
    pub http_proxy: Option<String>,
    pub ip: IpVersion,
    pub dns: String,
    pub kill_switch: bool,
    pub auto_connect: bool,
    pub start_with_windows: bool,
    pub quick_reconnect: bool,
    pub route_direct: String,
    pub route_block: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: TunnelMode::Proxy,
            protocol: Protocol::Masque,
            scan: ScanMode::Balanced,
            obfuscation: Obfuscation::Balanced,
            bind: "127.0.0.1:1819".into(),
            http_proxy: None,
            ip: IpVersion::V4,
            dns: "1.1.1.1, 1.0.0.1".into(),
            kill_switch: true,
            auto_connect: false,
            start_with_windows: false,
            quick_reconnect: true,
            route_direct: String::new(),
            route_block: String::new(),
        }
    }
}

impl Settings {
    /// Parse the bind field, falling back to Aether's own default on garbage input.
    pub fn socks_addr(&self) -> SocketAddr {
        self.bind
            .trim()
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 1819)))
    }

    pub fn http_addr(&self) -> Option<SocketAddr> {
        self.http_proxy.as_deref()?.trim().parse().ok()
    }

    /// DNS servers to hand the TUN adapter, normalised and de-spaced.
    pub fn dns_list(&self) -> Vec<String> {
        self.dns
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Idle,
    Starting,
    Scanning,
    Verifying,
    Routing,
    Connected,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub address: String,
    pub rtt_ms: u64,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub phase: Phase,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<PeerInfo>,
}

impl ProgressEvent {
    pub fn new(phase: Phase, detail: impl Into<String>) -> Self {
        Self {
            phase,
            detail: detail.into(),
            peer: None,
        }
    }

    pub fn with_peer(mut self, peer: PeerInfo) -> Self {
        self.peer = Some(peer);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsEvent {
    pub down_bps: f64,
    pub up_bps: f64,
    pub total_down: u64,
    pub total_up: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedPeer {
    pub address: String,
    pub rtt_ms: u64,
    pub ok: bool,
}

/// Event channel names, kept in one place so the TS side can't drift.
pub mod events {
    pub const PROGRESS: &str = "aether://progress";
    pub const STATS: &str = "aether://stats";
    pub const LOG: &str = "aether://log";
}
