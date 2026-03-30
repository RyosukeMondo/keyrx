[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=%InstallPrompt%
DisplayLicense=%DisplayLicense%
FinishMessage=%FinishMessage%
TargetName=%TargetName%
FriendlyName=%FriendlyName%
AppLaunched=%AppLaunched%
PostInstallCmd=%PostInstallCmd%
AdminQuietInstCmd=%AdminQuietInstCmd%
UserQuietInstCmd=%UserQuietInstCmd%
SourceFiles=SourceFiles

[Strings]
InstallPrompt=KeyRx v1.0.0 - Advanced Keyboard Remapping. Click OK to install.
DisplayLicense=LICENSE
FinishMessage=KeyRx v1.0.0 has been installed successfully! You can now run keyrx_daemon from the command line.
TargetName=.\keyrx-installer-v1.0.0.exe
FriendlyName=KeyRx Installer v1.0.0
AppLaunched=powershell.exe -ExecutionPolicy Bypass -File install.ps1
PostInstallCmd=<None>
AdminQuietInstCmd=powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Silent
UserQuietInstCmd=powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Silent

[SourceFiles]
SourceFiles0=.\installer-temp\

[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
%FILE4%=
