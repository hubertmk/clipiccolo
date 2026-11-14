#!/bin/sh
# Builds dist/Clipiccolo-<version>-mac-arm64.dmg: a signed (and, with Apple credentials, notarized) Apple Silicon
# build of Clipiccolo.app that carries ffmpeg. The Windows installer is build-installer.ps1; this is its macOS twin.
#
# Needs (build machine only): macOS on Apple Silicon with Xcode's command-line tools, Rust, the Tauri CLI
# (`cargo install tauri-cli --version "^2" --locked`), and a static arm64 ffmpeg — set $FFMPEG_BIN, or put it at
# src-tauri/binaries/ffmpeg-aarch64-apple-darwin. Get it from https://ffmpeg.martin-riedl.de/ (macOS, arm64,
# release, ffmpeg.zip; GPL build with libx264, no "nonfree" parts).
#
# Signing: the first "Developer ID Application" certificate in the keychain is used, or $APPLE_SIGNING_IDENTITY.
# Without one the app is signed ad hoc (runs here, not on other Macs).
# Notarization (so Gatekeeper opens it without a detour on other Macs), either
#   APPLE_ID, APPLE_PASSWORD (an app-specific password), APPLE_TEAM_ID          or
#   APPLE_API_ISSUER, APPLE_API_KEY, APPLE_API_KEY_PATH (App Store Connect key)   or
#   APPLE_KEYCHAIN_PROFILE (a profile saved with `xcrun notarytool store-credentials`).
set -eu
root=$(cd "$(dirname "$0")" && pwd)
target=aarch64-apple-darwin
sidecar="$root/src-tauri/binaries/ffmpeg-$target"

mkdir -p "$root/src-tauri/binaries"
if [ -n "${FFMPEG_BIN:-}" ]; then
    cp "$FFMPEG_BIN" "$sidecar"
fi
if [ ! -f "$sidecar" ]; then
    echo "ffmpeg not found. Set FFMPEG_BIN or copy a static arm64 ffmpeg to $sidecar (see the top of this script)." >&2
    exit 1
fi
chmod +x "$sidecar"
if ! file "$sidecar" | grep -q 'arm64'; then
    echo "$sidecar is not an arm64 binary." >&2
    exit 1
fi

if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
    APPLE_SIGNING_IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
        | sed -n 's/.*"\(Developer ID Application: .*\)"/\1/p' | head -n 1)
    if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
        echo "No Developer ID Application certificate in the keychain: signing ad hoc (this Mac only)." >&2
        APPLE_SIGNING_IDENTITY="-"
    fi
fi
export APPLE_SIGNING_IDENTITY
echo "Signing as: $APPLE_SIGNING_IDENTITY"

notarize=tauri
if [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_PASSWORD:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ]; then :
elif [ -n "${APPLE_API_ISSUER:-}" ] && [ -n "${APPLE_API_KEY:-}" ] && [ -n "${APPLE_API_KEY_PATH:-}" ]; then :
elif [ -n "${APPLE_KEYCHAIN_PROFILE:-}" ]; then notarize=profile
else
    notarize=none
    echo "No Apple credentials in the environment: the app will be signed but not notarized (see the top of this script)." >&2
fi

(cd "$root/src-tauri" && cargo tauri build --target "$target")

version=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$root/src-tauri/tauri.conf.json")
bundle="$root/src-tauri/target/$target/release/bundle"
app="$bundle/macos/Clipiccolo.app"
built=$(ls "$bundle"/dmg/Clipiccolo_"$version"_*.dmg | head -n 1)
mkdir -p "$root/dist"
out="$root/dist/Clipiccolo-$version-mac-arm64.dmg"
cp "$built" "$out"

if [ "$notarize" = profile ]; then
    echo "Notarizing with keychain profile $APPLE_KEYCHAIN_PROFILE …"
    xcrun notarytool submit "$out" --keychain-profile "$APPLE_KEYCHAIN_PROFILE" --wait
    xcrun stapler staple "$out"
fi

codesign --verify --deep --strict "$app"
printf '\nApp: %s\nDisk image: %s (%s)\n' "$app" "$out" "$(du -h "$out" | cut -f1)"
