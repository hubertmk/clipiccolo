# Compresses the files the launcher embeds and writes launcher/payload/payload.rs.
# APP_EXE overrides the app binary (default: the Linux cross-build). An MSVC build links WebView2 statically, so
# WebView2Loader.dll is only packed for the -gnu build.
import hashlib, os, zstandard as z
here = os.path.dirname(os.path.abspath(__file__))
out = os.path.join(here, "launcher", "payload")
app = os.environ.get("APP_EXE") or os.path.join(here, "src-tauri/target/x86_64-pc-windows-gnu/release/clipiccolo.exe")
ffmpeg = os.environ["FFMPEG_EXE"]
files = [("ClipiccoloApp.exe", app)]
if not os.environ.get("APP_EXE"):
    reg = os.path.expanduser("~/.cargo/registry/src")
    dll = [os.path.join(r, "x64/WebView2Loader.dll") for r in
           (os.path.join(reg, d, c) for d in os.listdir(reg) for c in os.listdir(os.path.join(reg, d)) if c.startswith("webview2-com-sys-"))][0]
    files.append(("WebView2Loader.dll", dll))
files += [("ffmpeg.exe", ffmpeg),
         ("FFMPEG-LICENSE.txt", os.path.join(here, "src-tauri/resources/FFMPEG-LICENSE.txt")),
         ("LICENSE.txt", os.path.join(here, "LICENSE")),
         ("THIRD-PARTY-NOTICES.md", os.path.join(here, "THIRD-PARTY-NOTICES.md")),
         ("THIRD-PARTY-LICENSES.html", os.path.join(here, "THIRD-PARTY-LICENSES.html"))]
rs = ['pub const APP: &str = "ClipiccoloApp.exe";', "pub static FILES: &[(&str, &[u8], u64)] = &["]
for name, src in files:
    data = open(src, "rb").read()
    zst = name + ".zst"
    cached = os.path.join(out, zst)
    stamp = cached + ".sha256"  # level-19 is slow for ffmpeg: reuse the .zst while the source is unchanged
    digest = hashlib.sha256(data).hexdigest()
    if not (os.path.exists(cached) and os.path.exists(stamp) and open(stamp).read() == digest):
        open(cached, "wb").write(z.ZstdCompressor(level=19, threads=2).compress(data))
        open(stamp, "w").write(digest)
    rs.append(f'    ("{name}", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/payload/{zst}")), {len(data)}),')
    print(f"{name}: {len(data):,} -> {os.path.getsize(cached):,}")
rs.append("];")
open(os.path.join(out, "payload.rs"), "w").write("\n".join(rs) + "\n")
