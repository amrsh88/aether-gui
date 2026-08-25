//! Full System Tunnel: TUN adapter -> userspace IP stack -> Aether's SOCKS5.
//!
//! This is what makes the app more than a proxy switcher. Windows hands us raw IP
//! packets on a virtual adapter, `ipstack` reassembles them into TCP and UDP
//! flows, and each flow is dialled out through Aether's local SOCKS5 listener. To
//! an application there is no proxy at all — it just talks to the network and the
//! packets happen to leave through the tunnel. Games, Telegram Desktop, torrent
//! clients and anything else that ignores the Windows proxy all get covered.
//!
//! The pieces:
//!   * `adapter` — Wintun device, address plan, byte counters
//!   * `routes`  — route table ordering and the kill switch
//!   * `socks`   — SOCKS5 CONNECT and UDP ASSOCIATE client
//!
//! Failure handling is deliberately blunt: if the tunnel task exits for any reason
//! the kill switch fires before the route table is restored, so traffic is never
//! briefly exposed on the physical link during teardown.

pub mod adapter;
pub mod routes;
pub mod socks;

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use ipstack::{IpStack, IpStackConfig, IpStackStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use adapter::{Counters, CountingDevice};

/// UDP flows are dropped after this long with no traffic in either direction.
/// Long enough for DNS and game keepalives, short enough not to hoard sockets.
const UDP_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Buffer for relayed UDP payloads. One datagram never exceeds the tunnel MTU.
const UDP_BUFFER: usize = 65_535;

/// A running Full Tunnel. Dropping this does nothing; call `shutdown` so the
/// route table is restored in the right order.
pub struct Tunnel {
    counters: Arc<Counters>,
    cancel: CancellationToken,
    routes: routes::RouteState,
    kill_switch: bool,
    task: tokio::task::JoinHandle<()>,
}

impl Tunnel {
    /// Bring up the adapter, apply routes, and start forwarding.
    ///
    /// `socks` is Aether's local listener; `peer` is the remote endpoint Aether
    /// connected to, which must be excluded from the tunnel.
    pub async fn start(
        socks: SocketAddr,
        peer: SocketAddr,
        dns: &[IpAddr],
        kill_switch: bool,
    ) -> io::Result<Self> {
        // Refuse early rather than half-configuring the machine.
        if !is_elevated() {
            return Err(io::Error::other(
                "Full Tunnel mode needs administrator rights. \
                 Restart Aether GUI as administrator, or switch to System Proxy mode.",
            ));
        }

        let device = adapter::create(dns)?;
        log::info!("[+] {} adapter is up", adapter::ADAPTER_NAME);

        // Routes go on after the device exists but before any packet is forwarded,
        // so there is no window where applications see a black hole.
        let route_state = routes::apply(peer).await?;

        let counters = Arc::new(Counters::default());
        let cancel = CancellationToken::new();

        let mut config = IpStackConfig::default();
        config
            .mtu(adapter::MTU)
            .map_err(|e| io::Error::other(format!("invalid MTU: {e}")))?;
        config.udp_timeout(UDP_IDLE_TIMEOUT);

        let stack = IpStack::new(config, CountingDevice::new(device, counters.clone()));

        let task = tokio::spawn(pump(stack, socks, cancel.clone()));

        Ok(Self {
            counters,
            cancel,
            routes: route_state,
            kill_switch,
            task,
        })
    }

    /// Byte counters, shared with the stats sampler.
    pub fn counters(&self) -> Arc<Counters> {
        self.counters.clone()
    }

    /// Whether the forwarding task is still alive.
    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }

    /// Stop forwarding and restore the route table.
    pub async fn shutdown(mut self) {
        self.cancel.cancel();

        // Give in-flight copies a moment to notice the cancellation, then stop
        // waiting — a stuck flow must not block the route restore.
        let _ = tokio::time::timeout(Duration::from_secs(3), &mut self.task).await;
        if !self.task.is_finished() {
            self.task.abort();
        }

        // Kill switch first: drops the tunnel's routes while the physical default
        // is still un-preferred, so nothing leaks during the gap.
        if self.kill_switch {
            routes::engage_kill_switch(&mut self.routes).await;
        }
        routes::restore(&mut self.routes).await;

        log::info!("[+] full tunnel torn down");
    }
}

/// Accept flows from the IP stack and forward each one through SOCKS5.
async fn pump(mut stack: IpStack, socks: SocketAddr, cancel: CancellationToken) {
    loop {
        let stream = tokio::select! {
            biased;
            _ = cancel.cancelled() => break,
            accepted = stack.accept() => match accepted {
                Ok(stream) => stream,
                Err(e) => {
                    log::error!("[-] the IP stack stopped accepting flows: {e}");
                    break;
                }
            },
        };

        match stream {
            IpStackStream::Tcp(tcp) => {
                let target = tcp.peer_addr();
                let cancel = cancel.clone();
                tokio::spawn(async move {
                    if let Err(e) = forward_tcp(tcp, socks, target, cancel).await {
                        log::debug!("[-] tcp {target} closed: {e}");
                    }
                });
            }

            IpStackStream::Udp(udp) => {
                let target = udp.peer_addr();
                let cancel = cancel.clone();
                tokio::spawn(async move {
                    if let Err(e) = forward_udp(udp, socks, target, cancel).await {
                        log::debug!("[-] udp {target} closed: {e}");
                    }
                });
            }

            // ICMP and friends. Aether's SOCKS5 listener cannot carry these, and
            // silently dropping them is standard for a tun2socks path — `ping`
            // stops working while normal traffic is unaffected.
            IpStackStream::UnknownTransport(unknown) => {
                log::trace!(
                    "[.] dropping an unsupported transport ({:?}) to {}",
                    unknown.ip_protocol(),
                    unknown.dst_addr()
                );
            }

            IpStackStream::UnknownNetwork(packet) => {
                log::trace!(
                    "[.] dropping an unparsable packet of {} bytes",
                    packet.len()
                );
            }
        }
    }

    log::info!("[+] tunnel forwarding stopped");
}

/// Splice one TCP flow to a SOCKS5 CONNECT tunnel.
async fn forward_tcp(
    mut tcp: ipstack::IpStackTcpStream,
    socks: SocketAddr,
    target: SocketAddr,
    cancel: CancellationToken,
) -> io::Result<()> {
    let mut upstream = socks::connect(socks, target).await?;

    tokio::select! {
        _ = cancel.cancelled() => {}
        result = tokio::io::copy_bidirectional(&mut tcp, &mut upstream) => {
            result?;
        }
    }

    let _ = upstream.shutdown().await;
    let _ = tcp.shutdown().await;
    Ok(())
}

/// Relay one UDP flow through a SOCKS5 UDP association.
///
/// `copy_bidirectional` cannot be used here: the relay needs a per-datagram
/// header added and stripped, and message boundaries have to survive the trip.
async fn forward_udp(
    mut udp: ipstack::IpStackUdpStream,
    socks: SocketAddr,
    target: SocketAddr,
    cancel: CancellationToken,
) -> io::Result<()> {
    let relay = socks::UdpRelay::open(socks, target).await?;
    log::trace!("[.] udp {target} relayed via {}", relay.relay_addr());

    let mut from_app = vec![0u8; UDP_BUFFER];
    let mut from_net = vec![0u8; UDP_BUFFER];

    loop {
        tokio::select! {
            biased;

            _ = cancel.cancelled() => break,

            // Application -> tunnel.
            read = udp.read(&mut from_app) => match read {
                Ok(0) | Err(_) => break,
                Ok(n) => relay.send(&from_app[..n]).await?,
            },

            // Tunnel -> application.
            received = relay.recv(&mut from_net) => match received {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if udp.write_all(&from_net[..n]).await.is_err() {
                        break;
                    }
                }
            },

            // Nothing either way for a whole idle period: let the flow go.
            _ = tokio::time::sleep(UDP_IDLE_TIMEOUT) => break,
        }
    }

    let _ = udp.shutdown().await;
    Ok(())
}

/// Whether this process holds an elevated token.
///
/// Checked up front because every route command would otherwise fail one by one,
/// producing a confusing pile of errors instead of one clear message.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    use std::mem;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;

        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        ) != 0;

        CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}
