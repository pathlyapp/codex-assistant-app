param(
  [Parameter(Mandatory = $true)]
  [string]$BaselineInstallerPath,
  [Parameter(Mandatory = $true)]
  [string]$UpdateArtifactPath,
  [Parameter(Mandatory = $true)]
  [string]$UpdateSignaturePath,
  [Parameter(Mandatory = $true)]
  [string]$UpdateVersion,
  [string]$ExpectedCurrentVersion = "0.9.0",
  [string]$ServerScriptPath = "",
  [int]$UpdatePort = 43123,
  [int]$DebugPort = 9231,
  [int]$TimeoutSeconds = 180
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

. (Join-Path $PSScriptRoot "windows-arch.ps1")

if ($env:USERNAME -eq "SYSTEM") {
  throw "Run this updater E2E as the interactive Windows user, not SYSTEM"
}

function Wait-Until([scriptblock]$Condition, [int]$Seconds, [string]$FailureMessage) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $value = & $Condition
    if ($value) { return $value }
    Start-Sleep -Milliseconds 250
  } until ((Get-Date) -gt $deadline)
  throw $FailureMessage
}

function Invoke-CdpExpression(
  [System.Net.WebSockets.ClientWebSocket]$Socket,
  [string]$Expression,
  [int]$Id
) {
  $request = @{
    id = $Id
    method = "Runtime.evaluate"
    params = @{
      expression = $Expression
      awaitPromise = $true
      returnByValue = $true
    }
  } | ConvertTo-Json -Depth 8 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($request)
  [void]$Socket.SendAsync(
    [ArraySegment[byte]]::new($bytes),
    [Net.WebSockets.WebSocketMessageType]::Text,
    $true,
    [Threading.CancellationToken]::None
  ).GetAwaiter().GetResult()

  while ($true) {
    $buffer = New-Object byte[] 131072
    $received = New-Object Text.StringBuilder
    do {
      $part = $Socket.ReceiveAsync(
        [ArraySegment[byte]]::new($buffer),
        [Threading.CancellationToken]::None
      ).GetAwaiter().GetResult()
      if ($part.MessageType -eq [Net.WebSockets.WebSocketMessageType]::Close) {
        throw "CDP socket closed before command $Id completed"
      }
      [void]$received.Append([Text.Encoding]::UTF8.GetString($buffer, 0, $part.Count))
    } until ($part.EndOfMessage)

    $response = $received.ToString() | ConvertFrom-Json
    if ($response.id -ne $Id) { continue }
    if ($response.error) { throw "CDP command failed: $($response.error.message)" }
    if ($response.result.exceptionDetails) {
      $description = $response.result.exceptionDetails.exception.description
      if (!$description) { $description = $response.result.exceptionDetails.text }
      if (!$description -or $description -eq "Object") {
        $description = $response.result.exceptionDetails | ConvertTo-Json -Depth 8 -Compress
      }
      throw "Browser expression failed: $description"
    }
    return $response.result.result.value
  }
}

function Get-AssistantRegistration {
  Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*" `
      -ErrorAction SilentlyContinue |
    Where-Object { $_.MainBinaryName -eq "codex-assistant.exe" } |
    Select-Object -First 1
}

$BaselineInstallerPath = (Resolve-Path $BaselineInstallerPath).Path
$UpdateArtifactPath = (Resolve-Path $UpdateArtifactPath).Path
$UpdateSignaturePath = (Resolve-Path $UpdateSignaturePath).Path
if (!$ServerScriptPath) {
  $ServerScriptPath = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) `
    "updater-test-server.mjs"
}
$ServerScriptPath = (Resolve-Path $ServerScriptPath).Path

$server = $null
$socket = $null
try {
  Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
  $installer = Start-Process -FilePath $BaselineInstallerPath -ArgumentList "/S" -PassThru -Wait
  if ($installer.ExitCode -ne 0) {
    throw "Baseline installer failed with exit code $($installer.ExitCode)"
  }
  $registration = Get-AssistantRegistration
  if (!$registration -or $registration.DisplayVersion -ne $ExpectedCurrentVersion) {
    throw "Baseline version $ExpectedCurrentVersion was not registered"
  }
  $assistantExe = Join-Path $registration.InstallLocation.Trim([char]34) "codex-assistant.exe"
  if (!(Test-Path $assistantExe)) { throw "Installed assistant executable was not found" }

  $serverLog = Join-Path $env:TEMP "codex-assistant-updater-e2e-server.log"
  $serverError = Join-Path $env:TEMP "codex-assistant-updater-e2e-server.err.log"
  $server = Start-Process -FilePath "node.exe" -ArgumentList @(
    $ServerScriptPath,
    "--artifact", $UpdateArtifactPath,
    "--signature", $UpdateSignaturePath,
    "--version", $UpdateVersion,
    "--target", "windows",
    "--arch", "aarch64",
    "--port", "$UpdatePort"
  ) -RedirectStandardOutput $serverLog -RedirectStandardError $serverError -PassThru
  [void](Wait-Until {
    try {
      $health = Invoke-RestMethod -Uri "http://127.0.0.1:$UpdatePort/health" -TimeoutSec 2
      if ($health.version -eq $UpdateVersion) { $health } else { $null }
    } catch { $null }
  } 20 "Updater test server did not become ready")

  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"
  Start-Process -FilePath $assistantExe
  $targets = Wait-Until {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:$DebugPort/json/list" -TimeoutSec 2 } catch { $null }
  } 20 "Codex Assistant WebView2 debug endpoint did not start"
  $target = $targets |
    Where-Object {
      $_.type -eq "page" -and (
        $_.url -like "http://tauri.localhost/*" -or
        $_.url -like "https://tauri.localhost/*"
      )
    } |
    Select-Object -First 1
  if (!$target) {
    $summary = @($targets | Select-Object type, url, title) | ConvertTo-Json -Compress
    throw "Codex Assistant page target was not found: $summary"
  }

  $socket = [Net.WebSockets.ClientWebSocket]::new()
  [void]$socket.ConnectAsync(
    [Uri]$target.webSocketDebuggerUrl,
    [Threading.CancellationToken]::None
  ).GetAwaiter().GetResult()
  $expectedCurrentJson = $ExpectedCurrentVersion | ConvertTo-Json -Compress
  $updateVersionJson = $UpdateVersion | ConvertTo-Json -Compress
  $uiEvidence = Invoke-CdpExpression $socket @"
(async () => {
  const deadline = Date.now() + 30000;
  while (!window.__TAURI__?.core && Date.now() < deadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  document.querySelector('[data-view="diagnostics"]').click();
  await new Promise(resolve => setTimeout(resolve, 500));
  const initial = await window.__TAURI__.core.invoke('get_assistant_update_status');
  if (initial.currentVersion !== $expectedCurrentJson || !initial.configured) {
    throw new Error('Baseline updater trust root is not configured: ' + JSON.stringify(initial));
  }
  const checkButton = document.querySelector('#checkAssistantUpdateButton');
  if (checkButton.classList.contains('hidden') || checkButton.disabled) {
    throw new Error('Update check button is not available');
  }
  checkButton.click();
  while (document.querySelector('#assistantUpdateState').dataset.phase !== 'available' &&
         Date.now() < deadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const checked = await window.__TAURI__.core.invoke('get_assistant_update_status');
  if (checked.phase !== 'available' || checked.availableVersion !== $updateVersionJson) {
    throw new Error('Expected update was not offered: ' + JSON.stringify(checked));
  }

  const downloadButton = document.querySelector('#downloadAssistantUpdateButton');
  if (downloadButton.classList.contains('hidden') || downloadButton.disabled) {
    throw new Error('Update download button is not available');
  }
  downloadButton.click();
  while (document.querySelector('#assistantUpdateState').dataset.phase !== 'ready_to_install' &&
         Date.now() < deadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const downloaded = await window.__TAURI__.core.invoke('get_assistant_update_status');
  if (downloaded.phase !== 'ready_to_install' ||
      downloaded.verification !== 'verified' ||
      downloaded.progressPercent !== 100) {
    throw new Error('Update did not reach signed ready state: ' + JSON.stringify(downloaded));
  }
  const badge = document.querySelector('#assistantUpdateState');
  const button = document.querySelector('#installAssistantUpdateButton');
  if (badge.dataset.phase !== 'ready_to_install' ||
      badge.dataset.verification !== 'verified' ||
      button.classList.contains('hidden') ||
      button.disabled) {
    throw new Error('Verified update UI is inconsistent: ' + JSON.stringify({
      phase: badge.dataset.phase,
      verification: badge.dataset.verification,
      hidden: button.classList.contains('hidden'),
      disabled: button.disabled
    }));
  }
  button.click();
  const confirmDeadline = Date.now() + 5000;
  while (document.querySelector('#confirmOverlay').classList.contains('hidden') &&
         Date.now() < confirmDeadline) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (document.querySelector('#confirmOverlay').classList.contains('hidden')) {
    throw new Error('Install confirmation did not open');
  }
  document.querySelector('#confirmAcceptButton').click();
  return {
    checkedPhase: checked.phase,
    downloadedPhase: downloaded.phase,
    verification: downloaded.verification,
    progressPercent: downloaded.progressPercent,
    renderedPhase: badge.dataset.phase
  };
})()
"@ 1
  $socket.Dispose()
  $socket = $null

  $updatedRegistration = Wait-Until {
    $value = Get-AssistantRegistration
    if ($value -and $value.DisplayVersion -eq $UpdateVersion) { $value } else { $null }
  } $TimeoutSeconds "Updated version $UpdateVersion was not registered"
  $updatedExe = Join-Path $updatedRegistration.InstallLocation.Trim([char]34) "codex-assistant.exe"
  $updatedProcess = Wait-Until {
    $process = Get-Process codex-assistant -ErrorAction SilentlyContinue |
      Where-Object {
        try { $_.Path -eq $updatedExe -and $_.Responding } catch { $false }
      } |
      Select-Object -First 1
    $process
  } 60 "Updated assistant did not relaunch responsively"

  $receiptEvidence = Wait-Until {
    $receiptFiles = @(
      Get-ChildItem $env:APPDATA, $env:LOCALAPPDATA -Filter "update-state.json" `
        -Recurse -Depth 6 -ErrorAction SilentlyContinue
    ) | Sort-Object LastWriteTime -Descending
    foreach ($receiptFile in $receiptFiles) {
      try {
        $receipt = Get-Content $receiptFile.FullName -Raw | ConvertFrom-Json
        if ($receipt.toVersion -eq $UpdateVersion -and $receipt.status -eq "healthy") {
          return [ordered]@{
            path = $receiptFile.FullName
            fromVersion = $receipt.fromVersion
            toVersion = $receipt.toVersion
            status = $receipt.status
          }
        }
      } catch {}
    }
    $null
  } 60 "Updated assistant did not confirm a healthy update receipt"

  [ordered]@{
    schemaVersion = 1
    architecture = Get-WindowsNativeArchitecture
    baselineVersion = $ExpectedCurrentVersion
    updateVersion = $UpdateVersion
    baselineInstallerSha256 = (Get-FileHash $BaselineInstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
    updateArtifactSha256 = (Get-FileHash $UpdateArtifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
    updateSignatureSha256 = (Get-FileHash $UpdateSignaturePath -Algorithm SHA256).Hash.ToLowerInvariant()
    ui = $uiEvidence
    registeredVersion = $updatedRegistration.DisplayVersion
    productVersion = (Get-Item $updatedExe).VersionInfo.ProductVersion
    relaunchedResponding = $updatedProcess.Responding
    receipt = $receiptEvidence
  } | ConvertTo-Json -Depth 6 -Compress
} finally {
  if ($socket) { $socket.Dispose() }
  if ($server -and !$server.HasExited) {
    Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
  }
  Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
}
