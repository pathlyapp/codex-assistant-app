# windows-arch.ps1
# Dot-source helper shared by the Windows build and E2E scripts.
#
# [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture returns
# $null under Windows PowerShell 5.1, so every caller needs the environment
# variable fallback below to keep working outside PowerShell 7.

function Get-WindowsNativeArchitecture {
  $runtimeArchitecture = $null
  try {
    $runtimeArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
  } catch {
    $runtimeArchitecture = $null
  }
  if ($runtimeArchitecture) {
    return $runtimeArchitecture.ToString()
  }

  # A 32-bit host process reports x86 in PROCESSOR_ARCHITECTURE; the real OS
  # architecture is then exposed through PROCESSOR_ARCHITEW6432.
  $processorArchitecture = $env:PROCESSOR_ARCHITEW6432
  if (!$processorArchitecture) {
    $processorArchitecture = $env:PROCESSOR_ARCHITECTURE
  }

  # Normalized to the RuntimeInformation spelling so callers can switch on one
  # set of values regardless of which branch produced the result.
  switch ($processorArchitecture) {
    "ARM64" { "Arm64" }
    "AMD64" { "X64" }
    "x86" { "X86" }
    default { $processorArchitecture }
  }
}
