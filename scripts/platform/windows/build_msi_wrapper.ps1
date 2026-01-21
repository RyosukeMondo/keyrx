# Wrapper script to build MSI with refreshed PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
& "$PSScriptRoot\build_windows_installer.ps1"
