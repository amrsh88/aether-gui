//! System route table manipulation and the kill switch.
//!
//! Getting the ordering right is the entire job of this module. A default route
//! pointing at the tunnel would also capture the tunnel's *own* packets to the
//! Cloudflare edge, looping traffic into itself and killing the connection
//! instantly. So the peer's address gets a host route pinned to the physical
//! gateway first, and only then does the default route move to the TUN adapter.
//!
//! The default route is installed as two /1 routes (`0.0.0.0/1` and
//! `128.0.0.0/1`) rather than a real `0.0.0.0/0`. Longest-prefix match means these
//! beat any other interface's default without deleting it, so if the process dies
//! uncleanly the physical route is still sitting there intact and the machine
//! recovers by itself.
//!
//! Changes are applied with `route` and `netsh` on purpose: those are the commands
//! a user would run to inspect or undo the damage by hand, so a wedged state stays
//! diagnosable instead of being buried inside an API call.

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use super::adapter::{ADAPTER_NAME, MTU, TUN_ADDR, TUN_GATEWAY};

/// The two halves of a split default route.
const SPLIT_DEFAULT: [&str; 2] = ["0.0.0.0", "128.0.0.0"];
const SPLIT_MASK: &str = "128.0.0.0";

/// What we changed, so `restore` can put it all back.
#[derive(Debug, Default)]
pub struct RouteState {
    /// Host route pinned to the physical gateway for the tunnel endpoint.
    pinned_peer: Option<IpAddr>,
    /// True once the split default route points at the TUN adapter.
    default_moved: bool,
    /// Interface index of the TUN adapter.
    tun_index: Option<u32>,
}

/// Run a command, treating a non-zero exit as an error.
async fn run(program: &str, args: &[&str]) -> io::Result<String> {
    log::debug!("[>] {program} {}", args.join(" "));

    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(io::Error::other(format!(
            "`{program} {}` failed: {detail}",
            args.join(" ")
        )));
    }

    Ok(stdout)
}

/// Same as `run`, but a failure is logged instead of propagated.
///
/// Used throughout teardown, where one failing step must never prevent the rest
/// of the cleanup from running — a half-restored route table is far worse than a
/// logged warning.
async fn run_best_effort(program: &str, args: &[&str]) {
    if let Err(e) = run(program, args).await {
        log::warn!("[!] {e}");
    }
}

/// The IPv4 gateway of the current default route, and the interface it uses.
///
/// Read *before* we touch anything: once our own routes exist this lookup would
/// return the tunnel instead of the physical link.
pub async fn physical_default() -> io::Result<(IpAddr, IpAddr)> {
    // `route print -4` has a stable, unlocalised layout, unlike PowerShell's
    // object formatting which changes with the system language.
    let table = run("route", &["print", "-4"]).await?;

    let mut best: Option<(u32, IpAddr, IpAddr)> = None;

    for line in table.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // Active-route rows look like:
        //   0.0.0.0   0.0.0.0   192.168.1.1   192.168.1.42   25
        if cols.len() != 5 || cols[0] != "0.0.0.0" || cols[1] != "0.0.0.0" {
            continue;
        }

        let (Ok(gateway), Ok(iface), Ok(metric)) = (
            cols[2].parse::<IpAddr>(),
            cols[3].parse::<IpAddr>(),
            cols[4].parse::<u32>(),
        ) else {
            continue;
        };

        // Our own tunnel gateway must never be mistaken for the physical one.
        if gateway == IpAddr::V4(TUN_GATEWAY) || iface == IpAddr::V4(TUN_ADDR) {
            continue;
        }

        if best.map(|(m, _, _)| metric < m).unwrap_or(true) {
            best = Some((metric, gateway, iface));
        }
    }

    best.map(|(_, gateway, iface)| (gateway, iface))
        .ok_or_else(|| io::Error::other("no IPv4 default route found — is the machine online?"))
}

/// Interface index of our TUN adapter, by name.
pub async fn tun_interface_index() -> io::Result<u32> {
    let output = run("netsh", &["interface", "ipv4", "show", "interfaces"]).await?;

    for line in output.lines() {
        if !line.contains(ADAPTER_NAME) {
            continue;
        }
        if let Some(index) = line
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u32>().ok())
        {
            return Ok(index);
        }
    }

    Err(io::Error::other(format!(
        "the {ADAPTER_NAME} adapter is not visible to netsh yet"
    )))
}

/// Poll until the adapter is registered with the IP stack and holds our address.
///
/// The `tun` crate already waits for the interface to become configurable, but
/// the routing table can still lag behind on slower machines, and adding a route
/// to an interface Windows has not finished registering fails outright.
async fn wait_for_adapter() -> io::Result<u32> {
    for attempt in 0..40 {
        if let Ok(index) = tun_interface_index().await {
            let addresses = run(
                "netsh",
                &[
                    "interface",
                    "ipv4",
                    "show",
                    "ipaddresses",
                    &index.to_string(),
                ],
            )
            .await
            .unwrap_or_default();

            if addresses.contains(&TUN_ADDR.to_string()) {
                return Ok(index);
            }
        }

        if attempt == 0 {
            log::info!("[*] waiting for the {ADAPTER_NAME} adapter to come up");
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    Err(io::Error::other(format!(
        "the {ADAPTER_NAME} adapter never came up — check that wintun.dll sits next to the executable"
    )))
}

/// Point all system traffic at the tunnel.
///
/// `peer` is the endpoint Aether connected to; its traffic must keep using the
/// physical link or the tunnel ends up carrying itself.
pub async fn apply(peer: SocketAddr) -> io::Result<RouteState> {
    let mut state = RouteState::default();

    let (physical_gateway, physical_iface) = physical_default().await?;
    log::info!("[+] physical default is {physical_gateway} via {physical_iface}");

    // 1. Pin the tunnel endpoint to the physical link. This has to happen before
    //    the default route moves, or the tunnel immediately captures itself.
    let peer_ip = peer.ip().to_string();
    let gateway = physical_gateway.to_string();
    run(
        "route",
        &[
            "add",
            &peer_ip,
            "mask",
            "255.255.255.255",
            &gateway,
            "metric",
            "1",
        ],
    )
    .await?;
    state.pinned_peer = Some(peer.ip());
    log::info!("[+] pinned {peer_ip} to the physical gateway");

    // From here on, any failure must roll back the pinned route rather than leave
    // a stray host route behind.
    let index = match wait_for_adapter().await {
        Ok(index) => index,
        Err(e) => {
            restore(&mut state).await;
            return Err(e);
        }
    };
    state.tun_index = Some(index);

    // 2. Match the adapter MTU on the IP interface. The adapter itself was
    //    configured at creation; this covers the sub-interface Windows tracks
    //    separately and otherwise leaves at 1500.
    run_best_effort(
        "netsh",
        &[
            "interface",
            "ipv4",
            "set",
            "subinterface",
            &index.to_string(),
            &format!("mtu={MTU}"),
            "store=active",
        ],
    )
    .await;

    // 3. Split default route. Two /1s beat any real default on prefix length, so
    //    we win without deleting the physical route.
    let tun_gateway = TUN_GATEWAY.to_string();
    let tun_index = index.to_string();
    for network in SPLIT_DEFAULT {
        if let Err(e) = run(
            "route",
            &[
                "add",
                network,
                "mask",
                SPLIT_MASK,
                &tun_gateway,
                "metric",
                "1",
                "if",
                &tun_index,
            ],
        )
        .await
        {
            state.default_moved = true; // a half-installed pair still needs cleanup
            restore(&mut state).await;
            return Err(e);
        }
    }
    state.default_moved = true;
    log::info!("[+] system traffic now routes through {ADAPTER_NAME}");

    Ok(state)
}

/// Undo everything `apply` did. Safe to call twice, and safe to call on a
/// partially-applied state.
pub async fn restore(state: &mut RouteState) {
    if state.default_moved {
        let tun_gateway = TUN_GATEWAY.to_string();
        for network in SPLIT_DEFAULT {
            run_best_effort(
                "route",
                &["delete", network, "mask", SPLIT_MASK, &tun_gateway],
            )
            .await;
        }
        state.default_moved = false;
        log::info!("[+] default route handed back to the physical link");
    }

    if let Some(peer) = state.pinned_peer.take() {
        run_best_effort("route", &["delete", &peer.to_string()]).await;
    }

    if let Some(index) = state.tun_index.take() {
        // Hand DNS back to DHCP so the interface leaves nothing behind even if
        // the adapter itself lingers until the driver unloads.
        run_best_effort(
            "netsh",
            &[
                "interface",
                "ipv4",
                "set",
                "dnsservers",
                &index.to_string(),
                "dhcp",
            ],
        )
        .await;
    }
}

/// Drop the tunnel's routes *without* restoring the physical ones.
///
/// This is the kill switch. With the default route gone and the physical one
/// still un-preferred, the machine simply has no way out — which is the correct
/// failure mode. An unreachable network beats an unprotected one.
pub async fn engage_kill_switch(state: &mut RouteState) {
    if !state.default_moved {
        return;
    }

    log::warn!("[!] kill switch engaged — dropping the tunnel's default route");

    let tun_gateway = TUN_GATEWAY.to_string();
    for network in SPLIT_DEFAULT {
        run_best_effort(
            "route",
            &["delete", network, "mask", SPLIT_MASK, &tun_gateway],
        )
        .await;
    }
    state.default_moved = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Count the leading ones in a dotted-quad mask.
    fn prefix_len(mask: &str) -> u32 {
        mask.split('.')
            .filter_map(|octet| octet.parse::<u8>().ok())
            .fold(0u32, |bits, octet| bits + octet.count_ones())
    }

    #[test]
    fn the_split_mask_is_a_slash_one() {
        // The whole route plan rests on longest-prefix match: a /1 always beats the
        // /0 the physical interface carries, regardless of metric. That is why the
        // physical default is never deleted, and therefore why a crash fails safe.
        assert_eq!(prefix_len(SPLIT_MASK), 1);
    }

    #[test]
    fn the_two_halves_cover_the_whole_address_space() {
        // 0.0.0.0/1 covers 0-127.x and 128.0.0.0/1 covers 128-255.x. A gap here
        // would silently leak whichever range fell through it.
        let bits = prefix_len(SPLIT_MASK);
        let size = 1u64 << (32 - bits);

        let starts: Vec<u64> = SPLIT_DEFAULT.iter().map(|n| to_u32(n) as u64).collect();

        assert_eq!(starts.len(), 2);
        assert_eq!(starts[0], 0, "the first half must start at the bottom");
        assert_eq!(
            starts[1], size,
            "the second half must start where the first ends"
        );
        assert_eq!(starts[1] + size, 1u64 << 32, "the pair must reach the top");
    }

    /// Dotted-quad to host-order integer.
    fn to_u32(address: &str) -> u32 {
        address
            .split('.')
            .filter_map(|octet| octet.parse::<u32>().ok())
            .fold(0u32, |acc, octet| (acc << 8) | octet)
    }
}
