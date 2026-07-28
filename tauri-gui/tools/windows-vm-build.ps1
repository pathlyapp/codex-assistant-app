param(
  [ValidateSet("auto", "x64", "arm64")]
  [string]$Architecture = "auto",
  [string]$LocalRoot = "C:\Temp\codex-assistant-vm-build"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$toolsDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$guiRoot = Split-Path -Parent $toolsDirectory
$sourceRoot = Split-Path -Parent $guiRoot
$sourceFull = [IO.Path]::GetFullPath($sourceRoot).TrimEnd("\")
$localFull = [IO.Path]::GetFullPath($LocalRoot).TrimEnd("\")
if ($sourceFull -eq $localFull) {
  throw "LocalRoot must be different from the shared source directory"
}

New-Item -ItemType Directory -Force $localFull | Out-Null
& robocopy.exe $sourceFull $localFull /MIR `
  /XD .git node_modules target artifact `
  /R:1 /W:1 /NFL /NDL /NJH /NJS /NP
$robocopyCode = $LASTEXITCODE
if ($robocopyCode -gt 7) {
  throw "Source synchronization failed with robocopy exit code $robocopyCode"
}

$localGui = Join-Path $localFull "tauri-gui"
$buildScript = Join-Path $localGui "tools\windows-build.ps1"
if (!(Test-Path $buildScript)) {
  throw "Synchronized Windows build script was not found at $buildScript"
}

$arguments = @(
  "-NoProfile",
  "-ExecutionPolicy", "Bypass",
  "-File", $buildScript,
  "-Architecture", $Architecture
)
if (Test-Path (Join-Path $localGui "node_modules\@tauri-apps\cli")) {
  $arguments += "-SkipNpmInstall"
}

& powershell.exe @arguments
if ($LASTEXITCODE -ne 0) {
  throw "Local Windows build failed with exit code $LASTEXITCODE"
}
