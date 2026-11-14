# Signs one file with the code-signing certificate whose thumbprint is in $env:CLIPICCOLO_SIGN_THUMBPRINT
# (looked up in Cert:\CurrentUser\My). build-installer.ps1 sets that up and hands this script to Tauri, which calls it
# for every binary it produces; build-portable.ps1 calls it for the portable exe.
#
# This is the one place that knows HOW to sign. Moving to a CA-issued certificate that lives in the store needs no
# change; a cloud signer (e.g. Azure Artifact Signing) would swap the signtool arguments below.
param([Parameter(Mandatory)][string]$File)
$ErrorActionPreference = 'Stop'

$thumb = $env:CLIPICCOLO_SIGN_THUMBPRINT
if (-not $thumb) { throw 'CLIPICCOLO_SIGN_THUMBPRINT is not set.' }

$signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
if (-not $signtool) {
    $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1 -ExpandProperty FullName
}
if (-not $signtool) { throw 'signtool.exe not found. Install the Windows SDK (Signing Tools for Desktop Apps).' }

# Timestamping keeps the signature valid after the certificate expires. Timestamp servers have hiccups: try a few.
$servers = 'http://timestamp.digicert.com', 'http://timestamp.sectigo.com', 'http://timestamp.globalsign.com/tsa/r6advanced1'
foreach ($ts in $servers) {
    & $signtool sign /sha1 $thumb /s My /fd SHA256 /tr $ts /td SHA256 /d Clipiccolo /q $File
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Signed $(Split-Path -Leaf $File)"
        exit 0
    }
    Write-Warning "Signing with timestamp server $ts failed; trying the next one."
}
throw "Could not sign $File"
