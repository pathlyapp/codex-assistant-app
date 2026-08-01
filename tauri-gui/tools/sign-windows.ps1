# sign-windows.ps1
# Signs a single file with the SafeNet eToken UKey certificate.
# Invoked by Tauri through bundle.windows.signCommand ("%1" is the file path),
# or manually: powershell -File tools/sign-windows.ps1 <file>
#
# Modes:
#   1. Silent mode (CI/CD, used when WIN_SIGN_PIN is set):
#      Reads the UKey certificate container/provider via get-cert-info.ps1 and
#      signs with the signtool hidden syntax /kc "[{{PIN}}]=ContainerName".
#      Note: this requires the SafeNet driver installed WITHOUT the "Minidriver"
#      component (custom install).
#   2. Interactive mode (fallback, no WIN_SIGN_PIN):
#      signtool sign /a, the SafeNet driver shows the PIN prompt. With SafeNet
#      "Single Logon" enabled only the first build asks for the PIN.
#
# Optional environment variables:
#   SIGNTOOL_PATH           explicit signtool.exe path
#   WIN_SIGN_SUBJECT_NAME   certificate subject filter for get-cert-info.ps1
#   WIN_SIGN_TIMESTAMP_URL  RFC3161 timestamp server (default http://timestamp.digicert.com)
#
# This script is written entirely in ASCII to avoid Windows PowerShell encoding issues.

param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string]$FilePath
)

$ErrorActionPreference = "Stop"

if (!(Test-Path $FilePath)) {
  throw "File to sign was not found: $FilePath"
}

function Find-SignTool {
  if ($env:SIGNTOOL_PATH -and (Test-Path $env:SIGNTOOL_PATH)) {
    return $env:SIGNTOOL_PATH
  }
  $kitsBase = "C:\Program Files (x86)\Windows Kits\10\bin"
  if (Test-Path $kitsBase) {
    $versions = Get-ChildItem $kitsBase -Directory | Sort-Object Name -Descending
    foreach ($version in $versions) {
      foreach ($arch in @("x64", "arm64")) {
        $candidate = Join-Path $version.FullName "$arch\signtool.exe"
        if (Test-Path $candidate) {
          return $candidate
        }
      }
    }
  }
  $command = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($command) {
    return $command.Source
  }
  throw "signtool.exe was not found. Install the Windows SDK or set SIGNTOOL_PATH."
}

$signtool = Find-SignTool
$timestampUrl = $env:WIN_SIGN_TIMESTAMP_URL
if (!$timestampUrl) {
  $timestampUrl = "http://timestamp.digicert.com"
}

Write-Host "[sign] target:   $FilePath"
Write-Host "[sign] signtool: $signtool"

$pin = $env:WIN_SIGN_PIN
if ($pin) {
  $getCertInfoScript = Join-Path $PSScriptRoot "get-cert-info.ps1"
  $certInfoJson = (& $getCertInfoScript | Out-String).Trim()
  if (!$certInfoJson) {
    throw "get-cert-info.ps1 returned no certificate information."
  }
  $certInfo = $certInfoJson | ConvertFrom-Json
  Write-Host "[sign] certificate: $($certInfo.subject)"
  Write-Host "[sign] container:   $($certInfo.container)"
  Write-Host "[sign] provider:    $($certInfo.provider)"

  $tempCerPath = $certInfo.tempCerPath
  try {
    # SafeNet hidden syntax: /kc "[{{PIN}}]=ContainerName"
    # The double braces around the PIN are required by the eToken CSP.
    $arguments = @(
      "sign",
      "/f", $tempCerPath,
      "/csp", $certInfo.provider,
      "/kc", "[{{$pin}}]=$($certInfo.container)",
      "/fd", "sha256",
      "/tr", $timestampUrl,
      "/td", "sha256",
      $FilePath
    )
    $maskedCommand = "$signtool " + (($arguments | ForEach-Object { "`"$_`"" }) -join " ")
    Write-Host "[sign] run: " + $maskedCommand.Replace($pin, "******")
    & $signtool @arguments
    if ($LASTEXITCODE -ne 0) {
      throw "signtool failed with exit code $LASTEXITCODE"
    }
  } finally {
    if ($tempCerPath -and (Test-Path $tempCerPath)) {
      Remove-Item $tempCerPath -Force -ErrorAction SilentlyContinue
    }
  }
} else {
  Write-Host "[sign] WIN_SIGN_PIN not set, using interactive certificate store signing (/a)."
  $subjectName = $env:WIN_SIGN_SUBJECT_NAME
  $arguments = @("sign")
  if ($subjectName) {
    $arguments += @("/n", $subjectName)
  } else {
    $arguments += "/a"
  }
  $arguments += @("/fd", "sha256", "/tr", $timestampUrl, "/td", "sha256", $FilePath)
  & $signtool @arguments
  if ($LASTEXITCODE -ne 0) {
    throw "signtool failed with exit code $LASTEXITCODE"
  }
}

Write-Host "[sign] success: $FilePath"
