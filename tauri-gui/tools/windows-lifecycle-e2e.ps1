param(
  [int]$DebugPort = 9225,
  [switch]$TestDefaultUninstall,
  [string]$InstallerPath = "",
  [string]$ExpectedSha256 = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Wait-Until([scriptblock]$Condition, [int]$TimeoutSeconds, [string]$FailureMessage) {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
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
    if ($response.id -eq $Id) {
      if ($response.error) { throw "CDP command failed: $($response.error.message)" }
      if ($response.result.exceptionDetails) {
        throw "Browser expression failed: $($response.result.exceptionDetails.text)"
      }
      return $response.result.result.value
    }
  }
}

function Get-AssistantRegistration {
  Get-ChildItem "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall" -ErrorAction SilentlyContinue |
    ForEach-Object { Get-ItemProperty $_.PSPath } |
    Where-Object { $_.MainBinaryName -eq "codex-assistant.exe" } |
    Select-Object -First 1
}

function Get-AssistantUninstallProcesses([string]$InstallRoot, [string]$UninstallerPath) {
  @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
      Where-Object {
        ($_.Name -ieq "uninstall.exe" -and $_.ExecutablePath -ieq $UninstallerPath) -or
        ($_.Name -ieq "Un.exe" -and $_.CommandLine -like "*$InstallRoot*")
      }
  )
}

if ([Security.Principal.WindowsIdentity]::GetCurrent().IsSystem) {
  throw "Lifecycle E2E must run as the signed-in interactive user"
}

$assistantExe = Get-ChildItem $env:LOCALAPPDATA -Filter "codex-assistant.exe" -Recurse -Depth 4 |
  Select-Object -First 1 -ExpandProperty FullName
if (!$assistantExe) { throw "Installed codex-assistant.exe was not found" }

$configPath = Join-Path $env:USERPROFILE ".codex\config.toml"
$dataRoot = Join-Path $env:LOCALAPPDATA "CodexAssistant"
if (!(Test-Path $configPath) -or !(Test-Path $dataRoot)) {
  throw "Lifecycle E2E requires an existing managed configuration and assistant data"
}

$config = Get-Content $configPath -Raw
if ($config -notmatch "codex_assistant_router" -and
    $config -notmatch "model_catalog_json") {
  throw "Lifecycle E2E requires an assistant-managed config"
}
$profileMarker = "[profiles.lifecycle_preserve]"
if ($config -notmatch [regex]::Escape($profileMarker)) {
  Add-Content -Path $configPath -Encoding UTF8 -Value @"

$profileMarker
model = "gpt-5"
"@
}

Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"
Start-Process -FilePath $assistantExe

$targets = Wait-Until {
  try { Invoke-RestMethod -Uri "http://127.0.0.1:$DebugPort/json/list" -TimeoutSec 2 } catch { $null }
} 20 "Codex Assistant WebView2 debug endpoint did not start"
$target = $targets |
  Where-Object { $_.type -eq "page" -and $_.url -eq "http://tauri.localhost/" } |
  Select-Object -First 1
if (!$target) { throw "Codex Assistant page target was not found" }

$socket = [Net.WebSockets.ClientWebSocket]::new()
[void]$socket.ConnectAsync(
  [Uri]$target.webSocketDebuggerUrl,
  [Threading.CancellationToken]::None
).GetAwaiter().GetResult()

try {
  $result = Invoke-CdpExpression $socket @'
(async () => {
  const readyDeadline = Date.now() + 20000;
  const refresh = document.querySelector('#refreshButton');
  while (refresh.disabled && Date.now() < readyDeadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  document.querySelector('[data-view="diagnostics"]').click();
  const uninstall = document.querySelector('#uninstallAssistantButton');
  const restoreButton = document.querySelector('#restoreManagedConfigButton');
  const deleteButton = document.querySelector('#deleteAssistantDataButton');
  const readyUiDeadline = Date.now() + 10000;
  while (
    (uninstall.disabled || restoreButton.disabled || !deleteButton.disabled) &&
    Date.now() < readyUiDeadline
  ) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const initial = await window.__TAURI__.core.invoke('get_lifecycle_status');

  let blockedDeleteCode = '';
  try {
    await window.__TAURI__.core.invoke('run_lifecycle_action', {
      request: {
        actionId: 'delete_assistant_data',
        confirmation: 'DELETE_ASSISTANT_DATA'
      }
    });
  } catch (error) {
    blockedDeleteCode = error?.code || '';
  }

  const runButtonAction = async (buttonSelector, expectedAction) => {
    const button = document.querySelector(buttonSelector);
    const buttonDeadline = Date.now() + 10000;
    while (button.disabled && Date.now() < buttonDeadline) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    if (button.disabled) throw new Error(expectedAction + ' button is disabled');
    const receipt = document.querySelector('#lifecycleResult');
    button.click();
    const overlay = document.querySelector('#confirmOverlay');
    const confirmDeadline = Date.now() + 5000;
    while (overlay.classList.contains('hidden') && Date.now() < confirmDeadline) {
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    if (overlay.classList.contains('hidden')) throw new Error(expectedAction + ' confirmation did not open');
    document.querySelector('#confirmAcceptButton').click();
    const actionDeadline = Date.now() + 30000;
    while (
      (receipt.dataset.actionId !== expectedAction || receipt.classList.contains('hidden')) &&
      Date.now() < actionDeadline
    ) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    if (receipt.dataset.actionId !== expectedAction || receipt.classList.contains('hidden')) {
      throw new Error(expectedAction + ' did not return a receipt');
    }
    return {
      actionId: receipt.dataset.actionId,
      status: receipt.dataset.status,
      changed: receipt.dataset.changed,
      beforeManagedConfig: receipt.dataset.beforeManagedConfig,
      afterManagedConfig: receipt.dataset.afterManagedConfig,
      beforeAssistantData: receipt.dataset.beforeAssistantData,
      afterAssistantData: receipt.dataset.afterAssistantData,
      text: receipt.textContent.trim()
    };
  };

  const restore = await runButtonAction(
    '#restoreManagedConfigButton',
    'restore_pre_assistant_config'
  );
  const afterRestore = await window.__TAURI__.core.invoke('get_lifecycle_status');
  const deleteData = await runButtonAction(
    '#deleteAssistantDataButton',
    'delete_assistant_data'
  );
  const final = await window.__TAURI__.core.invoke('get_lifecycle_status');
  return {
    initial,
    blockedDeleteCode,
    restore,
    afterRestore,
    deleteData,
    final,
    oldFactoryResetAbsent: !document.querySelector('#factoryResetButton')
  };
})()
'@ 1
} finally {
  $socket.Dispose()
  Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
}

$configAfter = Get-Content $configPath -Raw
if (!$result.initial.defaultPreservesConfig -or
    !$result.initial.defaultPreservesData -or
    !$result.initial.defaultPreservesOfficialApp -or
    !$result.initial.managedConfigPresent -or
    !$result.initial.assistantDataPresent -or
    !$result.initial.dataRemovalBlocked -or
    !$result.initial.officialAppInstalled -or
    !$result.initial.officialAppTrusted -or
    !$result.initial.assistantUninstallAvailable) {
  throw "Initial lifecycle status is invalid: $($result.initial | ConvertTo-Json -Compress)"
}
if ($result.blockedDeleteCode -ne "LIFECYCLE_DATA_IN_USE") {
  throw "Data deletion was not blocked by the managed-config dependency: $($result | ConvertTo-Json -Depth 8 -Compress)"
}
if ($result.restore.changed -ne "true" -or
    $result.restore.beforeManagedConfig -ne "true" -or
    $result.restore.afterManagedConfig -ne "false" -or
    $result.afterRestore.managedConfigPresent -or
    !$result.afterRestore.assistantDataPresent) {
  throw "Managed configuration restore receipt is invalid: $($result | ConvertTo-Json -Depth 8 -Compress)"
}
if ($configAfter -notmatch [regex]::Escape($profileMarker) -or
    $configAfter -match "codex_assistant_router" -or
    $configAfter -match "model_catalog_json") {
  throw "Managed configuration cleanup did not preserve unrelated user TOML"
}
if ($result.deleteData.changed -ne "true" -or
    $result.deleteData.beforeAssistantData -ne "true" -or
    $result.deleteData.afterAssistantData -ne "false" -or
    $result.final.assistantDataPresent -or
    $result.final.managedConfigPresent -or
    !$result.final.officialAppInstalled -or
    !$result.final.officialAppTrusted -or
    !$result.oldFactoryResetAbsent -or
    (Test-Path $dataRoot)) {
  throw "Assistant data deletion crossed a lifecycle boundary: $($result | ConvertTo-Json -Depth 8 -Compress)"
}

$uninstallEvidence = $null
if ($TestDefaultUninstall) {
  if (!$InstallerPath -or !(Test-Path $InstallerPath)) {
    throw "-InstallerPath must point to the exact candidate when -TestDefaultUninstall is used"
  }
  if (!$ExpectedSha256) {
    throw "-ExpectedSha256 is required when -TestDefaultUninstall is used"
  }
  $actualSha256 = (Get-FileHash $InstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actualSha256 -ne $ExpectedSha256.ToLowerInvariant()) {
    throw "Installer SHA256 mismatch: $actualSha256"
  }

  New-Item -ItemType Directory -Force $dataRoot | Out-Null
  $dataSentinel = Join-Path $dataRoot "preserve-on-uninstall.txt"
  Set-Content -Path $dataSentinel -Encoding UTF8 -Value "preserve"
  $registration = Get-AssistantRegistration
  if (!$registration) { throw "Codex Assistant uninstall registration was not found" }
  $installRoot = ([string]$registration.InstallLocation).Trim('"')
  $uninstaller = ([string]$registration.UninstallString).Trim('"')
  if (!(Test-Path $uninstaller)) { throw "Registered uninstaller was not found: $uninstaller" }
  Get-AssistantUninstallProcesses $installRoot $uninstaller |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Milliseconds 500
  $chatGptBefore = Get-AppxPackage -Name "OpenAI.Codex" -ErrorAction SilentlyContinue
  if (!$chatGptBefore) { throw "Trusted ChatGPT package was missing before assistant uninstall" }

  $handoffPort = $DebugPort + 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$handoffPort"
  Start-Process -FilePath $assistantExe
  $handoffTargets = Wait-Until {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:$handoffPort/json/list" -TimeoutSec 2 } catch { $null }
  } 20 "Codex Assistant handoff debug endpoint did not start"
  $handoffTarget = $handoffTargets |
    Where-Object { $_.type -eq "page" -and $_.url -eq "http://tauri.localhost/" } |
    Select-Object -First 1
  if (!$handoffTarget) { throw "Codex Assistant handoff page target was not found" }
  $handoffSocket = [Net.WebSockets.ClientWebSocket]::new()
  [void]$handoffSocket.ConnectAsync(
    [Uri]$handoffTarget.webSocketDebuggerUrl,
    [Threading.CancellationToken]::None
  ).GetAwaiter().GetResult()
  try {
    $handoff = Invoke-CdpExpression $handoffSocket @'
(async () => {
  const readyDeadline = Date.now() + 20000;
  const refresh = document.querySelector('#refreshButton');
  while (refresh.disabled && Date.now() < readyDeadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  document.querySelector('[data-view="diagnostics"]').click();
  const button = document.querySelector('#uninstallAssistantButton');
  const buttonDeadline = Date.now() + 10000;
  while (button.disabled && Date.now() < buttonDeadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (button.disabled) throw new Error('Assistant uninstall button is disabled');
  button.click();
  const overlay = document.querySelector('#confirmOverlay');
  const confirmDeadline = Date.now() + 5000;
  while (overlay.classList.contains('hidden') && Date.now() < confirmDeadline) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (overlay.classList.contains('hidden')) throw new Error('Assistant uninstall confirmation did not open');
  document.querySelector('#confirmAcceptButton').click();
  const receipt = document.querySelector('#lifecycleResult');
  const receiptDeadline = Date.now() + 10000;
  while (
    (receipt.dataset.actionId !== 'uninstall_assistant' || receipt.classList.contains('hidden')) &&
    Date.now() < receiptDeadline
  ) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  return {
    actionId: receipt.dataset.actionId || '',
    status: receipt.dataset.status || '',
    changed: receipt.dataset.changed || '',
    text: receipt.textContent.trim()
  };
})()
'@ 2
  } finally {
    $handoffSocket.Dispose()
  }
  if ($handoff.actionId -ne "uninstall_assistant" -or
      $handoff.status -ne "handoff_started" -or
      $handoff.changed -ne "false") {
    throw "Assistant uninstall handoff receipt is invalid: $($handoff | ConvertTo-Json -Compress)"
  }
  Wait-Until {
    !(Get-Process codex-assistant -ErrorAction SilentlyContinue)
  } 10 "Assistant did not exit after uninstall handoff" | Out-Null
  $uninstallProcesses = Wait-Until {
    $processes = @(Get-AssistantUninstallProcesses $installRoot $uninstaller)
    if ($processes.Count) { $processes } else { $null }
  } 10 "NSIS uninstaller did not start from the assistant handoff"
  @($uninstallProcesses) |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Milliseconds 500
  if (!(Get-AssistantRegistration) -or !(Test-Path $assistantExe)) {
    throw "Cancelling the interactive handoff unexpectedly removed the assistant"
  }

  $uninstallProcess = Start-Process -FilePath $uninstaller -ArgumentList "/S" -PassThru -Wait
  if ($uninstallProcess.ExitCode -ne 0) {
    throw "Assistant uninstaller failed with exit code $($uninstallProcess.ExitCode)"
  }
  Wait-Until { !(Get-AssistantRegistration) } 30 "Assistant uninstall registration was not removed" | Out-Null
  $chatGptAfter = Get-AppxPackage -Name "OpenAI.Codex" -ErrorAction SilentlyContinue
  if (!(Test-Path $configPath) -or
      (Get-Content $configPath -Raw) -notmatch [regex]::Escape($profileMarker) -or
      !(Test-Path $dataSentinel) -or
      !$chatGptAfter -or
      $chatGptAfter.PackageFamilyName -ne $chatGptBefore.PackageFamilyName) {
    throw "Default assistant uninstall removed data, config, or the official ChatGPT package"
  }

  $installProcess = Start-Process -FilePath $InstallerPath -ArgumentList "/S" -PassThru -Wait
  if ($installProcess.ExitCode -ne 0) {
    throw "Candidate reinstall failed with exit code $($installProcess.ExitCode)"
  }
  Wait-Until { Get-AssistantRegistration } 30 "Assistant registration was not restored after reinstall" | Out-Null
  $uninstallEvidence = [ordered]@{
    exactCandidateSha256 = $actualSha256
    assistantRemoved = $true
    configPreserved = $true
    dataPreserved = $true
    officialAppPreserved = $true
    interactiveHandoffVerified = $true
    candidateReinstalled = $true
  }
}

[ordered]@{
  success = $true
  blockedDeleteCode = $result.blockedDeleteCode
  configRestoreChanged = $true
  unrelatedConfigPreserved = $true
  assistantDataDeleted = $true
  officialAppPreserved = $true
  oldFactoryResetAbsent = $true
  defaultUninstall = $uninstallEvidence
} | ConvertTo-Json -Depth 6 -Compress
