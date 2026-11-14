# Contributing

Bug reports, ideas and pull requests are welcome.

## Reporting a bug

Open an [issue](https://github.com/hubertmk/clipiccolo/issues) with:
- the Clipiccolo version (Settings, or the installer name) and your Windows or macOS version,
- what you did, what you expected, and what happened,
- the error text shown on the video row, if there is one. For ffmpeg errors, also the input format: container,
  codec and where the video came from.

Please don't attach videos you aren't allowed to share.

## Development setup

You need Windows 10/11 with Rust's MSVC toolchain, or a Mac with Apple Silicon and Xcode's command-line tools, and
the Tauri CLI:

```powershell
cargo install tauri-cli --version "^2" --locked
```

On Windows the app expects `ffmpeg.exe` at `src-tauri\resources\ffmpeg.exe` (git-ignored); get the "essentials"
release from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/). On the Mac it expects a static arm64 `ffmpeg` at
`src-tauri/binaries/ffmpeg-aarch64-apple-darwin` (git-ignored); get it from
[ffmpeg.martin-riedl.de](https://ffmpeg.martin-riedl.de/). For `cargo test` / `cargo check` alone, an empty
placeholder file is enough.

```powershell
cd src-tauri
cargo test            # unit tests (clipboard parsing, links, ffmpeg arguments)
cargo tauri dev       # run the app with the UI from ../ui
```

The UI is plain HTML/CSS/JS in `ui/`: no bundler, no npm. Platform-specific code is small and lives behind `cfg`:
`clip::win` and `clip::mac` in `src-tauri/src/clip.rs`, and `src-tauri/tauri.macos.conf.json` on top of
`tauri.conf.json` for the macOS bundle. A change on one platform should leave the other's behaviour alone; CI builds
and tests both.

## Pull requests

- Keep changes focused; one topic per PR.
- Run `cargo test` and `cargo clippy` in `src-tauri` before pushing. CI runs them on Windows and macOS.
- If you add, remove or upgrade a Rust dependency, run `python gen_licenses.py` (needs
  `cargo install cargo-about --locked --features cli`) and commit the updated `THIRD-PARTY-LICENSES.html`.
- New dependencies must use a license compatible with MIT (see `accepted` in `about.toml`).

By submitting a pull request you agree that your contribution is licensed under the [MIT License](LICENSE).

## Releasing (maintainer)

1. Bump the version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `launcher/Cargo.toml`.
2. If dependencies changed, run `python gen_licenses.py`.
3. Commit, then push a `vX.Y.Z` tag (`git tag v0.6.0 && git push origin v0.6.0`). The Release workflow checks
   that the tag and all three versions match, builds and signs `Clipiccolo-Setup-X.Y.Z.exe` and `Clipiccolo.exe`,
   and opens a **draft** release with them, the public certificate, `SHA256SUMS.txt` and **the FFmpeg source for
   the bundled build** (FFmpeg's GPL v3 requires its source to be offered from the same place as the binary; see
   [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)). The Release (macOS) workflow builds the notarized
   `Clipiccolo-X.Y.Z-mac-arm64.dmg` from the same tag and adds it to that draft (one-time setup:
   [docs/mac-release-github.md](docs/mac-release-github.md)).
4. Check the draft, attach the FFmpeg source of the Mac build too (e.g. `ffmpeg-9.0.2.tar.xz` from
   <https://ffmpeg.org/releases/>; the Windows and Mac builds may be on different versions), write the notes, publish.

Upgrading ffmpeg: on Windows, change `FFMPEG_VERSION` and both checksums in `.github/workflows/release.yml` together;
on the Mac, change `FFMPEG_URL` and `FFMPEG_SHA256` in `.github/workflows/release-mac.yml` and the local build together.
To build a release by hand instead: `.\build-installer.ps1`, `.\build-portable.ps1` (signed if the certificate is
in your store, see README → Code signing) and, on a Mac with the Developer ID certificate and notarization
credentials, `./build-mac.sh`; then attach the same files.
