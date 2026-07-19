param(
  [string]$InstallerPath = ""
)

$ErrorActionPreference = "Stop"

if (!$InstallerPath) {
  $InstallerPath = Get-ChildItem "C:\Temp\CodexAssistantBuild-0.8.0\src-tauri\target\release\bundle\nsis\*0.8.4*.exe" |
    Select-Object -First 1 -ExpandProperty FullName
}
if (!$InstallerPath -or !(Test-Path $InstallerPath)) {
  throw "Codex Assistant installer was not found"
}

Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
$installer = Start-Process -FilePath $InstallerPath -ArgumentList "/S" -PassThru
$assistantStartedDuringInstall = $false
$deadline = (Get-Date).AddMinutes(3)
while (!$installer.HasExited -and (Get-Date) -lt $deadline) {
  if (Get-Process codex-assistant -ErrorAction SilentlyContinue) {
    $assistantStartedDuringInstall = $true
  }
  Start-Sleep -Milliseconds 100
  $installer.Refresh()
}
if (!$installer.HasExited) {
  throw "Installer did not finish within 3 minutes"
}
if ($installer.ExitCode -ne 0) {
  throw "Installer failed with exit code $($installer.ExitCode)"
}
Start-Sleep -Seconds 1
if ($assistantStartedDuringInstall -or (Get-Process codex-assistant -ErrorAction SilentlyContinue)) {
  throw "Codex Assistant started before the installer completion flow requested it"
}

$assistantExe = Get-ChildItem $env:LOCALAPPDATA -Filter "codex-assistant.exe" -Recurse -Depth 4 |
  Select-Object -First 1 -ExpandProperty FullName
if (!$assistantExe) {
  throw "Installed codex-assistant.exe was not found"
}

Start-Process -FilePath $assistantExe
Start-Sleep -Seconds 2
Start-Process -FilePath $assistantExe
Start-Sleep -Seconds 2
$processes = @(Get-Process codex-assistant -ErrorAction SilentlyContinue)
if ($processes.Count -ne 1) {
  throw "Single-instance check failed; process count is $($processes.Count)"
}

$result = [ordered]@{
  installerExitCode = $installer.ExitCode
  startedDuringInstall = $assistantStartedDuringInstall
  installedExecutable = $assistantExe
  productVersion = (Get-Item $assistantExe).VersionInfo.ProductVersion
  processCountAfterDoubleLaunch = $processes.Count
}
$result | ConvertTo-Json -Compress
$processes | Stop-Process -Force
