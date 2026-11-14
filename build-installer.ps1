# Builds dist\Clipiccolo-Setup-<version>.exe: a per-user installer (no admin rights) that carries the app
# and ffmpeg, and installs the WebView2 runtime first if the PC doesn't have it yet.
#
# Needs (build machine only): Rust (MSVC toolchain), the Tauri CLI (`cargo install tauri-cli --version "^2"`),
# and ffmpeg.exe — set $env:FFMPEG_EXE, or put it at src-tauri\resources\ffmpeg.exe.
# Get it from https://www.gyan.dev/ffmpeg/builds/ (release "essentials" zip, bin\ffmpeg.exe).
#
# Signing: if a Clipiccolo code-signing certificate is in your certificate store (signing\new-cert.ps1), the app,
# its uninstaller and the installer are signed with it. Otherwise the build is unsigned, with a warning;
# -RequireSigning turns that into an error (the release workflow uses it).
param([switch]$RequireSigning)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$res = Join-Path $root 'src-tauri\resources\ffmpeg.exe'
. (Join-Path $root 'signing\common.ps1')
$thumb = Initialize-Signing -Required:$RequireSigning

if ($env:FFMPEG_EXE) {
    Copy-Item $env:FFMPEG_EXE $res -Force
}
if (-not (Test-Path $res)) {
    throw "ffmpeg.exe not found. Set `$env:FFMPEG_EXE or copy it to $res (see the top of this script)."
}

Push-Location (Join-Path $root 'src-tauri')
try {
    # The .cargo\config.toml there is for the Linux cross-build; it only affects the -gnu target.
    $extra = @()
    if ($thumb) {
        # Tauri runs this for the app exe, the uninstaller and the installer.
        $conf = Join-Path $root 'src-tauri\target\sign.conf.json'
        New-Item -ItemType Directory -Force (Split-Path $conf) | Out-Null
        @{ bundle = @{ windows = @{ signCommand = @{
            cmd  = (Get-Process -Id $PID).Path
            args = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $root 'signing\sign.ps1'), '%1')
        } } } } | ConvertTo-Json -Depth 6 | Set-Content $conf
        $extra = '--config', $conf
    }
    cargo tauri build --target x86_64-pc-windows-msvc @extra
    if ($LASTEXITCODE -ne 0) { throw "cargo tauri build failed ($LASTEXITCODE)" }
} finally {
    Pop-Location
}

$version = (Get-Content (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json).version
$built = Join-Path $root "src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\Clipiccolo_${version}_x64-setup.exe"
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$out = Join-Path $dist "Clipiccolo-Setup-$version.exe"
Copy-Item $built $out -Force
# (Tauri signs a patched copy of the app exe for the installer and then restores target\...\Clipiccolo.exe
# unsigned, so only the installer itself can be checked here.)
if ($thumb) { Assert-Signed $out $thumb }
Write-Host ("`nInstaller: {0} ({1:N1} MB)" -f $out, ((Get-Item $out).Length / 1MB))
