param(
  [string]$RouterUrl = "http://10.211.55.2:11435/v1",
  [int]$DebugPort = 9224
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

$assistantExe = Get-ChildItem $env:LOCALAPPDATA -Filter "codex-assistant.exe" -Recurse -Depth 4 |
  Select-Object -First 1 -ExpandProperty FullName
if (!$assistantExe) { throw "Installed codex-assistant.exe was not found" }

$configPath = Join-Path $env:USERPROFILE ".codex\config.toml"
$runtimePath = Join-Path $env:LOCALAPPDATA "CodexAssistant\runtime\config.json"
if (!(Test-Path $configPath) -or !(Test-Path $runtimePath)) {
  throw "Repair E2E requires an existing managed configuration"
}
$runtimeBefore = Get-Content $runtimePath -Raw | ConvertFrom-Json
if ($runtimeBefore.gatewayBaseUrl -ne $RouterUrl) {
  throw "Saved Router does not match the repair fixture: $($runtimeBefore.gatewayBaseUrl)"
}
if ($runtimeBefore.responsesVerifiedAt -or $runtimeBefore.responsesProtocol) {
  throw "Repair E2E requires invalidated Responses evidence"
}
$configHashBefore = (Get-FileHash $configPath -Algorithm SHA256).Hash.ToLowerInvariant()

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
  const before = await window.__TAURI__.core.invoke('get_system_status');
  const plan = await window.__TAURI__.core.invoke('get_repair_plan', {
    request: { errorCode: 'ROUTER_RESPONSES_UNSUPPORTED' }
  });
  document.querySelector('[data-view="diagnostics"]').click();
  const button = document.querySelector('#repairActionButton');
  const planDeadline = Date.now() + 10000;
  while (button.classList.contains('hidden') && Date.now() < planDeadline) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (button.classList.contains('hidden')) throw new Error('Targeted repair button is hidden');
  if (button.dataset.actionId !== 'revalidate_router') {
    throw new Error('Unexpected repair action: ' + button.dataset.actionId);
  }
  button.click();
  let observedBusy = false;
  const repairDeadline = Date.now() + 120000;
  const receipt = document.querySelector('#repairResult');
  while (Date.now() < repairDeadline) {
    if (button.disabled) observedBusy = true;
    if (observedBusy && !receipt.classList.contains('hidden')) break;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  if (!observedBusy || receipt.classList.contains('hidden')) {
    throw new Error('Repair action did not complete');
  }
  const after = await window.__TAURI__.core.invoke('get_system_status');
  const planAfter = await window.__TAURI__.core.invoke('get_repair_plan', {
    request: { errorCode: 'ROUTER_RESPONSES_UNSUPPORTED' }
  });
  return {
    before: {
      overall: before.overall,
      routerState: before.router?.state,
      lastVerifiedAt: before.router?.lastVerifiedAt
    },
    plan: {
      state: plan.state,
      actionId: plan.action?.id || ''
    },
    receipt: {
      actionId: receipt.dataset.actionId || '',
      changed: receipt.dataset.changed || '',
      beforeRouterState: receipt.dataset.beforeRouterState || '',
      afterRouterState: receipt.dataset.afterRouterState || '',
      text: receipt.textContent.trim()
    },
    after: {
      overall: after.overall,
      routerState: after.router?.state,
      lastVerifiedAt: after.router?.lastVerifiedAt,
      ready: after.ready
    },
    planAfter: {
      state: planAfter.state,
      hasAction: Boolean(planAfter.action)
    },
    buttonHiddenAfter: button.classList.contains('hidden')
  };
})()
'@ 1
} finally {
  $socket.Dispose()
  Get-Process codex-assistant -ErrorAction SilentlyContinue | Stop-Process -Force
}

$configHashAfter = (Get-FileHash $configPath -Algorithm SHA256).Hash.ToLowerInvariant()
$runtimeAfter = Get-Content $runtimePath -Raw | ConvertFrom-Json
if ($result.before.routerState -ne "models_verified" -or
    $result.plan.state -ne "action_available" -or
    $result.plan.actionId -ne "revalidate_router") {
  throw "Initial repair state is invalid: $($result | ConvertTo-Json -Depth 8 -Compress)"
}
if ($result.receipt.actionId -ne "revalidate_router" -or
    $result.receipt.changed -ne "true" -or
    $result.receipt.beforeRouterState -ne "models_verified" -or
    $result.receipt.afterRouterState -ne "responses_verified") {
  throw "Repair receipt is invalid: $($result.receipt | ConvertTo-Json -Compress)"
}
if ($result.after.overall -ne "ready" -or
    $result.after.routerState -ne "responses_verified" -or
    !$result.after.lastVerifiedAt -or
    !$result.after.ready -or
    $result.planAfter.state -ne "not_needed" -or
    $result.planAfter.hasAction -or
    !$result.buttonHiddenAfter) {
  throw "Repair did not reach a stable ready state: $($result | ConvertTo-Json -Depth 8 -Compress)"
}
if ($configHashAfter -ne $configHashBefore) {
  throw "Targeted Router repair changed Codex config.toml"
}
if (!$runtimeAfter.responsesVerifiedAt -or
    $runtimeAfter.responsesProtocol -notin @("sse", "json")) {
  throw "Targeted Router repair did not persist Responses evidence"
}

[ordered]@{
  success = $true
  actionId = $result.receipt.actionId
  changed = $true
  beforeRouterState = $result.receipt.beforeRouterState
  afterRouterState = $result.receipt.afterRouterState
  configUnchanged = $true
  lastVerifiedAt = $runtimeAfter.responsesVerifiedAt
  responsesProtocol = $runtimeAfter.responsesProtocol
  planAfter = $result.planAfter.state
  repairText = $result.receipt.text
} | ConvertTo-Json -Compress
