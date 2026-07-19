param(
  [int]$Port = 9335,
  [string]$ScreenshotPath = "C:\Temp\codex-theme-inspect.png"
)

$ErrorActionPreference = "Stop"

function Invoke-CdpCommand(
  [System.Net.WebSockets.ClientWebSocket]$Socket,
  [string]$Method,
  [hashtable]$Params,
  [int]$Id
) {
  $request = @{ id = $Id; method = $Method; params = $Params } | ConvertTo-Json -Depth 8 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($request)
  [void]$Socket.SendAsync(
    [ArraySegment[byte]]::new($bytes),
    [Net.WebSockets.WebSocketMessageType]::Text,
    $true,
    [Threading.CancellationToken]::None
  ).GetAwaiter().GetResult()

  while ($true) {
    $buffer = New-Object byte[] 1048576
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
      throw "Browser expression failed: $($response.result.exceptionDetails.text)"
    }
    return $response.result
  }
}

$targets = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/list" -TimeoutSec 3
$target = $targets | Where-Object { $_.type -eq "page" -and $_.url -like "app://*" } | Select-Object -First 1
if (!$target) { throw "ChatGPT page target was not found on port $Port" }

$socket = [Net.WebSockets.ClientWebSocket]::new()
[void]$socket.ConnectAsync(
  [Uri]$target.webSocketDebuggerUrl,
  [Threading.CancellationToken]::None
).GetAwaiter().GetResult()

$expression = @'
(() => {
  const selectors = {
    html: 'html',
    body: 'body',
    sidebar: 'aside.app-shell-left-panel',
    main: 'main.main-surface',
    header: 'main.main-surface > header.app-header-tint',
    roleMain: 'main.main-surface [role="main"]',
    frame: '.app-shell-main-content-frame',
    fullSurface: '[class~="bg-token-main-surface-primary"][class~="h-full"][class~="w-full"]',
    thread: '.thread-scroll-container',
    composer: '.composer-surface-chrome'
  };
  const result = {};
  for (const [name, selector] of Object.entries(selectors)) {
    const element = document.querySelector(selector);
    if (!element) { result[name] = null; continue; }
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    result[name] = {
      tag: element.tagName,
      className: String(element.className).slice(0, 400),
      backgroundColor: style.backgroundColor,
      backgroundImage: style.backgroundImage.slice(0, 180),
      opacity: style.opacity,
      backdropFilter: style.backdropFilter,
      rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height }
    };
  }
  result.theme = document.documentElement.dataset.codexAssistantTheme || '';
  result.artVariable = getComputedStyle(document.documentElement).getPropertyValue('--codex-assistant-art').slice(0, 180);
  const injectedStyle = document.getElementById('codex-assistant-theme-style');
  result.injectedStyle = injectedStyle ? {
    length: injectedStyle.textContent.length,
    ruleCount: injectedStyle.sheet?.cssRules?.length ?? -1,
    firstRule: injectedStyle.sheet?.cssRules?.[0]?.cssText?.slice(0, 240) ?? ''
  } : null;
  const textSamples = [...document.querySelectorAll('h1,h2,h3,p,span,button,a')]
    .filter((element) => {
      const text = element.textContent?.trim() || '';
      return text === 'What should we build?' || text === 'Codex' || text === 'Choose project';
    })
    .slice(0, 12)
    .map((element) => ({
      text: element.textContent.trim(),
      tag: element.tagName,
      className: String(element.className).slice(0, 400),
      color: getComputedStyle(element).color,
      parentClassName: String(element.parentElement?.className || '').slice(0, 400)
    }));
  result.textSamples = textSamples;
  result.sidebarControls = [...document.querySelectorAll('aside.app-shell-left-panel button, aside.app-shell-left-panel a')]
    .slice(0, 8)
    .map((element) => ({ text: element.textContent.trim(), color: getComputedStyle(element).color }));
  result.viewport = { width: innerWidth, height: innerHeight, devicePixelRatio };
  return result;
})()
'@

$evaluation = Invoke-CdpCommand $socket "Runtime.evaluate" @{
  expression = $expression
  awaitPromise = $true
  returnByValue = $true
} 1
$capture = Invoke-CdpCommand $socket "Page.captureScreenshot" @{
  format = "png"
  captureBeyondViewport = $false
} 2
$socket.Dispose()

[IO.File]::WriteAllBytes($ScreenshotPath, [Convert]::FromBase64String($capture.data))
[pscustomobject]@{
  screenshotPath = $ScreenshotPath
  inspection = $evaluation.result.value
} | ConvertTo-Json -Depth 10 -Compress
