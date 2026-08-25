# Making the installer

Two ways to get an installed app instead of a folder of loose files:

## Option A — on the server, no Windows needed

`installer/build_installer.py` wraps the cross-compiled binary into a
self-extracting installer using 7-Zip's SFX module. Run it from the project root:

```bash
python3 installer/build_installer.py
```

It fetches the genuine `7z.sfx` module from 7-zip.org on first run (the p7zip
packages only ship the console module, which flashes a cmd window), compresses
the payload with LZMA, and emits `Aether-GUI-Setup-<version>.exe` at the repo root.

Running that file on Windows shows an install prompt, copies everything to
`C:\Program Files\Aether GUI`, creates Desktop and Start Menu shortcuts,
registers an Add/Remove Programs entry with a working uninstaller — and leaves
`%APPDATA%\AetherGUI` alone, so reinstalling never logs you out.

## Option B — on Windows, the official NSIS build

The Tauri bundler produces a proper NSIS and MSI installer, but only when run
on Windows. `installer/installer.nsi` is a standalone script for the same job;
build it there with:

```powershell
# 1. The app itself, already cross-compiled on Linux:
#      src-tauri/target/x86_64-pc-windows-gnu/release/aether-gui.exe

# 2. Lay out what the installer packs:
mkdir dist
copy src-tauri\target\x86_64-pc-windows-gnu\release\aether-gui.exe dist\
copy src-tauri\resources\wintun.dll          dist\
copy src-tauri\resources\WebView2Loader.dll  dist\

# 3. Compile:
makensis installer\installer.nsi

# -> Aether-GUI-Setup-1.0.0.exe in the repo root
```

Or run `installer\build.ps1`, which stages the files and checks them first.
Requires NSIS: <https://nsis.sourceforge.io/Download>

## What both installers do

- Install to `C:\Program Files\Aether GUI`
- Copy `aether-gui.exe`, `wintun.dll`, `WebView2Loader.dll`
- Desktop + Start Menu shortcuts
- Add/Remove Programs entry with size, icon and uninstaller
- Uninstaller that removes all of it — but **keeps** `%APPDATA%\AetherGUI`,
  where your Aether identities live, so reinstalling does not log you out
