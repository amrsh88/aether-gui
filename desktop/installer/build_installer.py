#!/usr/bin/env python3
"""
Build an installer for Aether GUI by NetRepublic.

Produces a single self-extracting .exe that installs the app to
%ProgramFiles%\Aether GUI and registers it in Add/Remove Programs.

Uses 7-Zip's SFX module — no NSIS, no Wine, no Windows required to build.

Usage from the project root:

    python3 installer/build_installer.py

    -> Aether-GUI-Setup-1.0.0.exe   (repo root)
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
VERSION = "1.0.0"
APPNAME = "Aether GUI"
PUBLISHER = "NetRepublic"
UNINST_KEY = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Aether GUI"

# 7-Zip's SFX module — either a system install or downloaded fresh.
SFX_MODULE = "7z.sfx"

INSTALLER_EXE = REPO / f"Aether-GUI-Setup-{VERSION}.exe"


def find_7z() -> Path | None:
    """Return the path to the 7z CLI, or None."""
    for candidate in ["7z", "/usr/bin/7z", "/usr/local/bin/7z"]:
        if shutil.which(candidate):
            return Path(candidate)
    return None


def find_7zsd() -> Path | None:
    """Locate the Windows SFX module (7z.sfx).

    This is the module for PE executables; the p7zip packages on Linux only ship
    7zCon.sfx, which builds console archives. The real 7z.sfx comes from the
    official 7-Zip Windows installer — build.py fetches it if missing.
    """
    candidates = [
        REPO / "installer" / "7z.sfx",
        Path("/usr/lib/p7zip/7z.sfx"),
        Path("7z_extract/7z.sfx"),
    ]
    for p in candidates:
        if p.exists():
            return p
    return None


def ensure_windows_sfx() -> bool:
    """Download the genuine 7z.sfx from 7-zip.org if not already present.

    Only the module that produces GUI executables will do: 7zCon.sfx makes a
    console window flash and cannot show a BeginPrompt dialog.
    """
    target = REPO / "installer" / "7z.sfx"
    if target.exists():
        return True

    url = "https://www.7-zip.org/a/7z2409-x64.exe"
    print(f"fetching the Windows SFX module from {url} ...")
    try:
        import urllib.request

        with tempfile.TemporaryDirectory() as tmp:
            installer = Path(tmp) / "7z-installer.exe"
            urllib.request.urlretrieve(url, installer)

            subprocess.run(
                ["7z", "x", "-y", f"-o{tmp}/extracted", str(installer)],
                check=True,
                capture_output=True,
            )

            shutil.copy2(Path(tmp) / "extracted" / "7z.sfx", target)
            print(f"  saved {target}")
            return True
    except Exception as e:
        print(f"could not fetch the SFX module: {e}")
        return False


def make_config() -> bytes:
    """Return the 7-Zip SFX config that drives the install.

    The SFX extracts itself to a temp directory and runs install.cmd, which
    copies the files to Program Files, registers the uninstall entry and prints
    what happened. Everything after extraction is plain cmd — no PowerShell, so
    no ExecutionPolicy surprises.
    """
    lines = [
        ";!@Install@!UTF-8!",
        f'Title="{APPNAME} {VERSION} Installer"',
        f'BeginPrompt="Install {APPNAME} {VERSION}?\\n\\nRequires administrator rights."',
        'RunProgram="install.cmd"',
        "ExtractTitle=\"Installing...\"",
        'ExtractDialogText="Copying files"',
        ";!@InstallEnd@!",
    ]
    return "\r\n".join(lines).encode("utf-8")


def write_registry_script(stage: Path, install_dir: str) -> None:
    """Write the install and uninstall cmd scripts.

    install.cmd copies from the extraction directory (%~dp0, wherever the SFX
    unpacked itself) into Program Files, then registers everything Add/Remove
    Programs needs. It also drops uninstall.cmd next to the app so that entry
    has something to run later.

    CRLF line endings are mandatory: cmd.exe mis-parses multi-line files with
    bare LF in ways that fail silently.
    """
    exe_path = f"{install_dir}\\aether-gui.exe"
    uninstall_cmd = f"{install_dir}\\uninstall.cmd"

    install = "\r\n".join([
        "@echo off",
        "rem Aether GUI installer steps. %~dp0 is where the SFX unpacked itself.",
        "",
        'if not exist "%ProgramFiles%" (',
        "    echo Could not locate the Program Files directory.",
        "    pause",
        "    exit /b 1",
        ")",
        "",
        f'set "DEST=%ProgramFiles%\\{APPNAME}"',
        'echo Installing to %DEST% ...',
        "",
        'if not exist "%DEST%" mkdir "%DEST%"',
        "",
        'copy /y "%~dp0aether-gui.exe"      "%DEST%\\" >nul || goto :fail',
        'copy /y "%~dp0wintun.dll"          "%DEST%\\" >nul || goto :fail',
        'copy /y "%~dp0WebView2Loader.dll"  "%DEST%\\" >nul || goto :fail',
        'copy /y "%~dp0LICENSE"             "%DEST%\\LICENSE.txt" >nul || goto :fail',
        'copy /y "%~dp0uninstall.cmd"       "%DEST%\\" >nul || goto :fail',
        "",
        'echo Creating shortcuts ...',
        f'powershell -NoProfile -Command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut([Environment]::GetFolderPath(\'Desktop\')+\'\\{APPNAME}.lnk\'); $s.TargetPath=\'{exe_path}\'; $s.Save()" >nul 2>&1',
        f'powershell -NoProfile -Command "$d=[Environment]::GetFolderPath(\'Programs\')+\'\\{PUBLISHER}\'; New-Item -ItemType Directory -Force -Path $d | Out-Null; $s=(New-Object -ComObject WScript.Shell).CreateShortcut($d+\'\\{APPNAME}.lnk\'); $s.TargetPath=\'{exe_path}\'; $s.Save()" >nul 2>&1',
        "",
        "echo Writing the registry entry ...",
        f'reg add "{UNINST_KEY}" /v DisplayName     /t REG_SZ /d "{APPNAME} {VERSION}" /f >nul',
        f'reg add "{UNINST_KEY}" /v DisplayVersion /t REG_SZ /d "{VERSION}" /f >nul',
        f'reg add "{UNINST_KEY}" /v Publisher      /t REG_SZ /d "{PUBLISHER}" /f >nul',
        f'reg add "{UNINST_KEY}" /v InstallLocation /t REG_SZ /d "%DEST%" /f >nul',
        f'reg add "{UNINST_KEY}" /v DisplayIcon    /t REG_SZ /d "{exe_path}" /f >nul',
        f'reg add "{UNINST_KEY}" /v URLInfoAbout   /t REG_SZ /d "https://t.me/net_republic" /f >nul',
        f'reg add "{UNINST_KEY}" /v UninstallString /t REG_SZ /d "{uninstall_cmd}" /f >nul',
        f'reg add "{UNINST_KEY}" /v NoModify       /t REG_DWORD /d 1 /f >nul',
        f'reg add "{UNINST_KEY}" /v NoRepair       /t REG_DWORD /d 1 /f >nul',
        "",
        "echo.",
        f"echo {APPNAME} installed to:",
        "echo   %DEST%",
        "echo.",
        "echo Run it as administrator for Full Tunnel mode.",
        "echo.",
        "pause",
        "exit /b 0",
        "",
        ":fail",
        "echo.",
        "echo Installation failed while copying files.",
        "pause",
        "exit /b 1",
        "",
    ])

    uninstall = "\r\n".join([
        "@echo off",
        f"echo Uninstalling {APPNAME} ...",
        "",
        # Identities live under %APPDATA%\AetherGUI and survive on purpose:
        # reinstalling must not log the user out of their tunnel.
        f'del /q "%ProgramFiles%\\{APPNAME}\\aether-gui.exe"     2>nul',
        f'del /q "%ProgramFiles%\\{APPNAME}\\wintun.dll"         2>nul',
        f'del /q "%ProgramFiles%\\{APPNAME}\\WebView2Loader.dll" 2>nul',
        f'del /q "%ProgramFiles%\\{APPNAME}\\LICENSE.txt"        2>nul',
        f'del /q "%ProgramFiles%\\{APPNAME}\\uninstall.cmd"      2>nul',
        f'rmdir "%ProgramFiles%\\{APPNAME}" 2>nul',
        "",
        f'del /q "%Public%\\Desktop\\{APPNAME}.lnk" 2>nul',
        f'rmdir /s /q "%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\{PUBLISHER}" 2>nul',
        "",
        f'reg delete "{UNINST_KEY}" /f >nul 2>&1',
        "",
        "echo Done. Your Aether identities were kept.",
        "pause",
        "",
    ])

    for name, body in [("install.cmd", install), ("uninstall.cmd", uninstall)]:
        path = stage / name
        with open(path, "w", encoding="ascii", newline="") as f:
            f.write(body)


def main() -> None:
    if not ensure_windows_sfx():
        sys.exit(1)

    sfx = find_7zsd()
    sevenz = find_7z()
    if sfx is None or sevenz is None:
        print("Could not find the 7z CLI.")
        sys.exit(1)

    print(f"  SFX module: {sfx}")
    print(f"  7z CLI:     {sevenz}")

    # --- stage files ----------------------------------------------------------

    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp) / "stage"
        stage.mkdir()

        # App payload.
        exe = (REPO / "src-tauri/target/x86_64-pc-windows-gnu/release"
               / "aether-gui.exe")
        for src in [
            exe,
            REPO / "src-tauri/resources/wintun.dll",
            REPO / "src-tauri/resources/WebView2Loader.dll",
            REPO / "LICENSE",
        ]:
            if not src.exists():
                print(f"missing: {src}")
                print("build the exe first — see installer/README.md")
                sys.exit(1)
            shutil.copy2(src, stage / src.name)

        install_dir_placeholder = "%ProgramFiles%\\Aether GUI"
        write_registry_script(stage, install_dir_placeholder)
        sfx_config = make_config()

        # --- compress ---------------------------------------------------------

        archive = Path(tmp) / "archive.7z"
        subprocess.run(
            [str(sevenz), "a", "-t7z", "-mx=7", str(archive), "."],
            cwd=stage,
            check=True,
            capture_output=True,
        )

        # --- combine SFX module + config + archive -----------------------------
        #
        # The SFX format is literally: [module bytes][config text][7z archive].
        # 7z.sfx scans its own image for the ";!@Install@!UTF-8!" marker, reads
        # the config until "!@InstallEnd@!", then treats everything after that
        # as a 7z stream. No separator bytes are wanted — anything between the
        # config end marker and the archive would corrupt the stream.

        with open(INSTALLER_EXE, "wb") as out:
            out.write(sfx.read_bytes())
            out.write(sfx_config)
            with open(archive, "rb") as a:
                shutil.copyfileobj(a, out)

    size_mb = INSTALLER_EXE.stat().st_size / (1024 * 1024)
    print(f"\nInstaller ready: {INSTALLER_EXE.name} ({size_mb:.1f} MB)")
    print("Right-click -> Run as administrator on Windows to install.")


if __name__ == "__main__":
    main()
