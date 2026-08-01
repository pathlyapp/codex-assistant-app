# get-cert-info.ps1
# Extract UKey code signing certificate container and provider name, and export public cert.
# This script is written entirely in ASCII to avoid Windows PowerShell encoding issues.
# Ported from deeppath-desktop/scripts/get-cert-info.ps1.

$ErrorActionPreference = "Stop"

# 1. Find code signing certificate
$subjectName = $env:WIN_SIGN_SUBJECT_NAME
$certs = @()

if ($subjectName) {
    $certs = Get-ChildItem -Path Cert:\CurrentUser\My | Where-Object { $_.Subject -like "*$subjectName*" }
} else {
    # Find certificates with Code Signing EKU (1.3.6.1.5.5.7.3.3)
    $certs = Get-ChildItem -Path Cert:\CurrentUser\My | Where-Object {
        ($_.EnhancedKeyUsageList | Where-Object { $_.FriendlyName -eq "Code Signing" -or $_.ObjectId -eq "1.3.6.1.5.5.7.3.3" -or $_.Value -eq "1.3.6.1.5.5.7.3.3" }) -ne $null
    }
}

if ($certs.Count -eq 0) {
    Write-Error "No valid code signing certificate found in Personal store. Please ensure UKey is inserted and driver is installed."
}

# Select the first matching certificate
$cert = $certs[0]

# 2. Extract private key container and provider info
$containerName = $null
$providerName = $null

try {
    if ($cert.PrivateKey -ne $null) {
        $containerName = $cert.PrivateKey.CspKeyContainerInfo.KeyContainerName
        $providerName = $cert.PrivateKey.CspKeyContainerInfo.ProviderName
    }
} catch {}

if (-not $containerName) {
    try {
        $asymmetricKey = [System.Security.Cryptography.X509Certificates.RSACertificateExtensions]::GetRSAPrivateKey($cert)
        if ($asymmetricKey -ne $null) {
            # Check if it's a CNG key
            if ($asymmetricKey.GetType().FullName -like "*Cng*") {
                $containerName = $asymmetricKey.Key.KeyName
                $providerName = $asymmetricKey.Key.Provider.Provider
            }
        }
    } catch {}
}

# Force legacy eToken provider for silent signing via CryptoAPI
if (-not $providerName -or $providerName -like "*Key Storage Provider*") {
    $providerName = "eToken Base Cryptographic Provider"
}

# If container name is still empty, try to get it via certutil
if (-not $containerName) {
    try {
        $certutilOut = certutil -user -store My $cert.SerialNumber
        # Match lines containing GUID or te- container name pattern
        $containerLine = $certutilOut | Where-Object { $_ -match '(?i)te-[0-9a-f]{8}-' -or $_ -match '(?i)[0-9a-f]{8}-[0-9a-f]{4}-' }
        if ($containerLine -and ($containerLine -match ':\s*(.+)$' -or $containerLine -match '=\s*(.+)$')) {
            $containerName = $Matches[1].Trim()
        }
    } catch {}
}

if (-not $containerName) {
    Write-Error "Failed to extract private key container name. Please ensure UKey is ready."
}

# 3. Export certificate to temporary .cer file
$tempCerPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "temp_code_sign.cer")
[System.IO.File]::WriteAllBytes($tempCerPath, $cert.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert))

# 4. Output JSON result
$result = @{
    subject = $cert.Subject
    thumbprint = $cert.Thumbprint
    container = $containerName
    provider = $providerName
    tempCerPath = $tempCerPath
} | ConvertTo-Json -Compress

Write-Output $result
