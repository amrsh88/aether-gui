; Aether GUI by NetRepublic — NSIS installer script
; Made by Amirreza
;
; Build on Windows with makensis (https://nsis.sourceforge.io):
;
;     makensis installer.nsi
;
; Produces: Aether-GUI-Setup-<version>.exe — a single-file installer that
; copies the app, its two DLLs, the Start Menu shortcuts and an uninstaller.

!include "MUI2.nsh"
!include "FileFunc.nsh"

;--------------------------------------------------------------------
; General
;--------------------------------------------------------------------

!define APPNAME      "Aether GUI"
!define PUBLISHER    "NetRepublic"
!define VERSION      "1.0.0"
!define EXE          "aether-gui.exe"
!define UNINSTKEY    "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"

; perMachine matches tauri.conf.json's installMode.
!define MULTIUSER_EXECUTIONLEVEL Admin
!define MULTIUSER_MUI
!define MULTIUSER_INSTALLMODE_COMMANDLINE
!include "MultiUser.nsh"

Name "${APPNAME}"
OutFile "..\Aether-GUI-Setup-${VERSION}.exe"
InstallDir "$PROGRAMFILES64\${APPNAME}"
InstallDirRegKey HKLM "${UNINSTKEY}" "InstallLocation"
RequestExecutionLevel admin
ShowInstDetails show
ShowUnInstDetails show

SetCompressor /SOLID lzma

;--------------------------------------------------------------------
; Interface
;--------------------------------------------------------------------

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

;--------------------------------------------------------------------
; Install
;--------------------------------------------------------------------

Section "Install" SecInstall
    SetOutPath "$INSTDIR"

    ; The application and its two side-by-side DLLs.
    File "..\dist\aether-gui.exe"
    File "..\dist\wintun.dll"
    File "..\dist\WebView2Loader.dll"

    ; AGPL requires shipping the licence text alongside the binary.
    File "/oname=LICENSE.txt" "..\LICENSE"

    ; Shortcuts. The desktop one passes no flags, so a normal launch shows the
    ; window; the Start Menu entry is the same.
    CreateShortcut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\${EXE}"
    CreateDirectory "$SMPROGRAMS\${PUBLISHER}"
    CreateShortcut "$SMPROGRAMS\${PUBLISHER}\${APPNAME}.lnk" "$INSTDIR\${EXE}"
    CreateShortcut "$SMPROGRAMS\${PUBLISHER}\Uninstall ${APPNAME}.lnk" "$INSTDIR\Uninstall.exe"

    ; Registry: Add/Remove Programs entry + the app's own settings location hint.
    WriteRegStr HKLM "${UNINSTKEY}" "DisplayName"     "${APPNAME}"
    WriteRegStr HKLM "${UNINSTKEY}" "DisplayVersion"  "${VERSION}"
    WriteRegStr HKLM "${UNINSTKEY}" "Publisher"       "${PUBLISHER}"
    WriteRegStr HKLM "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "${UNINSTKEY}" "DisplayIcon"     "$INSTDIR\${EXE}"
    WriteRegStr HKLM "${UNINSTKEY}" "URLInfoAbout"    "https://t.me/net_republic"
    WriteRegDWORD HKLM "${UNINSTKEY}" "NoModify" 1
    WriteRegDWORD HKLM "${UNINSTKEY}" "NoRepair" 1

    ; EstimatedSize is in KiB, computed from what we actually installed.
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "${UNINSTKEY}" "EstimatedSize" "$0"

    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

;--------------------------------------------------------------------
; Uninstall
;--------------------------------------------------------------------

Section "Uninstall"
    ; Never delete user data silently: identities live under %APPDATA%\AetherGUI
    ; and survive reinstalls on purpose.

    Delete "$INSTDIR\${EXE}"
    Delete "$INSTDIR\wintun.dll"
    Delete "$INSTDIR\WebView2Loader.dll"
    Delete "$INSTDIR\LICENSE.txt"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir  "$INSTDIR"

    Delete "$DESKTOP\${APPNAME}.lnk"
    Delete "$SMPROGRAMS\${PUBLISHER}\${APPNAME}.lnk"
    Delete "$SMPROGRAMS\${PUBLISHER}\Uninstall ${APPNAME}.lnk"
    RMDir  "$SMPROGRAMS\${PUBLISHER}"

    DeleteRegKey HKLM "${UNINSTKEY}"
SectionEnd
