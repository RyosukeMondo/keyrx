; Inno Setup Script for KeyRx
; https://jrsoftware.org/isinfo.php
;
; To build: iscc keyrx-installer.iss

#define AppName "KeyRx"
#define AppVersion GetVersionNumbersString("..\..\target\release\keyrx_daemon.exe")
#define AppPublisher "KeyRx Project"
#define AppURL "https://github.com/RyosukeMondo/keyrx"
#define AppExeName "keyrx_daemon.exe"
#define AppIcon "..\..\keyrx_daemon\assets\icon.ico"

[Setup]
; Basic information
AppId={{A5B3C8D1-E9F2-4A7C-B6D3-1E8F9A2C5D7B}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
; Use auto directory (Program Files if admin, LocalAppData if not)
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
OutputDir=..\..\target\windows-installer
OutputBaseFilename=keyrx_{#AppVersion}_x64_setup
SetupIconFile={#AppIcon}
UninstallDisplayIcon={app}\{#AppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

; License
LicenseFile=..\..\LICENSE

; Privileges
; Changed to lowest to allow non-admin install, PATH will be per-user if not admin
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline dialog

; Visual appearance
DisableProgramGroupPage=yes
DisableWelcomePage=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startmenu"; Description: "Create Start Menu shortcuts"; GroupDescription: "{cm:AdditionalIcons}"
Name: "addtopath"; Description: "Add KeyRx to PATH (system-wide if admin, current user otherwise)"; GroupDescription: "System Integration:"
Name: "autostart"; Description: "Auto-start daemon on Windows login (with administrator privileges)"; GroupDescription: "System Integration:"; Check: IsAdmin

[Files]
; Main binaries
Source: "..\..\target\release\keyrx_daemon.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\keyrx_compiler.exe"; DestDir: "{app}"; Flags: ignoreversion

; Documentation
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

; Example config
Source: "..\..\examples\user_layout.krx"; DestDir: "{commonappdata}\keyrx"; Flags: ignoreversion

[Dirs]
; Create config directory
Name: "{commonappdata}\keyrx"; Permissions: users-modify

[Icons]
; Start Menu shortcuts - daemon runs as admin for keyboard access
Name: "{group}\KeyRx Daemon"; Filename: "{app}\{#AppExeName}"; Parameters: "run"; Comment: "Start KeyRx keyboard remapping daemon"
Name: "{group}\KeyRx Web UI"; Filename: "http://localhost:9867"; IconFilename: "{app}\{#AppExeName}"; Comment: "Open KeyRx web interface (daemon must be running)"
Name: "{group}\KeyRx Compiler"; Filename: "{app}\keyrx_compiler.exe"; Comment: "Compile KeyRx configuration files"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Parameters: "run"; Comment: "Start KeyRx daemon"; Tasks: desktopicon

[Registry]
; Add to PATH (system-wide if admin, per-user otherwise)
Root: HKA; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
    ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; \
    Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}')); \
    Flags: uninsdeletekeyifempty

; Fallback: Add to user PATH if not admin
Root: HKCU; Subkey: "Environment"; \
    ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; \
    Tasks: addtopath; Check: not IsAdmin and NeedsAddPath(ExpandConstant('{app}'))

; App registration
Root: HKA; Subkey: "Software\KeyRx"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"
Root: HKA; Subkey: "Software\KeyRx"; ValueType: string; ValueName: "Version"; ValueData: "{#AppVersion}"

[Run]
; Create Windows Task Scheduler entry for auto-start with admin rights (if user selected autostart)
Filename: "schtasks.exe"; Parameters: "/Create /TN ""KeyRx Daemon"" /TR ""\""{app}\{#AppExeName}\"" run"" /SC ONLOGON /RL HIGHEST /F /DELAY 0000:05"; Flags: runhidden; Tasks: autostart; StatusMsg: "Creating auto-start task with administrator privileges..."
; Start daemon immediately after install (checked by default, requires UAC)
Filename: "{app}\{#AppExeName}"; Parameters: "run"; Description: "Launch {#AppName} Daemon now (requires administrator permission)"; Flags: nowait postinstall skipifsilent shellexec

[UninstallRun]
; Stop daemon before uninstall
Filename: "taskkill"; Parameters: "/F /IM keyrx_daemon.exe"; Flags: runhidden; RunOnceId: "StopDaemon"
; Remove Task Scheduler entry
Filename: "schtasks.exe"; Parameters: "/Delete /TN ""KeyRx Daemon"" /F"; Flags: runhidden; RunOnceId: "RemoveTask"

[Code]
// Check if PATH already contains the app directory
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
  RootKey: Integer;
begin
  // Check both system and user PATH
  if IsAdmin then
    RootKey := HKEY_LOCAL_MACHINE
  else
    RootKey := HKEY_CURRENT_USER;

  if IsAdmin then begin
    if RegQueryStringValue(HKEY_LOCAL_MACHINE,
      'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
      'Path', OrigPath) then begin
      Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
      exit;
    end;
  end else begin
    if RegQueryStringValue(HKEY_CURRENT_USER,
      'Environment',
      'Path', OrigPath) then begin
      Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
      exit;
    end;
  end;

  Result := True;
end;

// Check if daemon is running and offer to stop it
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  if CheckForMutexes('keyrx_daemon') then
  begin
    if MsgBox('KeyRx daemon is currently running. Setup will now close it.' + #13#10 + 'Continue?',
      mbConfirmation, MB_YESNO) = IDYES then
    begin
      Exec('taskkill', '/F /IM keyrx_daemon.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end
    else
    begin
      Result := False;
    end;
  end;
end;

// Determine if we should start daemon after install
function ShouldStartDaemon(): Boolean;
begin
  // Start by default, user can uncheck
  Result := True;
end;
