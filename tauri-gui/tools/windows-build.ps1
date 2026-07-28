param(
  [ValidateSet("auto", "x64", "arm64")]
  [string]$Architecture = "auto",
  [string]$BuildToolsRoot = "C:\BuildTools",
  [string]$OutputDirectory = "",
  [switch]$SkipNpmInstall
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Import-VcEnvironment([string]$VcVarsAll, [string]$TargetArchitecture) {
  $environment = & cmd.exe /d /s /c "call `"$VcVarsAll`" $TargetArchitecture >nul && set"
  if ($LASTEXITCODE -ne 0) {
    throw "Visual Studio environment initialization failed for $TargetArchitecture"
  }

  foreach ($line in $environment) {
    $separator = $line.IndexOf("=")
    if ($separator -le 0) { continue }
    $name = $line.Substring(0, $separator)
    $value = $line.Substring($separator + 1)
    Set-Item -Path "Env:$name" -Value $value
  }
}

function Invoke-Checked([string]$Program, [string[]]$Arguments) {
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "$Program failed with exit code $LASTEXITCODE"
  }
}

$toolsDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$guiRoot = Split-Path -Parent $toolsDirectory
if (!$OutputDirectory) {
  $OutputDirectory = Join-Path $guiRoot "artifact"
}

$nativeArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Architecture -eq "auto") {
  $Architecture = switch ($nativeArchitecture) {
    "Arm64" { "arm64" }
    "X64" { "x64" }
    default { throw "Unsupported Windows architecture: $nativeArchitecture" }
  }
}

$target = switch ($Architecture) {
  "arm64" { "aarch64-pc-windows-msvc" }
  "x64" { "x86_64-pc-windows-msvc" }
}
$vcArchitecture = switch ($Architecture) {
  "arm64" { "arm64" }
  "x64" { "x64" }
}
$llvmArchitecture = switch ($Architecture) {
  "arm64" { "ARM64" }
  "x64" { "x64" }
}

$vcVarsAll = Join-Path $BuildToolsRoot "VC\Auxiliary\Build\vcvarsall.bat"
if (!(Test-Path $vcVarsAll)) {
  throw "Visual Studio Build Tools were not found at $vcVarsAll"
}
Import-VcEnvironment $vcVarsAll $vcArchitecture

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$env:PATH = "$cargoBin;$env:PATH"

$llvmBin = Join-Path $BuildToolsRoot "VC\Tools\Llvm\$llvmArchitecture\bin"
if (!(Test-Path (Join-Path $llvmBin "clang.exe"))) {
  throw "LLVM for $Architecture was not found at $llvmBin"
}
$env:PATH = "$llvmBin;$env:PATH"

$installedTargets = @(& rustup target list --installed)
if ($LASTEXITCODE -ne 0) {
  throw "rustup target inspection failed"
}
if ($installedTargets -notcontains $target) {
  throw "Rust target $target is missing. Run: rustup target add $target"
}

Push-Location $guiRoot
try {
  if (!$SkipNpmInstall) {
    Invoke-Checked "npm.cmd" @("ci")
  }
  Invoke-Checked "node.exe" @("tools/check-version.mjs")
  Invoke-Checked "cargo.exe" @(
    "test",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--target",
    $target
  )
  Invoke-Checked "npm.cmd" @(
    "run",
    "build:windows",
    "--",
    "--target",
    $target
  )

  $version = (& node.exe -p "require('./package.json').version").Trim()
  if ($LASTEXITCODE -ne 0 -or !$version) {
    throw "Package version could not be read"
  }
  $bundleDirectory = Join-Path $guiRoot "src-tauri\target\$target\release\bundle\nsis"
  $installer = Get-ChildItem $bundleDirectory -Filter "*.exe" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
  if (!$installer) {
    throw "NSIS installer was not produced in $bundleDirectory"
  }

  New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
  $artifactPath = Join-Path $OutputDirectory "CodexAssistant-$version-windows-$Architecture-setup.exe"
  Copy-Item $installer.FullName $artifactPath -Force
  $artifact = Get-Item $artifactPath
  $hash = (Get-FileHash $artifact.FullName -Algorithm SHA256).Hash.ToLowerInvariant()

  [ordered]@{
    schemaVersion = 1
    version = $version
    architecture = $Architecture
    target = $target
    nativeArchitecture = $nativeArchitecture
    installer = $artifact.FullName
    bytes = $artifact.Length
    sha256 = $hash
  } | ConvertTo-Json -Compress
} finally {
  Pop-Location
}
