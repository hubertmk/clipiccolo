# Creates Clipiccolo's self-signed code-signing certificate (run once, on the maintainer's PC).
#
#   .\signing\new-cert.ps1
#
# - Puts the certificate with its private key into your user certificate store (Cert:\CurrentUser\My), where
#   build-installer.ps1 and build-portable.ps1 find it and sign with it automatically.
# - Exports the public part to signing\clipiccolo-codesign.cer (commit it: people/IT can import it to trust builds).
# - Exports certificate + private key to a password-protected .pfx OUTSIDE the repo, and prints the two values to
#   store as GitHub Actions secrets (SIGNING_PFX_BASE64, SIGNING_PFX_PASSWORD) for the release workflow.
#
# Keep the .pfx and its password private: anyone with both can sign programs as "Clipiccolo".
param(
    [string]$OutDir = (Join-Path $env:USERPROFILE '.clipiccolo-signing'),
    [int]$Years = 5,
    [switch]$Force
)
$ErrorActionPreference = 'Stop'
$subject = 'CN=Clipiccolo'

$existing = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Where-Object Subject -eq $subject
if ($existing -and -not $Force) {
    throw "A '$subject' code-signing certificate already exists ($($existing.Thumbprint -join ', ')). Use -Force to create another."
}

$cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject `
    -FriendlyName 'Clipiccolo code signing (self-signed)' `
    -KeyAlgorithm RSA -KeyLength 3072 -HashAlgorithm SHA256 -KeyExportPolicy Exportable `
    -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddYears($Years)

New-Item -ItemType Directory -Force $OutDir | Out-Null
$bytes = New-Object byte[] 24
[System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
$password = [Convert]::ToBase64String($bytes)
$pfx = Join-Path $OutDir 'clipiccolo-codesign.pfx'
Export-PfxCertificate -Cert $cert -FilePath $pfx -Password (ConvertTo-SecureString $password -AsPlainText -Force) | Out-Null
[IO.File]::WriteAllText((Join-Path $OutDir 'pfx-password.txt'), $password)
[IO.File]::WriteAllText((Join-Path $OutDir 'pfx-base64.txt'), [Convert]::ToBase64String([IO.File]::ReadAllBytes($pfx)))

$cer = Join-Path $PSScriptRoot 'clipiccolo-codesign.cer'
Export-Certificate -Cert $cert -FilePath $cer | Out-Null

Write-Host "Created $subject, valid until $($cert.NotAfter.ToString('yyyy-MM-dd'))"
Write-Host "  Thumbprint  $($cert.Thumbprint)"
Write-Host "  Public cert $cer (commit this)"
Write-Host "  Private key $pfx (keep private)"
Write-Host ''
Write-Host 'For the release workflow, add two repository secrets:'
Write-Host "  Get-Content -Raw `"$OutDir\pfx-base64.txt`"   | gh secret set SIGNING_PFX_BASE64"
Write-Host "  Get-Content -Raw `"$OutDir\pfx-password.txt`" | gh secret set SIGNING_PFX_PASSWORD"
