; KeyRx NSIS Installer Script
; Requires NSIS 3.0+
; Download from: https://nsis.sourceforge.io/
; Build: makensis keyrx-installer.nsi

;--------------------------------
; Includes

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"

;--------------------------------
; General

Name "KeyRx"
OutFile "..\..\installer-output\keyrx-setup-v0.1.5-windows-x64.exe"
Unicode True

; Default installation folder
InstallDir "$PROGRAMFILES64\KeyRx"

; Get installation folder from registry if available
InstallDirRegKey HKLM "Software\KeyRx Contributors\KeyRx" "InstallPath"

; Request application privileges
RequestExecutionLevel admin

; Compression
SetCompressor /SOLID lzma
SetCompressorDictSize 32

;--------------------------------
; Version Information

VIProductVersion "0.1.5.0"
VIAddVersionKey "ProductName" "KeyRx"
VIAddVersionKey "CompanyName" "KeyRx Contributors"
VIAddVersionKey "LegalCopyright" "© 2024 KeyRx Contributors"
VIAddVersionKey "FileDescription" "KeyRx Keyboard Remapping Installer"
VIAddVersionKey "FileVersion" "0.1.5.0"
VIAddVersionKey "ProductVersion" "0.1.5.0"

;--------------------------------
; Interface Settings

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\keyrx_ui\public\favicon.ico"
!define MUI_UNICON "..\..\keyrx_ui\public\favicon.ico"
!define MUI_HEADERIMAGE
!define MUI_WELCOMEFINISHPAGE_BITMAP "..\..\installer-assets\wizard.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "..\..\installer-assets\wizard.bmp"

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

  SectionIn RO  ; Read-only, cannot be deselected

  SetOutPath "$INSTDIR"

  ; Core binaries
  File "..\..\target\release\keyrx_daemon.exe"
  File "..\..\target\release\keyrx_compiler.exe"

  ; Documentation
  File "..\..\README.md"
  File "..\..\LICENSE"

  ; Web UI
  SetOutPath "$INSTDIR\ui"
  File /r "..\..\keyrx_ui\dist\*.*"

  ; Create user data directories
  SetOutPath "$APPDATA\KeyRx\configs"
  SetOutPath "$APPDATA\KeyRx\logs"

  ; Store installation folder
  WriteRegStr HKLM "Software\KeyRx Contributors\KeyRx" "InstallPath" "$INSTDIR"
  WriteRegStr HKLM "Software\KeyRx Contributors\KeyRx" "Version" "0.1.5"

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Add to PATH
  EnVar::SetHKLM
  EnVar::AddValue "PATH" "$INSTDIR"
  Pop $0

  ; File association
  WriteRegStr HKCR ".krx" "" "KeyRxConfig"
  WriteRegStr HKCR "KeyRxConfig" "" "KeyRx Configuration"
  WriteRegStr HKCR "KeyRxConfig\DefaultIcon" "" "$INSTDIR\keyrx_daemon.exe,0"
  WriteRegStr HKCR "KeyRxConfig\shell\open\command" "" '"$INSTDIR\keyrx_daemon.exe" "%1"'

  ; Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayName" "KeyRx"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayIcon" "$INSTDIR\keyrx_daemon.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "Publisher" "KeyRx Contributors"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "DisplayVersion" "0.1.5"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "NoRepair" 1

  ; Get size
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx" "EstimatedSize" "$0"

SectionEnd

Section "Documentation" SecDocs

  SetOutPath "$INSTDIR\docs"
  File /r "..\..\docs\*.*"

SectionEnd

Section "Example Configurations" SecExamples

  SetOutPath "$INSTDIR\examples"
  File /r "..\..\examples\*.*"

SectionEnd

Section "Start Menu Shortcuts" SecStartMenu

  CreateDirectory "$SMPROGRAMS\KeyRx"

  CreateShortcut "$SMPROGRAMS\KeyRx\KeyRx Daemon.lnk" "$INSTDIR\keyrx_daemon.exe" "--help" "$INSTDIR\keyrx_daemon.exe" 0
  CreateShortcut "$SMPROGRAMS\KeyRx\KeyRx Compiler.lnk" "$INSTDIR\keyrx_compiler.exe" "--help" "$INSTDIR\keyrx_compiler.exe" 0
  CreateShortcut "$SMPROGRAMS\KeyRx\KeyRx Web UI.lnk" "http://localhost:9867"
  CreateShortcut "$SMPROGRAMS\KeyRx\Configuration Folder.lnk" "$APPDATA\KeyRx\configs"
  CreateShortcut "$SMPROGRAMS\KeyRx\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

SectionEnd

Section "Desktop Shortcut" SecDesktop

  CreateShortcut "$DESKTOP\KeyRx.lnk" "$INSTDIR\keyrx_daemon.exe" "" "$INSTDIR\keyrx_daemon.exe" 0

SectionEnd

;--------------------------------
; Descriptions

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecCore} "Core KeyRx components (daemon, compiler, web UI)"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDocs} "User guides and documentation"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecExamples} "Sample configuration files"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "Create Start Menu shortcuts"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create desktop shortcut"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
; Installer Functions

Function .onInit

  ; Check if 64-bit Windows
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "KeyRx requires 64-bit Windows 10 or later."
    Abort
  ${EndIf}

  ; Check Windows version (Windows 10 build 17763+)
  ${If} ${AtMostWin8.1}
    MessageBox MB_OK|MB_ICONSTOP "KeyRx requires Windows 10 version 1809 (build 17763) or later."
    Abort
  ${EndIf}

  ; Check if daemon is running
  FindWindow $0 "" "keyrx_daemon"
  ${If} $0 != 0
    MessageBox MB_YESNO|MB_ICONQUESTION "KeyRx daemon is currently running. It will be stopped during installation. Continue?" IDYES +2
    Abort
  ${EndIf}

FunctionEnd

Function .onInstSuccess

  MessageBox MB_OK "KeyRx has been installed successfully.$\n$\nYou can now run 'keyrx_daemon' from any command prompt."

FunctionEnd

;--------------------------------
; Uninstaller Section

Section "Uninstall"

  ; Stop daemon if running
  KillProcWMI::KillProc "keyrx_daemon.exe"

  ; Remove files
  Delete "$INSTDIR\keyrx_daemon.exe"
  Delete "$INSTDIR\keyrx_compiler.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\Uninstall.exe"

  RMDir /r "$INSTDIR\ui"
  RMDir /r "$INSTDIR\docs"
  RMDir /r "$INSTDIR\examples"
  RMDir "$INSTDIR"

  ; Remove shortcuts
  RMDir /r "$SMPROGRAMS\KeyRx"
  Delete "$DESKTOP\KeyRx.lnk"

  ; Remove from PATH
  EnVar::SetHKLM
  EnVar::DeleteValue "PATH" "$INSTDIR"
  Pop $0

  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx"
  DeleteRegKey HKLM "Software\KeyRx Contributors\KeyRx"
  DeleteRegKey HKCR ".krx"
  DeleteRegKey HKCR "KeyRxConfig"

  ; Ask about user data
  MessageBox MB_YESNO|MB_ICONQUESTION "Remove configuration files from $APPDATA\KeyRx?" IDNO +2
  RMDir /r "$APPDATA\KeyRx"

SectionEnd

Function un.onUninstSuccess

  MessageBox MB_OK "KeyRx has been uninstalled successfully."

FunctionEnd
