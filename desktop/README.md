# Aether GUI by NetRepublic

A modern Windows client for the [Aether](https://github.com/CluvexStudio/Aether) censorship-circumvention core.

Two ways to route your traffic:

- **System Proxy** — sets the Windows proxy to Aether's SOCKS5 listener. Fast, needs no admin rights, and covers browsers and most CLI tools.
- **Full Tunnel (TUN)** — creates a virtual network adapter and moves the system's default route onto it. Every packet on the machine goes through the tunnel: games, Telegram Desktop, torrent clients, and anything else that ignores the Windows proxy setting.

Made by **Amirreza**. The tunnelling engine is CluvexStudio's work — this project is the window around it.

---

## Status

The UI, the engine bindings, both tunnel modes and the installer config are complete.

Verified so far:

- Frontend typechecks and builds (`tsc --noEmit && vite build`).
- `cargo clippy --all-targets` is clean for both the host and the `x86_64-pc-windows-gnu` target.
- `cargo test` passes — 8 unit tests over the SOCKS5 datagram framing, the address-plan invariants and the split-route arithmetic.
- A **release `.exe` cross-compiles from Linux** (11 MB, `PE32+ GUI x86-64`), with the elevation manifest confirmed embedded in the binary.

Not yet verified, and it needs a real Windows machine:

- The TUN adapter actually coming up, and traffic flowing through it.
- The route table changes and the kill switch.
- The system proxy registry path.

Cross-compilation proves the code compiles and links for Windows; it says nothing about whether Wintun loads or whether the routes behave. Treat the first Windows run as the real test.

---

## What it looks like

A 420×680 frameless window, dark, with a single large power button at the centre.

- The button carries a gradient that shifts from indigo to emerald as the tunnel comes up, and three pulse rings that breathe outward — fast and tight while scanning, slow and wide once connected.
- Behind everything, two blurred gradient blobs drift on a 26-second loop, so the surface never looks like a flat rectangle.
- Live download and upload cards with sparklines, a gateway chip showing address, latency and protocol, and a Stats page with a throughput graph, the list of probed gateways and the engine log.
- Every transition runs through Framer Motion: spring physics on the button, a sliding pill on the nav and the segmented controls, blur-and-slide on text swaps.

---

## Requirements

To build, on Windows 10 or 11 (x64):

| Tool | Why |
| --- | --- |
| [Rust](https://rustup.rs) (stable) | The GUI and the core are both Rust |
| [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/downloads/) — *Desktop development with C++* | MSVC linker, required by Rust on Windows |
| [CMake](https://cmake.org/download/) | Builds BoringSSL inside quiche |
| [Strawberry Perl](https://strawberryperl.com/) | BoringSSL's build scripts need it |
| [Node.js](https://nodejs.org) 20+ | Builds the frontend |
| [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) | Already present on Windows 11; the installer fetches it on Windows 10 |

To run: Windows 10 1809 or later. Full Tunnel mode needs administrator rights.

---

## Building

```powershell
# 1. Clone this project
git clone <your-repo-url> aether-gui
cd aether-gui

# 2. Clone the Aether core alongside it.
#    It must land at core/aether and core/quiche.
git clone --depth 1 https://github.com/CluvexStudio/Aether.git core

# 3. Fetch wintun.dll — see the section below

# 4. Frontend dependencies
npm install

# 5. Development run, with hot reload
npm run tauri:dev

# 6. Release build: produces an installer under
#    src-tauri/target/release/bundle/
npm run tauri:build
```

### Cross-compiling from Linux

Useful for a quick compile check without a Windows box. It produces a working
`.exe` but **not** an installer, and it cannot test any of the networking.

```bash
rustup target add x86_64-pc-windows-gnu
apt-get install -y mingw-w64 nasm cmake clang libclang-dev   # BoringSSL needs these

npm run build                                                # dist/ must exist first
cd src-tauri
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/aether-gui.exe
```

Copy `wintun.dll` next to the `.exe` before running it on Windows.

The `corrupt .drectve at end of def file` warnings from the linker come from
BoringSSL's MinGW build and are harmless — the MSVC build does not produce them.

Afterwards, build the installer: see `installer/README.md`.

### wintun.dll

Full Tunnel mode needs Wintun, WireGuard's TUN driver for Windows.

1. Download from [wintun.net](https://www.wintun.net/) — the official signed release.
2. Extract `bin/amd64/wintun.dll`.
3. Place it at `src-tauri/resources/wintun.dll`.

Tauri bundles it next to the executable, and `src-tauri/src/tun/adapter.rs` loads it from there. Without it the build fails outright, since it is declared as a bundled resource; System Proxy mode never touches it at runtime.

Verified against 0.14.1, SHA-256 `e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce`.

### Icons

Regenerate the icon set after changing the mark:

```powershell
python3 scripts/make_icons.py
```

The script renders each `.ico` entry at its native size rather than downscaling one large image, because the Æ ligature closes up below 32px — the tray sizes fall back to a bare A for that reason.

---

## How Full Tunnel works

```
┌───────────┐  IP packets  ┌──────────┐  TCP/UDP  ┌────────┐
│  Wintun   │─────────────▶│ ipstack  │──────────▶│ Aether │
│  adapter  │              │ netstack │  SOCKS5   │  :1819 │
└───────────┘              └──────────┘           └────────┘
      ▲
   the whole system's traffic
```

The Aether core carries no TUN code of its own — it exposes a SOCKS5 listener and nothing else. So this project supplies the missing half: a Wintun adapter, a userspace TCP/IP stack (`ipstack`) to turn raw packets back into flows, and a SOCKS5 client to hand each flow to Aether.

**Route ordering is the part that matters.** A default route pointing at the tunnel would also capture the tunnel's own packets to the Cloudflare edge, looping traffic into itself. So:

1. The gateway's address gets a host route pinned to the *physical* interface.
2. Only then does the default route move — installed as `0.0.0.0/1` plus `128.0.0.0/1` rather than a real `0.0.0.0/0`.

Two `/1` routes beat any real default on longest-prefix match, so the tunnel wins **without** the physical default being deleted. If the process dies uncleanly, the original route is still sitting there and the machine recovers by itself.

**DNS** is set on the tunnel interface, so lookups do not leak out the physical link.

**The kill switch** drops the tunnel's routes without restoring the physical ones. With no default route the machine has no way out at all — an unreachable network beats an unprotected one.

---

## Project layout

```
aether-gui/
├── core/                       Aether upstream, cloned, untouched
├── scripts/make_icons.py       Icon generator
├── src/                        React frontend
│   ├── components/             PowerButton, PulseRings, Sparkline, …
│   ├── pages/                  Connect, Settings, Stats, About
│   └── lib/                    api (IPC), store (Zustand), motion, types
└── src-tauri/
    ├── src/
    │   ├── engine.rs           Drives the Aether core as a library
    │   ├── proxy_mode.rs       Windows proxy registry keys
    │   ├── relay.rs            Counting relay, so proxy mode has real stats
    │   ├── tun/
    │   │   ├── adapter.rs      Wintun device, address plan, byte counters
    │   │   ├── routes.rs       Route ordering and the kill switch
    │   │   └── socks.rs        SOCKS5 CONNECT + UDP ASSOCIATE
    │   ├── commands.rs         IPC surface and state transitions
    │   └── stats.rs            Throughput sampler
    └── tauri.conf.json
```

The core is linked as a **library**, not spawned as a child process. That is what lets the progress events in the UI come from the code that actually did the work, rather than from parsing log lines out of a pipe.

---

## Developing the UI without Windows

`npm run dev` in a plain browser works: `src/lib/api.ts` detects the missing Tauri runtime and substitutes a mock backend that walks through the connect phases and emits fake throughput. Every screen and animation can be reviewed that way.

---

## Troubleshooting

**`cargo build` fails inside `boring-sys` or `quiche`.** This is the most likely first failure, and it is almost always a missing build tool rather than a code problem. Confirm `cmake --version` and `perl --version` both answer in the same shell you are building from, and that you are in a *Developer* PowerShell so MSVC is on the path.

**"could not create the Aether adapter".** `wintun.dll` is missing from next to the executable, or the app is not elevated. Both produce this message.

**Full Tunnel connects but nothing loads.** Check `route print -4` for the two `/1` routes pointing at `10.6.7.1`, and confirm a host route exists for the gateway address. If the second is missing, the tunnel is carrying itself.

**The route table is left broken after a crash.** The physical default is never deleted, so this should not happen. If it does: `route delete 0.0.0.0 mask 128.0.0.0` and `route delete 128.0.0.0 mask 128.0.0.0`.

---

## Credits

The tunnelling engine — MASQUE, WireGuard, gool, endpoint discovery, obfuscation — is built by **[CluvexStudio](https://github.com/CluvexStudio/Aether)**. Thanks to them for building and open-sourcing the hard part.

MASQUE support rests on Cloudflare's **Quiche**. The TUN device uses **Wintun** by WireGuard LLC.

Updates and releases: **[t.me/net_republic](https://t.me/net_republic)**

## Licence

AGPL-3.0-or-later, matching the Aether core. The source of this GUI is published as that licence requires.

Aether is a trademark of CluvexStudio; this is an independent front end and is not affiliated with them.
