param(
  [string]$RouterUrl = "http://10.211.55.2:11435/v1",
  [string]$RouterKey = "",
  [int]$DebugPort = 9223,
  [switch]$LaunchAfterSetup,
  [switch]$TestRestore,
  [ValidateSet("none", "focus", "custom", "official")]
  [string]$ApplyAppearance = "none",
  [string]$ThemeImagePath = ""
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

function Get-ChatGptProcessCount {
  $count = 0
  Get-Process -ErrorAction SilentlyContinue | ForEach-Object {
    try {
      if ($_.Path -like "*\WindowsApps\OpenAI.Codex_*\*") { $count++ }
    } catch {}
  }
  return $count
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
  document.querySelector('[data-view="setup"]').click();
  const status = await window.__TAURI__.core.invoke('get_system_status');
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
    localOllamaDiagnostic,
    models: [...document.querySelector('#modelInput').options].map(option => option.value).filter(Boolean),
    selectedModel: document.querySelector('#modelInput').value
  };
})()
"@ 1

if ($connection.className -notmatch "success") { throw "Router UI test failed: $($connection.message)" }
if ($connection.gateway -ne $RouterUrl) { throw "Router input changed unexpectedly: $($connection.gateway)" }
if (!$connection.selectedModel) { throw "Router returned no selectable model" }

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
    logs: document.querySelector('#logOutput').textContent
  };
})()
'@ 2
$chatGptAfter = Get-ChatGptProcessCount

if (!$result.visible -or !$result.success) {
  throw "Configuration UI did not complete: $($result.title) $($result.message)"
}
if ($chatGptBefore -ne 0 -or $chatGptAfter -ne 0) {
  throw "ChatGPT was running during configuration (before=$chatGptBefore, after=$chatGptAfter)"
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
  keyConfigured = $keyConfigured
  resultTitle = $result.title
  localOllamaDiagnostic = $connection.localOllamaDiagnostic
  chatGptProcessCountDuringSetup = $chatGptAfter
  chatGptLaunchedAfterExplicitAction = $chatGptLaunched
  backupAvailableInUi = [bool]($backupUiVisible.home -and $backupUiVisible.diagnostics)
  restoreRoundTrip = $restoreRoundTrip
  appearanceApplied = $appearanceApplied
  appearanceDomTheme = $appearanceDomTheme
  appearanceStylePresent = $appearanceStylePresent
  appearanceBlockedByFirstRun = $appearanceBlockedByFirstRun
  configPath = $configPath
  runtimeConfigPath = $runtimeConfigPath
} | ConvertTo-Json -Depth 5 -Compress
