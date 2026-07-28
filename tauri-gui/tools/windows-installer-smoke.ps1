param(
  [Parameter(Mandatory = $true)]
  [string]$InstallerPath,
  [string]$ExpectedSha256 = "",
  [int]$TimeoutSeconds = 180
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Wait-Until([scriptblock]$Condition, [int]$Seconds, [string]$FailureMessage) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $value = & $Condition
    if ($value) { return $value }
    Start-Sleep -Milliseconds 250
  } until ((Get-Date) -gt $deadline)
  throw $FailureMessage
}

if ($env:USERNAME -eq "SYSTEM") {
  throw "Run this smoke test as the interactive Windows user, not SYSTEM"
}

$InstallerPath = (Resolve-Path $InstallerPath).Path
$installerFile = Get-Item $InstallerPath
$candidateSha256 = (Get-FileHash $InstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($ExpectedSha256 -and $candidateSha256 -ne $ExpectedSha256.ToLowerInvariant()) {
  throw "Installer SHA256 mismatch: expected $ExpectedSha256, got $candidateSha256"
}

Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
$installer = Start-Process -FilePath $InstallerPath -ArgumentList "/S" -PassThru
$assistantStartedDuringInstall = $false
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while (!$installer.HasExited -and (Get-Date) -lt $deadline) {
  if (Get-Process codex-assistant -ErrorAction SilentlyContinue) {
    $assistantStartedDuringInstall = $true
  }
  Start-Sleep -Milliseconds 100
  $installer.Refresh()
}
if (!$installer.HasExited) {
  try { $installer.Kill() } catch {}
  throw "Installer did not finish within $TimeoutSeconds seconds"
}
if ($installer.ExitCode -ne 0) {
  throw "Installer failed with exit code $($installer.ExitCode)"
}
Start-Sleep -Seconds 1
if ($assistantStartedDuringInstall -or (Get-Process codex-assistant -ErrorAction SilentlyContinue)) {
  throw "Codex Assistant started before the installer completion flow requested it"
}

$registration = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*" `
    -ErrorAction SilentlyContinue |
  Where-Object { $_.MainBinaryName -eq "codex-assistant.exe" } |
  Select-Object -First 1
if (!$registration) {
  throw "Codex Assistant per-user uninstall registration was not found"
}
$installRoot = $registration.InstallLocation.Trim([char]34)
$assistantExe = Join-Path $installRoot "codex-assistant.exe"
if (!(Test-Path $assistantExe)) { throw "Installed executable was not found at $assistantExe" }

Start-Process -FilePath $assistantExe
$firstProcess = Wait-Until {
  $process = Get-Process codex-assistant -ErrorAction SilentlyContinue |
    Select-Object -First 1
  if ($process -and $process.Responding) { $process } else { $null }
} 20 "Codex Assistant did not become responsive after launch"
Start-Process -FilePath $assistantExe
Start-Sleep -Seconds 2
$processes = @(Get-Process codex-assistant -ErrorAction SilentlyContinue)
if ($processes.Count -ne 1) {
  throw "Single-instance check failed; process count is $($processes.Count)"
}
if (@($processes | Where-Object { !$_.Responding }).Count -ne 0) {
  throw "Codex Assistant process is not responding"
}

$result = [ordered]@{
  schemaVersion = 1
  candidate = $installerFile.Name
  candidateBytes = $installerFile.Length
  candidateSha256 = $candidateSha256
  osArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
  installerExitCode = $installer.ExitCode
  startedDuringInstall = $assistantStartedDuringInstall
  installedExecutable = $assistantExe
  displayVersion = $registration.DisplayVersion
  productVersion = (Get-Item $assistantExe).VersionInfo.ProductVersion
  firstLaunchResponding = $firstProcess.Responding
  processCountAfterDoubleLaunch = $processes.Count
}
$result | ConvertTo-Json -Compress
$processes | Stop-Process -Force
