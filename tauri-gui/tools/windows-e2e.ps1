param(
  [string]$RouterUrl = "http://10.211.55.2:11435/v1",
  [string]$RouterKey = "",
  [int]$DebugPort = 9223,
  [switch]$LaunchAfterSetup,
  [switch]$TestRestore,
  [switch]$ExpectSetupFailure,
  [switch]$ExpectRollback,
  [ValidateSet("none", "focus", "custom", "official")]
  [string]$ApplyAppearance = "none",
  [string]$ThemeImagePath = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
if ($ExpectSetupFailure -and $ExpectRollback) {
  throw "-ExpectSetupFailure and -ExpectRollback cannot be used together"
}

function Wait-Until([scriptblock]$Condition, [int]$TimeoutSeconds, [string]$FailureMessage) {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  do {
    $value = & $Condition
    if ($value) { return $value }
    Start-Sleep -Milliseconds 250
  } until ((Get-Date) -gt $deadline)
  throw $FailureMessage
}

function Get-ChatGptProcessCount {
  $count = 0
  Get-Process -ErrorAction SilentlyContinue | ForEach-Object {
    try {
      if ($_.Path -like "*\WindowsApps\OpenAI.Codex_*\*") { $count++ }
    } catch {}
  }
  return $count
}

function Get-FileFingerprint([string]$Path) {
  if (!(Test-Path $Path)) {
    return [ordered]@{ exists = $false; sha256 = "" }
  }
  return [ordered]@{
    exists = $true
    sha256 = (Get-FileHash $Path -Algorithm SHA256).Hash
  }
}

function Get-BytesSha256([byte[]]$Bytes) {
  $sha256 = [Security.Cryptography.SHA256]::Create()
  try {
    return ([BitConverter]::ToString($sha256.ComputeHash($Bytes))).Replace("-", "").ToLowerInvariant()
  } finally {
    $sha256.Dispose()
  }
}

function Assert-FileFingerprint([string]$Path, $Before, [string]$Label) {
  $after = Get-FileFingerprint $Path
  if ($after.exists -ne $Before.exists -or $after.sha256 -ne $Before.sha256) {
    throw "$Label changed during a transaction that should have rolled back"
  }
}

function Assert-TransactionManifest($Summary, [string]$ExpectedStatus, [string]$ExpectedOperation) {
  if (!$Summary.manifestPath -or !(Test-Path $Summary.manifestPath)) {
    throw "Transaction manifest was not preserved"
  }
  $manifest = Get-Content $Summary.manifestPath -Raw | ConvertFrom-Json
  if ($manifest.schemaVersion -ne 2 -or
      $manifest.transactionId -ne $Summary.transactionId -or
      $manifest.status -ne $ExpectedStatus -or
      $manifest.operation -ne $ExpectedOperation -or
      !$manifest.createdAt -or
      !$manifest.createdOrder -or
      !$manifest.appVersion) {
    throw "Transaction manifest metadata is incomplete or inconsistent"
  }
  $expectedIds = @("codex-config", "model-catalog", "router-secret", "runtime-state")
  $actualIds = @($manifest.files | ForEach-Object { $_.id } | Sort-Object)
  if ($actualIds.Count -ne $expectedIds.Count -or
      (Compare-Object $expectedIds $actualIds)) {
    throw "Transaction manifest does not cover all managed configuration files"
  }
  $snapshotRoot = Split-Path $Summary.manifestPath -Parent
  foreach ($file in $manifest.files) {
    if (!$file.targetPath) {
      throw "Transaction manifest contains an empty target path"
    }
    if ($file.existed) {
      if (!$file.backupFile -or !$file.sha256) {
        throw "Transaction manifest is missing backup evidence for $($file.id)"
      }
      $backupPath = Join-Path $snapshotRoot $file.backupFile
      if (!(Test-Path $backupPath)) {
        throw "Transaction backup is missing for $($file.id)"
      }
      $actualHash = (Get-FileHash $backupPath -Algorithm SHA256).Hash.ToLowerInvariant()
      if ($actualHash -ne $file.sha256.ToLowerInvariant()) {
        throw "Transaction backup SHA256 mismatch for $($file.id)"
      }
    } elseif ($file.backupFile -or $file.sha256) {
      throw "Transaction manifest recorded backup evidence for a file that did not exist"
    }
  }
  return $manifest
}

function Invoke-CdpExpression([System.Net.WebSockets.ClientWebSocket]$Socket, [string]$Expression, [int]$Id) {
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
  $segment = [ArraySegment[byte]]::new($bytes)
  [void]$Socket.SendAsync($segment, [Net.WebSockets.WebSocketMessageType]::Text, $true, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

  while ($true) {
    $buffer = New-Object byte[] 131072
    $received = New-Object Text.StringBuilder
    do {
      $part = $Socket.ReceiveAsync([ArraySegment[byte]]::new($buffer), [Threading.CancellationToken]::None).GetAwaiter().GetResult()
      if ($part.MessageType -eq [Net.WebSockets.WebSocketMessageType]::Close) {
        throw "CDP socket closed before command $Id completed"
      }
      [void]$received.Append([Text.Encoding]::UTF8.GetString($buffer, 0, $part.Count))
    } until ($part.EndOfMessage)

    $response = $received.ToString() | ConvertFrom-Json
    if ($response.id -eq $Id) {
      if ($response.error) { throw "CDP command failed: $($response.error.message)" }
      if ($response.result.exceptionDetails) {
        $description = $response.result.exceptionDetails.exception.description
        if (!$description) { $description = $response.result.exceptionDetails.text }
        throw "Browser expression failed: $description"
      }
      return $response.result.result.value
    }
  }
}

$assistantExe = Get-ChildItem $env:LOCALAPPDATA -Filter "codex-assistant.exe" -Recurse -Depth 4 |
  Select-Object -First 1 -ExpandProperty FullName
if (!$assistantExe) { throw "Installed codex-assistant.exe was not found" }
$configPath = Join-Path $env:USERPROFILE ".codex\config.toml"
$runtimeConfigPath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\config.json"
$modelCatalogPath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\models.json"
$routerSecretPath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\router-key.secret"
$lastTransactionPath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\transaction.last.json"
$activeTransactionPath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\transaction.active.json"
$hadConfigBefore = Test-Path $configPath
$stateBeforeRestore = if ($TestRestore -and (Test-Path $runtimeConfigPath)) {
  Get-Content $runtimeConfigPath -Raw | ConvertFrom-Json
} else {
  $null
}
$tokenBeforeRestore = if ($stateBeforeRestore -and $stateBeforeRestore.tokenMode -eq "static") {
  (& $assistantExe --codex-assistant-token-helper $runtimeConfigPath | Out-String).Trim()
} else {
  ""
}

Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -ErrorAction SilentlyContinue | ForEach-Object {
  try {
    if ($_.Path -like "*\WindowsApps\OpenAI.Codex_*\*") { Stop-Process -Id $_.Id -Force }
  } catch {}
}
[void](Wait-Until { (Get-ChatGptProcessCount) -eq 0 } 15 "ChatGPT processes did not stop before setup")
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"
Start-Process -FilePath $assistantExe

$targets = Wait-Until {
  try { Invoke-RestMethod -Uri "http://127.0.0.1:$DebugPort/json/list" -TimeoutSec 2 } catch { $null }
} 20 "Codex Assistant WebView2 debug endpoint did not start"
$target = $targets | Where-Object { $_.type -eq "page" -and $_.url -eq "http://tauri.localhost/" } | Select-Object -First 1
if (!$target) { throw "Codex Assistant page target was not found" }

$socket = [Net.WebSockets.ClientWebSocket]::new()
[void]$socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

$routerJson = $RouterUrl | ConvertTo-Json -Compress
$keyJson = $RouterKey | ConvertTo-Json -Compress
$hasKeyJson = if ($RouterKey) { "true" } else { "false" }
$connection = Invoke-CdpExpression $socket @"
(async () => {
  const readyDeadline = Date.now() + 20000;
  const refresh = document.querySelector('#refreshButton');
  const overviewAction = document.querySelector('#overviewAction');
  while ((refresh.disabled || overviewAction.disabled) && Date.now() < readyDeadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (refresh.disabled || overviewAction.disabled) throw new Error('Initial status check did not finish');
  const status = await window.__TAURI__.core.invoke('get_system_status');
  if (status.app?.state !== 'installed' ||
      status.app?.trusted !== true ||
      status.app?.source !== 'microsoft-store' ||
      !status.app?.version) {
    throw new Error('Official ChatGPT package did not produce trusted Windows evidence: ' +
      JSON.stringify(status.app));
  }
  const statusEvidence = {
    app: document.querySelector('#appStatusBadge').classList.contains('success'),
    router: document.querySelector('#routerStatusBadge').classList.contains('success'),
    config: document.querySelector('#configStatusBadge').classList.contains('success')
  };
  if (statusEvidence.app !== status.appInstalled ||
      statusEvidence.router !== status.routerReachable ||
      statusEvidence.config !== status.configPresent) {
    throw new Error('Overview status cards disagree with SystemStatusV1: ' + JSON.stringify({
      status: {
        appInstalled: status.appInstalled,
        routerReachable: status.routerReachable,
        configPresent: status.configPresent
      },
      rendered: statusEvidence
    }));
  }
  document.querySelector('[data-view="diagnostics"]').click();
  const diagnosticEvidence = {
    app: document.querySelector('#diagApp').textContent.trim(),
    router: document.querySelector('#diagRouter').textContent.trim(),
    config: document.querySelector('#diagConfig').textContent.trim()
  };
  if ((status.appDetail && !diagnosticEvidence.app.includes(status.appDetail)) ||
      (status.routerDetail && !diagnosticEvidence.router.includes(status.routerDetail)) ||
      (status.configPresent && status.configuredModel &&
       !diagnosticEvidence.config.includes(status.configuredModel))) {
    throw new Error('Diagnostics status disagrees with SystemStatusV1');
  }
  document.querySelector('[data-view="setup"]').click();
  const guidedSteps = [...document.querySelectorAll('[data-guided-step]')].map(step => ({
    id: step.dataset.guidedStep,
    label: step.textContent.trim()
  }));
  if (guidedSteps.map(step => step.id).join(',') !== 'environment,app,service,verify') {
    throw new Error('Guided setup does not expose the required four user steps: ' + JSON.stringify(guidedSteps));
  }
  const setupEvidence = {
    environment: document.querySelector('#setupEnvironmentDetail').textContent.trim(),
    app: document.querySelector('#setupAppDetail').textContent.trim(),
    environmentReady: document.querySelector('#setupEnvironmentState').classList.contains('success'),
    appReady: document.querySelector('#setupAppState').classList.contains('success')
  };
  if (!setupEvidence.environmentReady ||
      setupEvidence.appReady !== status.appInstalled ||
      !setupEvidence.environment.includes(status.platform)) {
    throw new Error('Guided setup prerequisites disagree with SystemStatusV1: ' + JSON.stringify(setupEvidence));
  }
  const expectedGateway = status.configuredGateway || 'http://127.0.0.1:11434/v1';
  if (document.querySelector('#gatewayInput').value !== expectedGateway) {
    throw new Error('Setup form gateway disagrees with SystemStatusV1');
  }
  let localOllamaDiagnostic = '';
  if (status.platform === 'Windows' && ['aarch64', 'arm64'].includes(status.architecture)) {
    document.querySelector('[data-preset="ollama"]').click();
    const localButton = document.querySelector('#testRouterButton');
    localButton.click();
    const localDeadline = Date.now() + 15000;
    while (localButton.disabled && Date.now() < localDeadline) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    const localResult = document.querySelector('#connectionResult');
    localOllamaDiagnostic = localResult.textContent.trim();
    if (!localResult.classList.contains('error') || !localOllamaDiagnostic.includes('Windows ARM64')) {
      throw new Error('Local Ollama diagnostic is not actionable: ' + localOllamaDiagnostic);
    }
  }
  const input = document.querySelector('#gatewayInput');
  input.value = $routerJson;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  const noAuth = document.querySelector('#noAuthInput');
  noAuth.checked = !$hasKeyJson;
  noAuth.dispatchEvent(new Event('change', { bubbles: true }));
  const key = document.querySelector('#keyInput');
  key.value = $keyJson;
  key.dispatchEvent(new Event('input', { bubbles: true }));
  const button = document.querySelector('#testRouterButton');
  button.click();
  const deadline = Date.now() + 20000;
  while (button.disabled && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 100));
  const result = document.querySelector('#connectionResult');
  return {
    className: result.className,
    message: result.textContent.trim(),
    gateway: input.value,
    statusConsistency: true,
    guidedSteps,
    setupEvidence,
    localOllamaDiagnostic,
    models: [...document.querySelector('#modelInput').options].map(option => option.value).filter(Boolean),
    selectedModel: document.querySelector('#modelInput').value
  };
})()
"@ 1

if ($connection.className -notmatch "success") { throw "Router UI test failed: $($connection.message)" }
if ($connection.gateway -ne $RouterUrl) { throw "Router input changed unexpectedly: $($connection.gateway)" }
if (!$connection.selectedModel) { throw "Router returned no selectable model" }

$configExistedBeforeSetup = Test-Path $configPath
$configHashBeforeSetup = if ($configExistedBeforeSetup) {
  (Get-FileHash $configPath -Algorithm SHA256).Hash
} else {
  ""
}
$runtimeBeforeSetup = if (Test-Path $runtimeConfigPath) {
  Get-Content $runtimeConfigPath -Raw | ConvertFrom-Json
} else {
  $null
}
if (($ExpectSetupFailure -or $ExpectRollback) -and
    (!$configExistedBeforeSetup -or
     !$runtimeBeforeSetup -or
     !$runtimeBeforeSetup.responsesVerifiedAt)) {
  throw "Expected-failure setup requires an existing Responses-verified configuration"
}
$managedBeforeSetup = [ordered]@{
  config = Get-FileFingerprint $configPath
  runtime = Get-FileFingerprint $runtimeConfigPath
  models = Get-FileFingerprint $modelCatalogPath
  secret = Get-FileFingerprint $routerSecretPath
}

$chatGptBefore = Get-ChatGptProcessCount
$result = Invoke-CdpExpression $socket @'
(async () => {
  document.querySelector('#routerForm').requestSubmit();
  const deadline = Date.now() + 900000;
  const panel = document.querySelector('#resultPanel');
  while (panel.classList.contains('hidden') && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 150));
  return {
    visible: !panel.classList.contains('hidden'),
    success: document.querySelector('#resultMark').classList.contains('success'),
    title: document.querySelector('#resultTitle').textContent.trim(),
    message: document.querySelector('#resultText').textContent.trim(),
    summary: document.querySelector('#resultSummary').textContent.trim(),
    failedTaskId: document.querySelector('.task-item.failed')?.dataset.taskId || '',
    failedStep: document.querySelector('.task-item.failed .task-copy strong')?.textContent.trim() || '',
    failedMessage: document.querySelector('.task-item.failed .task-copy small')?.textContent.trim() || '',
    recoveryVisible: !document.querySelector('#resultRecovery').classList.contains('hidden'),
    recoveryState: document.querySelector('#resultRecovery').dataset.recoveryState || 'none',
    recovery: document.querySelector('#resultRecoveryText').textContent.trim(),
    summaryKeys: [...document.querySelectorAll('#resultSummary [data-summary-key]')].map(row => row.dataset.summaryKey),
    diagnosticActionVisible: !document.querySelector('#resultDiagnosticButton').classList.contains('hidden'),
    retryIsPrimary: document.querySelector('#resultBackButton').classList.contains('primary-button'),
    installTaskState: document.querySelector('[data-task-id="install_chatgpt"]')?.classList.contains('skipped')
      ? 'skipped'
      : 'unexpected',
    completedGuidedSteps: document.querySelectorAll('[data-guided-step].complete').length,
    logs: document.querySelector('#logOutput').textContent
  };
})()
'@ 2
$chatGptAfter = Get-ChatGptProcessCount

if (!$result.visible) { throw "Configuration result did not become visible" }
if (!$result.diagnosticActionVisible) { throw "Result page does not expose the persistent diagnostic action" }
if ($result.installTaskState -ne "skipped") {
  throw "Healthy official ChatGPT package did not skip the install stage"
}
if ($chatGptBefore -ne 0 -or $chatGptAfter -ne 0) {
  throw "ChatGPT was running during configuration (before=$chatGptBefore, after=$chatGptAfter)"
}

if ($ExpectRollback) {
  if ($result.success) { throw "Configuration unexpectedly succeeded" }
  if ($result.failedTaskId -ne "verify") {
    throw "Configuration failed at an unexpected step: $($result.failedTaskId)"
  }
  if (!$result.recoveryVisible -or $result.recoveryState -ne "restored") {
    throw "Rollback result is not clearly presented to the user: $($result.recovery)"
  }
  if (!$result.retryIsPrimary) {
    throw "Failure result does not expose one primary retry action"
  }
  Assert-FileFingerprint $configPath $managedBeforeSetup.config "Codex config"
  Assert-FileFingerprint $runtimeConfigPath $managedBeforeSetup.runtime "Assistant runtime state"
  Assert-FileFingerprint $modelCatalogPath $managedBeforeSetup.models "Model catalog"
  Assert-FileFingerprint $routerSecretPath $managedBeforeSetup.secret "Router secret"
  if (!(Test-Path $lastTransactionPath)) {
    throw "Rolled-back transaction summary was not written"
  }
  $lastTransaction = Get-Content $lastTransactionPath -Raw | ConvertFrom-Json
  if ($lastTransaction.status -ne "rolled_back" -or !$lastTransaction.transactionId) {
    throw "Last transaction does not record a successful rollback"
  }
  $rollbackManifest = Assert-TransactionManifest $lastTransaction "rolled_back" "configure"
  if (Test-Path $activeTransactionPath) {
    throw "Active transaction journal remained after a successful rollback"
  }
  $statusAfterRollback = Invoke-CdpExpression $socket @'
(async () => {
  const status = await window.__TAURI__.core.invoke('get_system_status');
  return {
    overall: status.overall,
    configState: status.config?.state,
    lastTransactionId: status.config?.lastTransactionId,
    configPresent: status.configPresent,
    ready: status.ready
  };
})()
'@ 18
  if ($statusAfterRollback.configState -ne "verified" -or
      !$statusAfterRollback.configPresent -or
      $statusAfterRollback.lastTransactionId -ne $lastTransaction.transactionId) {
    throw "SystemStatusV1 does not expose the rolled-back transaction: $($statusAfterRollback | ConvertTo-Json -Compress)"
  }
  $socket.Dispose()
  [pscustomobject]@{
    success = $true
    expectedRollback = $true
    router = $RouterUrl
    selectedModel = $connection.selectedModel
    failedTaskId = $result.failedTaskId
    filesRestored = $true
    transactionId = $lastTransaction.transactionId
    transactionStatus = $lastTransaction.status
    activeJournalRemoved = $true
    systemStatus = $statusAfterRollback
    chatGptProcessCountDuringSetup = $chatGptAfter
  } | ConvertTo-Json -Depth 5 -Compress
  return
}

if ($ExpectSetupFailure) {
  if ($result.success) { throw "Configuration unexpectedly succeeded" }
  if ($result.failedTaskId -ne "validate_router_response") {
    throw "Configuration failed at an unexpected step: $($result.failedTaskId)"
  }
  if (!$result.retryIsPrimary) {
    throw "Failure result does not expose one primary retry action"
  }
  if (!(Test-Path $configPath)) { throw "Existing Codex config was removed after a failed probe" }
  $configHashAfterSetup = (Get-FileHash $configPath -Algorithm SHA256).Hash
  if ($configHashAfterSetup -ne $configHashBeforeSetup) {
    throw "Codex config changed after a failed Responses probe"
  }
  if (!(Test-Path $runtimeConfigPath)) { throw "Assistant runtime state was removed after a failed probe" }
  $runtimeAfterFailure = Get-Content $runtimeConfigPath -Raw | ConvertFrom-Json
  if ($runtimeAfterFailure.gatewayBaseUrl -ne $runtimeBeforeSetup.gatewayBaseUrl -or
      $runtimeAfterFailure.model -ne $runtimeBeforeSetup.model -or
      $runtimeAfterFailure.tokenMode -ne $runtimeBeforeSetup.tokenMode) {
    throw "Router identity or token mode changed after a failed Responses probe"
  }
  if ($runtimeAfterFailure.responsesVerifiedAt -or $runtimeAfterFailure.responsesProtocol) {
    throw "Failed Responses probe left stale verification evidence in runtime state"
  }
  $statusAfterFailure = Invoke-CdpExpression $socket @'
(async () => {
  const status = await window.__TAURI__.core.invoke('get_system_status');
  return {
    overall: status.overall,
    routerState: status.router?.state,
    lastVerifiedAt: status.router?.lastVerifiedAt,
    ready: status.ready
  };
})()
'@ 19
  if ($statusAfterFailure.routerState -ne "models_verified" -or
      $statusAfterFailure.lastVerifiedAt -or
      $statusAfterFailure.ready) {
    throw "SystemStatusV1 retained stale readiness after a failed Responses probe: $($statusAfterFailure | ConvertTo-Json -Compress)"
  }
  $socket.Dispose()
  [pscustomobject]@{
    success = $true
    expectedFailure = $true
    router = $RouterUrl
    selectedModel = $connection.selectedModel
    failedTaskId = $result.failedTaskId
    failedStep = $result.failedStep
    failedMessage = $result.failedMessage
    configUnchanged = $true
    evidenceInvalidated = $true
    systemStatus = $statusAfterFailure
    chatGptProcessCountDuringSetup = $chatGptAfter
  } | ConvertTo-Json -Depth 5 -Compress
  return
}

if (!$result.success) {
  throw "Configuration UI did not complete: $($result.title) $($result.message)"
}
if ($result.completedGuidedSteps -ne 4) {
  throw "Successful setup did not complete all four guided steps"
}
if ($result.summaryKeys -notcontains "last-verified" -or $result.summaryKeys -notcontains "recovery") {
  throw "Successful setup summary is missing validation or recovery evidence: $($result.summary)"
}

$backupUiVisible = Invoke-CdpExpression $socket @'
(async () => {
  const refresh = document.querySelector('#refreshButton');
  const deadline = Date.now() + 10000;
  while (refresh.disabled && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 100));
  return {
    home: !document.querySelector('#restoreConfigButton').classList.contains('hidden'),
    diagnostics: !document.querySelector('#diagnosticRestoreButton').disabled
  };
})()
'@ 6
if ($hadConfigBefore -and (!$backupUiVisible.home -or !$backupUiVisible.diagnostics)) {
  throw "Configuration backup was written but the restore actions are not available in the UI"
}

if (!(Test-Path $configPath)) { throw "Codex config was not written" }
if (!(Test-Path $runtimeConfigPath)) { throw "Assistant runtime config was not written" }
$configText = Get-Content $configPath -Raw
if ($configText -notmatch [regex]::Escape($RouterUrl)) { throw "Codex config does not contain the tested Router URL" }
if ($configText -notmatch [regex]::Escape($connection.selectedModel)) { throw "Codex config does not contain the selected model" }
$runtimeState = Get-Content $runtimeConfigPath -Raw | ConvertFrom-Json
if (!$runtimeState.responsesVerifiedAt -or $runtimeState.responsesProtocol -notin @("sse", "json")) {
  throw "Runtime state does not contain valid Responses verification evidence"
}
if (!$runtimeState.transactionId) {
  throw "Runtime state does not contain the committed transaction ID"
}
if (!(Test-Path $lastTransactionPath)) {
  throw "Committed transaction summary was not written"
}
$committedTransaction = Get-Content $lastTransactionPath -Raw | ConvertFrom-Json
if ($committedTransaction.status -ne "committed" -or
    $committedTransaction.transactionId -ne $runtimeState.transactionId) {
  throw "Committed transaction summary does not match runtime state"
}
$committedManifest = Assert-TransactionManifest $committedTransaction "committed" "configure"
if (Test-Path $activeTransactionPath) {
  throw "Active transaction journal remained after a successful setup"
}
$statusAfterSetup = Invoke-CdpExpression $socket @'
(async () => {
  const status = await window.__TAURI__.core.invoke('get_system_status');
  return {
    overall: status.overall,
    routerState: status.router?.state,
    lastVerifiedAt: status.router?.lastVerifiedAt,
    configState: status.config?.state,
    lastTransactionId: status.config?.lastTransactionId,
    ready: status.ready
  };
})()
'@ 20
if ($statusAfterSetup.overall -ne "ready" -or
    $statusAfterSetup.routerState -ne "responses_verified" -or
    !$statusAfterSetup.lastVerifiedAt -or
    $statusAfterSetup.configState -ne "verified" -or
    $statusAfterSetup.lastTransactionId -ne $runtimeState.transactionId -or
    !$statusAfterSetup.ready) {
  throw "SystemStatusV1 did not become Responses-verified after setup: $($statusAfterSetup | ConvertTo-Json -Compress)"
}

$diagnosticBundle = Invoke-CdpExpression $socket @'
(async () => {
  return window.__TAURI__.core.invoke('export_diagnostics', {
    request: {
      supportId: '',
      errorCode: '',
      errorStage: '',
      suggestedAction: ''
    }
  });
})()
'@ 21
if ($diagnosticBundle.fileName -notmatch '^diagnostics-CA-[A-Z0-9-]+\.zip$' -or
    !$diagnosticBundle.supportId -or
    !$diagnosticBundle.contentBase64 -or
    !$diagnosticBundle.savedPath -or
    !(Test-Path $diagnosticBundle.savedPath) -or
    $diagnosticBundle.sha256 -notmatch '^[a-f0-9]{64}$') {
  throw "Diagnostic bundle receipt is incomplete"
}
$diagnosticBytes = [Convert]::FromBase64String($diagnosticBundle.contentBase64)
if ($diagnosticBytes.Length -ne $diagnosticBundle.byteLength -or
    (Get-BytesSha256 $diagnosticBytes) -ne $diagnosticBundle.sha256 -or
    (Get-FileHash $diagnosticBundle.savedPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne
      $diagnosticBundle.sha256) {
  throw "Diagnostic bundle receipt hash or byte length is invalid"
}

Add-Type -AssemblyName System.IO.Compression
$diagnosticStream = [IO.MemoryStream]::new($diagnosticBytes)
$diagnosticArchive = [IO.Compression.ZipArchive]::new(
  $diagnosticStream,
  [IO.Compression.ZipArchiveMode]::Read,
  $false
)
try {
  $expectedDiagnosticEntries = @("checksums.txt", "manifest.json", "recent.log", "status.json")
  $actualDiagnosticEntries = @($diagnosticArchive.Entries | ForEach-Object { $_.FullName } | Sort-Object)
  if ($actualDiagnosticEntries.Count -ne $expectedDiagnosticEntries.Count -or
      (Compare-Object $expectedDiagnosticEntries $actualDiagnosticEntries)) {
    throw "Diagnostic bundle does not contain the required four files"
  }

  $diagnosticTexts = @{}
  foreach ($entryName in $expectedDiagnosticEntries) {
    $entry = $diagnosticArchive.GetEntry($entryName)
    $reader = [IO.StreamReader]::new($entry.Open(), [Text.Encoding]::UTF8)
    try {
      $diagnosticTexts[$entryName] = $reader.ReadToEnd()
    } finally {
      $reader.Dispose()
    }
  }
  $combinedDiagnosticText = ($diagnosticTexts.Values -join "`n")
  if (($RouterKey -and $combinedDiagnosticText.Contains($RouterKey)) -or
      $combinedDiagnosticText -match '(?i)Bearer\s+(?!\[redacted\])\S+' -or
      $combinedDiagnosticText -match '(?i)[A-Z]:\\Users\\[^\\\s]+' -or
      $combinedDiagnosticText -match '(?i)\b(sk-|ghp_|xoxb-)[A-Za-z0-9_-]{8,}') {
    throw "Diagnostic bundle contains unredacted sensitive data"
  }

  $checksums = $diagnosticTexts["checksums.txt"]
  foreach ($entryName in @("manifest.json", "status.json", "recent.log")) {
    $entryHash = Get-BytesSha256 ([Text.Encoding]::UTF8.GetBytes($diagnosticTexts[$entryName]))
    if (!$checksums.Contains("$entryHash  $entryName")) {
      throw "Diagnostic bundle checksum mismatch for $entryName"
    }
  }
} finally {
  $diagnosticArchive.Dispose()
  $diagnosticStream.Dispose()
}

$downloadsDirectory = Split-Path $diagnosticBundle.savedPath -Parent
$existingDiagnosticDownloads = @{}
Get-ChildItem $downloadsDirectory -Filter "diagnostics-CA-*.zip" -ErrorAction SilentlyContinue |
  ForEach-Object {
    $existingDiagnosticDownloads[$_.FullName] =
      (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  }
$diagnosticDownloadUi = Invoke-CdpExpression $socket @'
(async () => {
  document.querySelector('[data-view="diagnostics"]').click();
  const button = document.querySelector('#diagnosticExportButton');
  button.click();
  const deadline = Date.now() + 30000;
  while (button.disabled && Date.now() < deadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  return {
    completed: !button.disabled,
    toast: document.querySelector('#actionToast').textContent.trim()
  };
})()
'@ 22
if (!$diagnosticDownloadUi.completed -or
    $diagnosticDownloadUi.toast -notmatch 'CA-[A-Z0-9-]+') {
  throw "Diagnostic export button did not complete with a support ID"
}
$downloadedDiagnostic = Wait-Until {
  Get-ChildItem $downloadsDirectory -Filter "diagnostics-CA-*.zip" -ErrorAction SilentlyContinue |
    Where-Object {
      !$existingDiagnosticDownloads.ContainsKey($_.FullName) -or
      $existingDiagnosticDownloads[$_.FullName] -ne
        (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
} 15 "Diagnostic export button did not create a ZIP in Downloads"
if ($downloadedDiagnostic.Length -le 0) {
  throw "Diagnostic export button created an empty ZIP"
}
$diagnosticUiDownloadSha256 = (Get-FileHash $downloadedDiagnostic.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
Remove-Item $downloadedDiagnostic.FullName -Force
Remove-Item $diagnosticBundle.savedPath -Force

$keyConfigured = [bool]$RouterKey
if ($keyConfigured) {
  if ($runtimeState.tokenMode -ne "static" -or $runtimeState.secretStorage -ne "dpapi") {
    throw "Router key was not stored with Windows DPAPI"
  }
  if (!(Test-Path $runtimeState.keyPath)) { throw "Protected Router key file was not written" }
  $resolvedKey = (& $assistantExe --codex-assistant-token-helper $runtimeConfigPath | Out-String).Trim()
  if ($resolvedKey -ne $RouterKey) { throw "Token helper did not return the original Router key" }
  $resolvedKey = $null
} elseif ($runtimeState.tokenMode -ne "none") {
  throw "No-key setup produced an unexpected token mode"
}

$restoreRoundTrip = $false
if ($TestRestore) {
  if (!$stateBeforeRestore) { throw "Restore test requires an existing assistant state" }
  $restoreResult = Invoke-CdpExpression $socket @'
(async () => {
  const button = document.querySelector('#restoreConfigButton');
  if (button.classList.contains('hidden')) throw new Error('Restore action is hidden');
  button.click();
  const confirmDeadline = Date.now() + 5000;
  while (document.querySelector('#confirmOverlay').classList.contains('hidden') && Date.now() < confirmDeadline) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (document.querySelector('#confirmOverlay').classList.contains('hidden')) {
    throw new Error('Restore confirmation did not open');
  }
  document.querySelector('#confirmAcceptButton').click();
  let observedBusy = false;
  const deadline = Date.now() + 60000;
  while (Date.now() < deadline) {
    if (button.disabled) observedBusy = true;
    if (observedBusy && !button.disabled) return { completed: true };
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  return { completed: false };
})()
'@ 7
  if (!$restoreResult.completed) { throw "Restore UI did not complete" }
  [void](Wait-Until { (Get-ChatGptProcessCount) -gt 0 } 45 "ChatGPT did not restart after restore")
  $restoredState = Get-Content $runtimeConfigPath -Raw | ConvertFrom-Json
  if ($restoredState.gatewayBaseUrl -ne $stateBeforeRestore.gatewayBaseUrl -or
      $restoredState.model -ne $stateBeforeRestore.model -or
      $restoredState.tokenMode -ne $stateBeforeRestore.tokenMode) {
    throw "Restore did not recover the previous Router state"
  }
  $restoredConfig = Get-Content $configPath -Raw
  if ($restoredConfig -notmatch [regex]::Escape($stateBeforeRestore.gatewayBaseUrl) -or
      $restoredConfig -notmatch [regex]::Escape($stateBeforeRestore.model)) {
    throw "Restore did not recover the previous Codex config"
  }
  if ($stateBeforeRestore.tokenMode -eq "static") {
    $restoredToken = (& $assistantExe --codex-assistant-token-helper $runtimeConfigPath | Out-String).Trim()
    if ($restoredToken -ne $tokenBeforeRestore) { throw "Restore did not recover the previous DPAPI key" }
    $restoredToken = $null
  }
  $tokenBeforeRestore = $null
  $restoreTransaction = Get-Content $lastTransactionPath -Raw | ConvertFrom-Json
  if ($restoreTransaction.status -ne "committed" -or $restoreTransaction.operation -ne "restore") {
    throw "Restore did not commit its own reversible transaction"
  }
  $restoreManifest = Assert-TransactionManifest $restoreTransaction "committed" "restore"
  if (Test-Path $activeTransactionPath) {
    throw "Active transaction journal remained after restore"
  }
  $restoreRoundTrip = $true
}

$chatGptLaunched = $false
if ($LaunchAfterSetup) {
  [void](Invoke-CdpExpression $socket @'
(async () => {
  const button = document.querySelector('#launchButton');
  button.click();
  const confirm = document.querySelector('#confirmAcceptButton');
  const confirmDeadline = Date.now() + 5000;
  while (document.querySelector('#confirmOverlay').classList.contains('hidden') && Date.now() < confirmDeadline) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (document.querySelector('#confirmOverlay').classList.contains('hidden')) {
    throw new Error('Restart confirmation did not open');
  }
  confirm.click();
  const deadline = Date.now() + 30000;
  while (button.disabled && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 100));
  return !button.disabled;
})()
'@ 3)
  [void](Wait-Until { (Get-ChatGptProcessCount) -gt 0 } 45 "ChatGPT did not start after the explicit launch action")
  $chatGptLaunched = $true
}

$appearanceApplied = "none"
$appearanceDomTheme = ""
$appearanceStylePresent = $false
$appearanceBlockedByFirstRun = $false
if ($ApplyAppearance -ne "none") {
  $themeImportScript = ""
  if ($ApplyAppearance -eq "custom") {
    if (!$ThemeImagePath) { throw "-ThemeImagePath is required for the custom appearance test" }
    $resolvedThemeImage = (Resolve-Path $ThemeImagePath).Path
    $themeImageBytes = [IO.File]::ReadAllBytes($resolvedThemeImage)
    if ($themeImageBytes.Length -gt 8MB) { throw "Theme test image exceeds the 8 MB product limit" }
    $themeMimeType = switch ([IO.Path]::GetExtension($resolvedThemeImage).ToLowerInvariant()) {
      ".png" { "image/png" }
      ".jpg" { "image/jpeg" }
      ".jpeg" { "image/jpeg" }
      ".webp" { "image/webp" }
      default { throw "Theme test image must be PNG, JPEG, or WebP" }
    }
    $themeDataUrl = "data:$themeMimeType;base64,$([Convert]::ToBase64String($themeImageBytes))"
    $themeFileNameJson = [IO.Path]::GetFileName($resolvedThemeImage) | ConvertTo-Json -Compress
    $themeMimeTypeJson = $themeMimeType | ConvertTo-Json -Compress
    $themeDataUrlJson = $themeDataUrl | ConvertTo-Json -Compress
    $themeImportScript = @"
  await window.__TAURI__.core.invoke('import_theme_image', {
    request: { fileName: $themeFileNameJson, mimeType: $themeMimeTypeJson, dataUrl: $themeDataUrlJson }
  });
"@
  }
  $themeJson = $ApplyAppearance | ConvertTo-Json -Compress
  $appearance = Invoke-CdpExpression $socket @"
(async () => {
  $themeImportScript
  document.querySelector('[data-view="appearance"]').click();
  if ($themeJson === 'custom') {
    const readyDeadline = Date.now() + 5000;
    while (document.querySelector('#customThemeLabel').textContent.includes('选择') && Date.now() < readyDeadline) {
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    if (document.querySelector('#customThemeLabel').textContent.includes('选择')) {
      throw new Error('Imported custom background was not reflected in the UI');
    }
  }
  document.querySelector('[data-theme=$themeJson]').click();
  const button = document.querySelector('#applyAppearanceButton');
  button.click();
  const confirmDeadline = Date.now() + 5000;
  while (document.querySelector('#confirmOverlay').classList.contains('hidden') && Date.now() < confirmDeadline) {
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (document.querySelector('#confirmOverlay').classList.contains('hidden')) {
    throw new Error('Appearance confirmation did not open');
  }
  document.querySelector('#confirmAcceptButton').click();
  const deadline = Date.now() + 70000;
  const badge = document.querySelector('#appearanceStatus');
  let observedBusy = false;
  while (Date.now() < deadline) {
    if (button.disabled || badge.textContent.trim() === '应用中') observedBusy = true;
    if (observedBusy && !button.disabled && badge.textContent.trim() !== '应用中') break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (!observedBusy || button.disabled || badge.textContent.trim() === '应用中') {
    throw new Error('Appearance operation did not complete');
  }
  return {
    success: badge.classList.contains('success'),
    badge: badge.textContent.trim(),
    message: document.querySelector('#appearanceMessage').textContent.trim()
  };
})()
"@ 4
  $appearanceStatePath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\appearance.json"
  if (!$appearance.success) {
    if ($appearance.message -notmatch "Finish Windows setup") {
      throw "Appearance UI failed: $($appearance.badge) $($appearance.message)"
    }
    $fallback = Wait-Until {
      if (Test-Path $appearanceStatePath) {
        $candidate = Get-Content $appearanceStatePath -Raw | ConvertFrom-Json
        if ($candidate.selectedTheme -eq "official") { return $candidate }
      }
      return $null
    } 10 "Appearance fallback state was not saved"
    $appearanceApplied = $fallback.selectedTheme
    $appearanceBlockedByFirstRun = $true
  } else {
    $appearanceState = Wait-Until {
      if (Test-Path $appearanceStatePath) {
        $candidate = Get-Content $appearanceStatePath -Raw | ConvertFrom-Json
        if ($candidate.selectedTheme -eq $ApplyAppearance) { return $candidate }
      }
      return $null
    } 10 "Appearance state was not saved"
    $appearanceApplied = $appearanceState.selectedTheme

    if ($ApplyAppearance -in @("focus", "custom")) {
      $chatGptTargets = Wait-Until {
        try { Invoke-RestMethod -Uri "http://127.0.0.1:$($appearanceState.port)/json/list" -TimeoutSec 2 } catch { $null }
      } 20 "ChatGPT theme debug endpoint did not start"
      $chatGptTarget = $chatGptTargets | Where-Object { $_.type -eq "page" -and $_.url -like "app://*" } | Select-Object -First 1
      if (!$chatGptTarget) { throw "ChatGPT page target was not found after applying theme" }
      $chatGptSocket = [Net.WebSockets.ClientWebSocket]::new()
      [void]$chatGptSocket.ConnectAsync([Uri]$chatGptTarget.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()
      $themeDom = Invoke-CdpExpression $chatGptSocket @'
({
  theme: document.documentElement.dataset.codexAssistantTheme || '',
  stylePresent: Boolean(document.getElementById('codex-assistant-theme-style')),
  customImagePresent: getComputedStyle(document.body).backgroundImage.includes('blob:'),
  background: getComputedStyle(document.body).backgroundColor
})
'@ 5
      $chatGptSocket.Dispose()
      if ($themeDom.theme -ne $ApplyAppearance -or !$themeDom.stylePresent) { throw "Theme was not injected into ChatGPT" }
      if ($ApplyAppearance -eq "custom" -and !$themeDom.customImagePresent) {
        throw "Custom background image was not injected into ChatGPT"
      }
      $appearanceDomTheme = $themeDom.theme
      $appearanceStylePresent = [bool]$themeDom.stylePresent
    }
  }
}

$socket.Dispose()

[pscustomobject]@{
  success = $true
  router = $RouterUrl
  models = $connection.models
  selectedModel = $connection.selectedModel
  statusConsistency = [bool]$connection.statusConsistency
  transactionId = $runtimeState.transactionId
  transactionStatus = $committedTransaction.status
  installTaskState = $result.installTaskState
  keyConfigured = $keyConfigured
  resultTitle = $result.title
  localOllamaDiagnostic = $connection.localOllamaDiagnostic
  chatGptProcessCountDuringSetup = $chatGptAfter
  chatGptLaunchedAfterExplicitAction = $chatGptLaunched
  backupAvailableInUi = [bool]($backupUiVisible.home -and $backupUiVisible.diagnostics)
  diagnosticSupportId = $diagnosticBundle.supportId
  diagnosticBundleSha256 = $diagnosticBundle.sha256
  diagnosticUiDownloadSha256 = $diagnosticUiDownloadSha256
  restoreRoundTrip = $restoreRoundTrip
  appearanceApplied = $appearanceApplied
  appearanceDomTheme = $appearanceDomTheme
  appearanceStylePresent = $appearanceStylePresent
  appearanceBlockedByFirstRun = $appearanceBlockedByFirstRun
  configPath = $configPath
  runtimeConfigPath = $runtimeConfigPath
} | ConvertTo-Json -Depth 5 -Compress
