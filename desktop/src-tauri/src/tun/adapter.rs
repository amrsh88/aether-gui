//! TUN device creation and byte accounting.
//!
//! The `tun` crate owns the Wintun plumbing: loading `wintun.dll`, creating or
//! reopening the adapter, assigning the address, and exposing the whole thing as
//! an async device. This module adds two things on top.
//!
//! First, a fixed identity and address plan, so repeated runs reuse one interface
//! instead of littering the registry with a new GUID per launch.
//!
//! Second, a counting wrapper. Every tunnelled byte must cross this device, which
//! makes it the only honest place to measure throughput — anything higher up would
//! miss retransmits and anything lower is inside the driver.
//!
//! Note what is deliberately *not* configured here: `destination`. Setting it
//! would make the `tun` crate install a default route the moment the adapter
//! comes up, before the tunnel endpoint has been pinned to the physical link.
//! That ordering loops the tunnel through itself. Routes are applied by
//! `super::routes` instead, in an order that cannot deadlock the connection.

use std::io;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Adapter name as it appears in `ncpa.cpl` and in every `netsh` command.
pub const ADAPTER_NAME: &str = "Aether";

/// Stable adapter GUID. Any fixed value works; it only has to stay constant
/// across runs so Windows treats it as the same interface each time.
#[cfg(windows)]
pub const ADAPTER_GUID: u128 = 0x4145_5448_4552_0001_0000_0000_0000_0001;

/// 1500 minus headroom for the outer MASQUE (QUIC/HTTP-2) or WireGuard headers.
/// Set this too high and every large packet fragments inside the tunnel.
pub const MTU: u16 = 1420;

/// Address plan for the tunnel interface: a tiny island nothing else uses.
pub const TUN_ADDR: Ipv4Addr = Ipv4Addr::new(10, 6, 7, 2);
/// Only the Windows adapter setup reads this; the tests assert on it too.
#[cfg_attr(not(windows), allow(dead_code))]
pub const TUN_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 252);
pub const TUN_GATEWAY: Ipv4Addr = Ipv4Addr::new(10, 6, 7, 1);

/// Cumulative byte counts in each direction.
///
/// `down` is traffic arriving from the tunnel and handed to applications; `up` is
/// traffic applications sent into the tunnel.
#[derive(Debug, Default)]
pub struct Counters {
    pub down: AtomicU64,
    pub up: AtomicU64,
}

impl Counters {
    pub fn snapshot(&self) -> (u64, u64) {
        (
            self.down.load(Ordering::Relaxed),
            self.up.load(Ordering::Relaxed),
        )
    }
}

/// Resolve `wintun.dll`, preferring the copy Tauri bundles next to the binary.
///
/// Falling back to the bare filename lets Windows use its normal search order,
/// which is what `cargo run` during development relies on.
#[cfg(windows)]
fn wintun_path() -> std::ffi::OsString {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("wintun.dll")))
        .filter(|path| path.exists())
        .map(Into::into)
        .unwrap_or_else(|| "wintun.dll".into())
}

/// Create the TUN adapter, addressed and up.
#[cfg(windows)]
pub fn create(dns: &[std::net::IpAddr]) -> io::Result<tun::AsyncDevice> {
    let mut config = tun::Configuration::default();
    config
        .tun_name(ADAPTER_NAME)
        .address(TUN_ADDR)
        .netmask(TUN_MASK)
        .mtu(MTU)
        // Beat the physical adapter on interface metric, so that once our routes
        // exist Windows has no reason to prefer the old path.
        .metric(1)
        .up();

    let dns = dns.to_vec();
    config.platform_config(move |platform| {
        platform.device_guid(ADAPTER_GUID);
        platform.wintun_file(wintun_path());
        if !dns.is_empty() {
            // Setting DNS on the interface itself is what stops resolution from
            // leaking out the physical link while the tunnel is up.
            platform.dns_servers(&dns);
        }
        // Wintun reports the device before Windows finishes plumbing the IP
        // interface; without this wait the address assignment races and fails.
        platform.wait_for_interfaces(true, false, std::time::Duration::from_secs(10));
    });

    tun::create_as_async(&config).map_err(|e| {
        io::Error::other(format!(
            "could not create the {ADAPTER_NAME} adapter: {e}. \
             Check that wintun.dll sits next to the executable and that Aether GUI \
             is running as administrator."
        ))
    })
}

#[cfg(not(windows))]
pub fn create(_dns: &[std::net::IpAddr]) -> io::Result<tokio::io::DuplexStream> {
    Err(io::Error::other(
        "Full Tunnel mode is only implemented on Windows",
    ))
}

/// Wraps any async IP device and tallies every byte that crosses it.
pub struct CountingDevice<D> {
    inner: D,
    counters: Arc<Counters>,
}

impl<D> CountingDevice<D> {
    pub fn new(inner: D, counters: Arc<Counters>) -> Self {
        Self { inner, counters }
    }
}

impl<D: AsyncRead + Unpin> AsyncRead for CountingDevice<D> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        if result.is_ready() {
            let read = buf.filled().len().saturating_sub(before);
            if read > 0 {
                // Reading *from* the adapter means an application sent outbound.
                self.counters.up.fetch_add(read as u64, Ordering::Relaxed);
            }
        }
        result
    }
}

impl<D: AsyncWrite + Unpin> AsyncWrite for CountingDevice<D> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        let result = Pin::new(&mut self.inner).poll_write(cx, data);
        if let Poll::Ready(Ok(n)) = &result {
            // Writing *to* the adapter means inbound traffic reached an app.
            self.counters.down.fetch_add(*n as u64, Ordering::Relaxed);
        }
        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tunnel_gateway_sits_inside_the_tunnel_subnet() {
        // A /30 covers .0-.3, so .1 (gateway) and .2 (us) must both be in range
        // or Windows silently refuses the route.
        let mask = u32::from(TUN_MASK);
        assert_eq!(u32::from(TUN_ADDR) & mask, u32::from(TUN_GATEWAY) & mask);
    }

    #[test]
    fn the_mtu_leaves_room_for_an_outer_header() {
        // Compile-time, so a bad MTU fails the build rather than a test run.
        const _: () = assert!(
            MTU < 1500,
            "MTU must leave room for tunnel encapsulation, or every large packet fragments"
        );
        const _: () = assert!(MTU > 1200, "an MTU this low would cripple throughput");
    }
}
