# Ensures a clang executable is available for ring's build script.
#
# Lookup mirrors Find-LlvmBin in windows-build.ps1. When no clang is found
# (the case on a freshly provisioned self-hosted runner), the official
# portable LLVM toolchain is downloaded and extracted under a per-user
# directory that persists across runs, then its bin directory is appended to
# GITHUB_PATH so later steps (and Find-LlvmBin's PATH fallback) can find it.
# The portable archive needs no administrator rights.
param(
  [string]$Version = "18.1.8",
  [string]$InstallRoot = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$toolchainName = "clang+llvm-$Version-x86_64-pc-windows-msvc"
$downloadUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$Version/$toolchainName.tar.xz"

function Test-ClangAvailable {
  $clangOnPath = Get-Command clang.exe -ErrorAction SilentlyContinue
  if ($clangOnPath) { return $clangOnPath.Source }

  $candidates = @("C:\Program Files\LLVM\bin\clang.exe")
  $programFilesX86 = ${env:ProgramFiles(x86)}
  if ($programFilesX86) {
    $candidates += Join-Path $programFilesX86 "Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\x64\bin\clang.exe"
    $candidates += Join-Path $programFilesX86 "Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\bin\clang.exe"
  }
  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) { return $candidate }
  }
  return $null
}

$existing = Test-ClangAvailable
if ($existing) {
  Write-Host "clang is already available at $existing"
  exit 0
}

if (!$InstallRoot) {
  $InstallRoot = Join-Path $env:LOCALAPPDATA "llvm-portable"
}
$toolchainDirectory = Join-Path $InstallRoot $toolchainName
$clangPath = Join-Path $toolchainDirectory "bin\clang.exe"

if (!(Test-Path $clangPath)) {
  if (Test-Path $toolchainDirectory) {
    Remove-Item $toolchainDirectory -Recurse -Force
  }
  New-Item -ItemType Directory -Force $InstallRoot | Out-Null

  $archivePath = Join-Path $env:TEMP "$toolchainName.tar.xz"
  Write-Host "Downloading $downloadUrl"
  Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
  Write-Host "Extracting $archivePath to $InstallRoot"
  tar.exe -xf $archivePath -C $InstallRoot
  if ($LASTEXITCODE -ne 0) {
    throw "Failed to extract $archivePath"
  }
  Remove-Item $archivePath -Force
}

if (!(Test-Path $clangPath)) {
  throw "clang.exe was not found at $clangPath after installing LLVM $Version"
}

$binDirectory = Join-Path $toolchainDirectory "bin"
Write-Host "Using portable LLVM at $binDirectory"
$env:PATH = "$binDirectory;$env:PATH"
if ($env:GITHUB_PATH) {
  # Windows PowerShell 的 >> 会写出 UTF-16，GITHUB_PATH 必须是 UTF-8。
  $binDirectory | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
}
& $clangPath --version
