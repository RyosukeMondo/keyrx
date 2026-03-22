; KeyRx NSIS Installer Script
; Requires NSIS 3.0+
; Build: makensis /DVERSION=1.0.0 keyrx-installer.nsi
; Version is passed via -DVERSION=x.y.z from build script (SSOT: Cargo.toml)

;--------------------------------
; Includes

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"

;--------------------------------
; Version from command line (default: 0.0.0)

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

;--------------------------------
; General

Name "KeyRx"
OutFile "..\..\installer-output\keyrx-setup-v${VERSION}-windows-x64.exe"
Unicode True

InstallDir "$PROGRAMFILES64\KeyRx"
InstallDirRegKey HKLM "Software\KeyRx Contributors\KeyRx" "InstallPath"
RequestExecutionLevel admin

SetCompressor /SOLID lzma
SetCompressorDictSize 32

;--------------------------------
; Version Information

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "KeyRx"
VIAddVersionKey "CompanyName" "KeyRx Contributors"
VIAddVersionKey "LegalCopyright" "© 2024-2026 KeyRx Contributors"
VIAddVersionKey "FileDescription" "KeyRx Keyboard Remapping Installer"
VIAddVersionKey "FileVersion" "${VERSION}.0"
VIAddVersionKey "ProductVersion" "${VERSION}.0"

;--------------------------------
; Interface Settings

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\keyrx_ui\public\favicon.ico"
!define MUI_UNICON "..\..\keyrx_ui\public\favicon.ico"

;--------------------------------
; Pages

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
; Languages

!insertmacro MUI_LANGUAGE "English"

;--------------------------------
; Installer Sections

Section "KeyRx Core" SecCore

  SectionIn RO

  SetOutPath "$INSTDIR"

  ; Core binaries
  File "..\..\target\release\keyrx_daemon.exe"
  File "..\..\target\release\keyrx_compiler.exe"

  ; Documentation
  File "..\..\README.md"
  File "..\..\LICENSE"

  ; Create user data directories
  SetOutPath "$APPDATA\keyrx\profiles"
  SetOutPath "$APPDATA\keyrx"

  ; Store installation folder
  WriteRegStr HKLM "Software\KeyRx Contributors\KeyRx" "InstallPath" "$INSTDIR"
  WriteRegStr HKLM "Software\KeyRx Contributors\KeyRx" "Version" "${VERSION}"

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; File association (.krx)
  WriteRegStr HKCR ".krx" "" "KeyRxConfig"
  WriteRegStr HKCR "KeyRxConfig" "" "KeyRx Configuration"
  WriteRegStr HKCR "KeyRxConfig\DefaultIcon" "" "$INSTDIR\keyrx_daemon.exe,0"
  WriteRegStr HKCR "KeyRxConfig\shell\open\command" "" '"$INSTDIR\keyrx_daemon.exe" run --config "%1"'

  ; File association (.rhai)
  WriteRegStr HKCR ".rhai" "" "KeyRxScript"
  WriteRegStr HKCR "KeyRxScript" "" "KeyRx Rhai Script"
  WriteRegStr HKCR "KeyRxScript\DefaultIcon" "" "$INSTDIR\keyrx_compiler.exe,0"

  ; Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayName" "KeyRx"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayIcon" "$INSTDIR\keyrx_daemon.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "Publisher" "KeyRx Contributors"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "URLInfoAbout" "https://github.com/RyosukeMondo/keyrx"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "NoRepair" 1

  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "EstimatedSize" "$0"

SectionEnd

Section "Example Configurations" SecExamples

  SetOutPath "$INSTDIR\examples"
  File /r "..\..\examples\*.rhai"

SectionEnd

Section "Start Menu Shortcuts" SecStartMenu

  CreateDirectory "$SMPROGRAMS\KeyRx"

  CreateShortcut "$SMPROGRAMS\KeyRx\KeyRx Daemon.lnk" "$INSTDIR\keyrx_daemon.exe" "run" "$INSTDIR\keyrx_daemon.exe" 0
  CreateShortcut "$SMPROGRAMS\KeyRx\KeyRx Web UI.lnk" "http://localhost:9867"
  CreateShortcut "$SMPROGRAMS\KeyRx\Configuration Folder.lnk" "$APPDATA\keyrx\profiles"
  CreateShortcut "$SMPROGRAMS\KeyRx\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

SectionEnd

Section "Desktop Shortcut" SecDesktop

  CreateShortcut "$DESKTOP\KeyRx.lnk" "$INSTDIR\keyrx_daemon.exe" "run" "$INSTDIR\keyrx_daemon.exe" 0

SectionEnd

;--------------------------------
; Descriptions

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecCore} "Core KeyRx components (daemon with embedded web UI, compiler)"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecExamples} "Sample Rhai configuration files"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "Create Start Menu shortcuts"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create desktop shortcut"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
; Installer Functions

Function .onInit

  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "KeyRx requires 64-bit Windows 10 or later."
    Abort
  ${EndIf}

  ${If} ${AtMostWin8.1}
    MessageBox MB_OK|MB_ICONSTOP "KeyRx requires Windows 10 version 1809 or later."
    Abort
  ${EndIf}

FunctionEnd

Function .onInstSuccess

  MessageBox MB_YESNO "KeyRx has been installed successfully.$\n$\nLaunch KeyRx now?" IDNO +2
  Exec '"$INSTDIR\keyrx_daemon.exe" run'

FunctionEnd

;--------------------------------
; Uninstaller Section

Section "Uninstall"

  ; Stop daemon if running
  nsExec::ExecToLog 'taskkill /F /IM keyrx_daemon.exe'

  ; Remove files
  Delete "$INSTDIR\keyrx_daemon.exe"
  Delete "$INSTDIR\keyrx_compiler.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\Uninstall.exe"

  RMDir /r "$INSTDIR\examples"
  RMDir "$INSTDIR"

  ; Remove shortcuts
  RMDir /r "$SMPROGRAMS\KeyRx"
  Delete "$DESKTOP\KeyRx.lnk"

  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx"
  DeleteRegKey HKLM "Software\KeyRx Contributors\KeyRx"
  DeleteRegKey HKCR ".krx"
  DeleteRegKey HKCR "KeyRxConfig"
  DeleteRegKey HKCR ".rhai"
  DeleteRegKey HKCR "KeyRxScript"

  ; Ask about user data
  MessageBox MB_YESNO|MB_ICONQUESTION "Remove configuration files from $APPDATA\keyrx?" IDNO +2
  RMDir /r "$APPDATA\keyrx"

SectionEnd

Function un.onUninstSuccess

  MessageBox MB_OK "KeyRx has been uninstalled successfully."

FunctionEnd
