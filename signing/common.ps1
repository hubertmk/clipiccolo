# Dot-sourced by the build scripts. Finds the certificate to sign with and exports its thumbprint for sign.ps1.
#
# Order: $env:CLIPICCOLO_SIGN_THUMBPRINT if set (CI sets it after importing the .pfx), otherwise the newest valid
# "CN=Clipiccolo" code-signing certificate with a private key in Cert:\CurrentUser\My (see new-cert.ps1).
# Set $env:CLIPICCOLO_NO_SIGN=1 to build unsigned on purpose.
function Initialize-Signing([switch]$Required) {
    if ($env:CLIPICCOLO_NO_SIGN) {
        if ($Required) { throw 'Signing is required, but CLIPICCOLO_NO_SIGN is set.' }
        Write-Warning 'CLIPICCOLO_NO_SIGN is set: building UNSIGNED.'
        return $null
    }
    $cert = if ($env:CLIPICCOLO_SIGN_THUMBPRINT) {
        Get-Item "Cert:\CurrentUser\My\$($env:CLIPICCOLO_SIGN_THUMBPRINT)" -ErrorAction SilentlyContinue
    } else {
        Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert |
            Where-Object { $_.Subject -eq 'CN=Clipiccolo' -and $_.HasPrivateKey -and $_.NotAfter -gt (Get-Date) } |
            Sort-Object NotAfter | Select-Object -Last 1
    }
    if (-not $cert -or -not $cert.HasPrivateKey) {
        if ($env:CLIPICCOLO_SIGN_THUMBPRINT) { throw "Certificate $($env:CLIPICCOLO_SIGN_THUMBPRINT) (with private key) not found in Cert:\CurrentUser\My." }
        if ($Required) { throw 'Signing is required, but no Clipiccolo code-signing certificate was found. Run signing\new-cert.ps1.' }
        Write-Warning 'No Clipiccolo code-signing certificate found (signing\new-cert.ps1 creates one): building UNSIGNED.'
        return $null
    }
    $env:CLIPICCOLO_SIGN_THUMBPRINT = $cert.Thumbprint
    Write-Host "Signing with $($cert.Subject) ($($cert.Thumbprint), valid until $($cert.NotAfter.ToString('yyyy-MM-dd')))"
    return $cert.Thumbprint
}

function Assert-Signed([string]$File, [string]$Thumbprint) {
    $sig = Get-AuthenticodeSignature $File
    if ($sig.SignerCertificate.Thumbprint -ne $Thumbprint) {
        throw "$File is not signed with $Thumbprint (status: $($sig.Status))."
    }
}
