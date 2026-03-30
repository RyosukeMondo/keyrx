; KeyRx Inno Setup Installer Script
; Requires Inno Setup 6+
; Build: iscc /DAppVersion=1.0.0 keyrx-installer.iss
; Version passed via /D flag from build scripts (SSOT: Cargo.toml)

#define AppName "KeyRx"
#define AppPublisher "KeyRx Contributors"
#define AppURL "https://github.com/RyosukeMondo/keyrx"
#define AppExeName "keyrx_daemon.exe"

; Version must be passed via /DAppVersion=x.y.z from build script
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{A5B3C8D1-E9F2-4A7C-B6D3-1E8F9A2C5D7B}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes
LicenseFile=LICENSE
OutputDir=installer-output
OutputBaseFilename=keyrx-setup-v{#AppVersion}-windows-x64
SetupIconFile=keyrx_ui\public\favicon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
PrivilegesRequired=admin
UsedUserAreasWarning=no
UninstallDisplayIcon={app}\{#AppExeName}
VersionInfoVersion={#AppVersion}.0
VersionInfoCompany={#AppPublisher}
VersionInfoDescription=KeyRx Keyboard Remapping Installer
VersionInfoProductName={#AppName}
VersionInfoProductVersion={#AppVersion}.0
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Types]
Name: "full"; Description: "Full installation"
Name: "compact"; Description: "Compact installation"
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: "core"; Description: "KeyRx Core (daemon + compiler)"; Types: full compact custom; Flags: fixed
Name: "examples"; Description: "Example configurations"; Types: full
Name: "shortcuts"; Description: "Start Menu shortcuts"; Types: full compact

[Tasks]
Name: "addtopath"; Description: "Add to system PATH"; GroupDescription: "System integration:"; Flags: checkedonce
Name: "autostart"; Description: "Start KeyRx on Windows login"; GroupDescription: "System integration:"; Flags: unchecked
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Core binaries
Source: "target\release\keyrx_daemon.exe"; DestDir: "{app}"; Flags: ignoreversion; Components: core
Source: "target\release\keyrx_compiler.exe"; DestDir: "{app}"; Flags: ignoreversion; Components: core

; Documentation
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion; Components: core
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion; Components: core

; Example configurations
Source: "examples\*.rhai"; DestDir: "{app}\examples"; Flags: ignoreversion; Components: examples

[Dirs]
Name: "{userappdata}\keyrx"
Name: "{userappdata}\keyrx\profiles"

[Icons]
Name: "{group}\KeyRx Daemon"; Filename: "{app}\{#AppExeName}"; Parameters: "run"; Components: shortcuts
Name: "{group}\KeyRx Web UI"; Filename: "http://localhost:9867"; Components: shortcuts
Name: "{group}\Configuration Folder"; Filename: "{userappdata}\keyrx\profiles"; Components: shortcuts
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"; Components: shortcuts
Name: "{autodesktop}\KeyRx"; Filename: "{app}\{#AppExeName}"; Parameters: "run"; Tasks: desktopicon

[Registry]
; Installation info
Root: HKLM; Subkey: "Software\KeyRx Contributors\KeyRx"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\KeyRx Contributors\KeyRx"; ValueType: string; ValueName: "Version"; ValueData: "{#AppVersion}"; Flags: uninsdeletekey

; Add to PATH (optional task)
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}'))

; Auto-start on login (optional task)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "KeyRx"; ValueData: """{app}\{#AppExeName}"" run"; Flags: uninsdeletevalue; Tasks: autostart

; File associations (.krx)
Root: HKCR; Subkey: ".krx"; ValueType: string; ValueData: "KeyRxConfig"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyRxConfig"; ValueType: string; ValueData: "KeyRx Configuration"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyRxConfig\DefaultIcon"; ValueType: string; ValueData: "{app}\{#AppExeName},0"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyRxConfig\shell\open\command"; ValueType: string; ValueData: """{app}\{#AppExeName}"" run --config ""%1"""; Flags: uninsdeletekey

; File associations (.rhai)
Root: HKCR; Subkey: ".rhai"; ValueType: string; ValueData: "KeyRxScript"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyRxScript"; ValueType: string; ValueData: "KeyRx Rhai Script"; Flags: uninsdeletekey
Root: HKCR; Subkey: "KeyRxScript\DefaultIcon"; ValueType: string; ValueData: "{app}\keyrx_compiler.exe,0"; Flags: uninsdeletekey

[Run]
Filename: "{app}\{#AppExeName}"; Parameters: "run"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill"; Parameters: "/F /IM keyrx_daemon.exe"; Flags: runhidden; RunOnceId: "StopDaemon"

[Code]
function NeedsAddPath(Param: string): Boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath)
  then begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Uppercase(Param) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;

procedure RemoveFromPath(const Dir: string);
var
  OrigPath, UpperDir, UpperPath: string;
  P: Integer;
begin
  if not RegQueryStringValue(HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath) then
    exit;
  UpperDir := Uppercase(Dir);
  UpperPath := Uppercase(OrigPath);
  P := Pos(';' + UpperDir, ';' + UpperPath);
  if P > 0 then
  begin
    if P = 1 then
      Delete(OrigPath, 1, Length(Dir) + 1)
    else
      Delete(OrigPath, P, Length(Dir) + 1);
    RegWriteStringValue(HKLM,
      'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
      'Path', OrigPath);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    { Remove from PATH on uninstall }
    RemoveFromPath(ExpandConstant('{app}'));
    { Ask about config removal }
    if MsgBox('Remove configuration files from ' + ExpandConstant('{userappdata}\keyrx') + '?',
              mbConfirmation, MB_YESNO) = IDYES then
    begin
      DelTree(ExpandConstant('{userappdata}\keyrx'), True, True, True);
    end;
  end;
end;
