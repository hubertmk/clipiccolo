# Builds dist\Clipiccolo.exe, the portable single file, on Windows with the MSVC toolchain. Run build-installer.ps1
# first: this packs the app exe and ffmpeg.exe it used. (For the Linux cross-build, see the README.)
#
# Needs Python with zstandard (`pip install zstandard`). Signs the packed app and the launcher like
# build-installer.ps1 does; -RequireSigning makes a missing certificate an error.
param([switch]$RequireSigning)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
. (Join-Path $root 'signing\common.ps1')
$thumb = Initialize-Signing -Required:$RequireSigning

$app = Join-Path $root 'src-tauri\target\x86_64-pc-windows-msvc\release\Clipiccolo.exe'
$ffmpeg = Join-Path $root 'src-tauri\resources\ffmpeg.exe'
if (-not (Test-Path $app)) { throw "$app not found. Run build-installer.ps1 first." }

# Pack a signed copy: Tauri leaves the exe in target\ unsigned (see build-installer.ps1).
$stage = Join-Path $root 'launcher\target\stage'
New-Item -ItemType Directory -Force $stage | Out-Null
$staged = Join-Path $stage 'ClipiccoloApp.exe'
Copy-Item $app $staged -Force
if ($thumb) { & (Join-Path $root 'signing\sign.ps1') $staged }

$env:APP_EXE = $staged
$env:FFMPEG_EXE = $ffmpeg
python (Join-Path $root 'pack_payload.py')
if ($LASTEXITCODE -ne 0) { throw "pack_payload.py failed ($LASTEXITCODE)" }

Push-Location (Join-Path $root 'launcher')
$rustflags = $env:RUSTFLAGS
try {
    # Static C runtime, so the one file runs without the VC++ redistributable (Tauri does the same for the app).
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
} finally {
    $env:RUSTFLAGS = $rustflags
    Pop-Location
}

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$out = Join-Path $dist 'Clipiccolo.exe'
Copy-Item (Join-Path $root 'launcher\target\x86_64-pc-windows-msvc\release\Clipiccolo.exe') $out -Force
if ($thumb) {
    & (Join-Path $root 'signing\sign.ps1') $out
    Assert-Signed $out $thumb
}
Write-Host ("`nPortable: {0} ({1:N1} MB)" -f $out, ((Get-Item $out).Length / 1MB))
