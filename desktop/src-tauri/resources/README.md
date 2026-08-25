# Wintun

Full Tunnel mode needs `wintun.dll`, WireGuard's TUN driver for Windows. It is
**not** committed here — it ships as a signed binary from its authors and should be
obtained from them directly rather than mirrored through a third party.

## Getting it

1. Download the current release from <https://www.wintun.net/>.
2. Open the zip and take `bin/amd64/wintun.dll`.
3. Drop it in this directory, next to this file.

The result should be:

```
src-tauri/resources/wintun.dll
```

Tauri copies it beside the executable at build time, and
`src-tauri/src/tun/adapter.rs` loads it from there.

## If you skip it

The build fails: `wintun.dll` is declared as a bundled resource in
`tauri.conf.json`, and Tauri refuses to build when a declared resource is missing.
Nothing at runtime needs it unless Full Tunnel mode is used.

## Verified against

Wintun 0.14.1, `bin/amd64/wintun.dll`:

```
sha256  e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce
size    427552 bytes
```

## Why not vendor it

Wintun is signed by WireGuard LLC, and that signature is why Windows loads the driver
without complaint. Redistributing a copy invites version skew and gives users no way
to verify what they are loading. One download is the better trade.
