# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Anyone on Windows or a Mac who has a video that is too big to send and wants a smaller one without learning a video tool. The repo's examples are work clips (sprint demos, bug repros, phone recordings pasted into Teams, Jira or email), but the confirmed audience is the general public and any video use, not only office work and not video professionals.

Situation: the person has the file (or a link to it) at hand and is about to share it. The job is "make this small enough to paste, now", with no decisions beyond one quality choice.

## Product Purpose

Clipiccolo turns any video into a small H.264 MP4 that plays everywhere and lands on the clipboard, ready to paste. Success is a round trip measured in seconds: copy or drop a video, switch to Clipiccolo, paste the small version wherever it was going.

It is free and open source (MIT), a private product of the author under HuKe GmbH, and a sibling of Dettino, Kiesli and VoxFluo.

## Positioning

The clipboard is the product, in both directions. Clipiccolo converts what is on the clipboard when you switch to it, and puts the finished small video back on the clipboard when the batch is done. Other converters produce a file you then have to find; Clipiccolo produces a paste. This mechanism is confirmed as the core and must be protected on every surface.

Supporting claims that are true today: one window, three presets, no account, no admin rights, no upload (everything runs locally with a bundled ffmpeg), never upscales, keeps portrait videos portrait.

## Operating Context

- Windows 10/11 desktop. Shipped as a per-user NSIS installer (about 28 MB, installs WebView2 if missing) and as a single portable exe that unpacks on first start.
- macOS 11+ on Apple Silicon. Shipped as a signed (and, with credentials, notarized) `Clipiccolo.app` on a disk image, ffmpeg inside; native title bar with traffic lights, otherwise the same window. No Intel build.
- Inputs: files dropped on the window, the exe or the Dock icon, Browse, Ctrl+V / ⌘V of files, "Copy as path" text, or http(s) links to a video (downloaded with the system's curl; HLS `.m3u8` goes straight to ffmpeg).
- Outputs: `<name> - piccolo.mp4` next to the original, or in a folder chosen in Settings. Finished videos go to the clipboard (when the setting is on, default on) with a toast; Copy, Copy all, Ctrl+C / ⌘C and drag-out also work. Show selects the file in Explorer or Finder.
- Presets: Small (≤1280 px, ≤30 fps, CRF 30, AAC 64 kb/s), Balanced (≤1920 px, ≤30 fps, CRF 26, AAC 96 kb/s, default), Sharp (≤2560 px, ≤60 fps, CRF 22, AAC 128 kb/s). Several files queue and run one after another; Cancel kills the current ffmpeg.
- Settings live in `%APPDATA%\io.github.hubertmk.clipiccolo\settings.json` (Mac: `~/Library/Application Support/io.github.hubertmk.clipiccolo/settings.json`).
- Downloads come from the GitHub releases page of `hubertmk/clipiccolo`; the landing page in `site/` links there.

## Capabilities and Constraints

- Stack: Tauri 2 (Rust) with a plain HTML/CSS/JS UI in `ui/`, tokens from `brand/tokens.css`, fonts bundled in `ui/fonts/`. Window is 380×520 at minimum 340×440, undecorated (custom title bar), not maximizable. CSP allows only self, inline styles and data: images.
- Bundled ffmpeg (Windows: gyan.dev 8.0 essentials; Mac: Martin Riedl's arm64 build; both GPL v3, run as a separate process) handles any input format ffmpeg reads. Output is always H.264 MP4 with faststart.
- Windows builds are not code-signed: SmartScreen warns on first run of the installer and the portable exe. Future copy must not claim otherwise. The Mac app is signed with the author's Developer ID (and notarized when the release is built with Apple credentials).
- Terminology used in the product: Drop / paste, Browse, Small / Balanced / Sharp, Copy / Copy all, Show, Clear list, Cancel, "Convert copied videos", "Copy converted videos", "Save converted videos".
- Decided 2026-11-13: Clipiccolo ships for Windows and for macOS on Apple Silicon, with the same features on both (clipboard in and out, drag-out, links, Show). Linux and Intel Macs are not planned.
- Undecided: whether the bundle identifier, settings path and output naming are frozen. The user did not mark them as fixed; treat them as current facts, and ask before any surface or feature depends on changing them.

## Brand Commitments

- Name: Clipiccolo (formerly Clip Shrinker), wordmark lowercase "clipiccolo". The name carries the idea: a big clip becomes a piccolo one.
- Identity "Due punti", chosen 2026-11-13 by the user and already shipped in the app, the site and `brand/`: espresso and crema with a pistachio accent, mark, wordmark and lockups as SVG masters plus generated PNGs, tokens in `brand/tokens.css`, Bricolage Grotesque for UI and DM Mono for numbers. Recorded here as a binding constraint, not described further; the design record belongs in DESIGN.md.
- Voice: plain, short, confident English with at most one Italian word per screen (for example the toast "Fatto. Copied to clipboard."). Confirmed as fixed.
- Family: Clipiccolo follows the same quiet frame as its HuKe siblings (lockup, copy left, one product loop right, one-line footer) while owning its own accent.

## Evidence on Hand

- Real product behavior and numbers in `README.md` (preset table, installer size, install path, clipboard behaviors).
- Brand assets in `brand/` (mark, wordmark, lockups, light and dark variants, PNG sizes 16 to 1024) and app icons in `src-tauri/icons/`.
- Landing page `site/index.html` with a scripted product loop using illustrative clip names and sizes; those figures are examples, not measurements.
- Licensing facts: MIT for Clipiccolo, GPL v3 for ffmpeg, SIL OFL 1.1 for the fonts, Lucide icons; `THIRD-PARTY-LICENSES.html` is present.
- Absent, do not fabricate: testimonials, user or download counts, press, benchmarks against other tools, measured compression ratios, Windows code signing, Linux or Intel Mac builds, pricing (it is free).

## Product Principles

1. The paste is the finish line. Every flow ends with the small video on the clipboard or one obvious action away from it.
2. One decision, then nothing. Quality preset is the only choice; everything else has a sensible default that stays out of the way.
3. Honest about the file. Show real names, sizes and savings; never promise quality or size the encoder did not deliver.
4. Local and self-contained. No accounts, no uploads, no prerequisites; the app carries what it needs.
5. A pinch of Italian. Personality comes from the name and one well-placed word, not from decoration or chatter.
