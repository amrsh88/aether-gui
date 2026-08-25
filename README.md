# Aether by NetRepublic

Two open-source clients for the [Aether](https://github.com/CluvexStudio/Aether)
censorship-circumvention core — a Windows desktop app with a full system tunnel,
and an Android app.

Made by **Amirreza** ([@net_republic](https://t.me/net_republic)).

```
├── desktop/   Windows client — Tauri v2 + React + Rust
└── android/   Android client  — Kotlin + Jetpack Compose
```

## Desktop

A modern client for the Aether core with two ways to route your traffic:

- **System Proxy** — sets the Windows proxy to Aether's SOCKS5 listener. Fast, no
  admin rights, covers browsers and CLI tools.
- **Full Tunnel (TUN)** — creates a Wintun adapter and moves the default route onto
  it. Every packet on the machine goes through the tunnel: games, Telegram
  Desktop, torrent clients, everything.

See [`desktop/README.md`](desktop/README.md) for features, building, and how the
full tunnel works internally.

| Download | |
|---|---|
| Portable | `desktop` releases → zip with exe + DLLs |
| Installer | `desktop` releases → `Aether-GUI-Setup.exe` |

## Android

The Aether client for Android: same engine, wrapped in a Compose UI. Universal APK,
arm64 + armv7 + x86_64.

See [`android/README.md`](android/README.md).

| Download | |
|---|---|
| APK (universal) | [`android/app/release/Aether-by-Net-Republic-universal.apk`](android/app/release/Aether-by-Net-Republic-universal.apk) |

## Building

Each folder is self-contained:

```bash
cd desktop && npm install && npm run tauri:dev   # desktop, dev mode
cd android && ./gradlew assembleRelease          # android (needs the SDK)
```

Both fetch the Aether core themselves; see the README in each folder for the
one-time setup steps.

## Credits

The tunnelling engine — MASQUE, WireGuard, gool, endpoint discovery, obfuscation —
is built by **[CluvexStudio](https://github.com/CluvexStudio/Aether)**. Thanks to
them for building and open-sourcing the hard part. 🙏

MASQUE support rests on Cloudflare's **Quiche**. The Windows TUN device uses
**Wintun** by WireGuard LLC.

Channel: **[t.me/net_republic](https://t.me/net_republic)**

## Licence

AGPL-3.0-or-later, matching the Aether core. Source is published as that licence
requires — see [LICENSE](LICENSE) and [NOTICE](NOTICE).
