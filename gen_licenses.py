# Regenerates THIRD-PARTY-LICENSES.html (license texts of every Rust crate shipped in the app and the launcher).
# Needs cargo-about:  cargo install cargo-about --locked --features cli
# Run after any dependency change:  python gen_licenses.py
import os, subprocess, tempfile

here = os.path.dirname(os.path.abspath(__file__))
out = os.path.join(here, "THIRD-PARTY-LICENSES.html")


def generate(manifest):
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "licenses.html")
        subprocess.run(
            ["cargo", "about", "generate", "-c", "about.toml", "-m", manifest, "about.hbs", "-o", path],
            cwd=here, check=True,
        )
        return open(path, encoding="utf-8").read()


app = generate("src-tauri/Cargo.toml")
launcher = generate("launcher/Cargo.toml")
# Keep the app's page and append the launcher's license sections under their own heading.
marker = "<h2>License texts</h2>"
launcher_sections = launcher.split(marker, 1)[1].rsplit("</body>", 1)[0]
html = app.replace("</body>", f"<h2>Portable launcher (Clipiccolo.exe)</h2>{launcher_sections}</body>", 1)
open(out, "w", encoding="utf-8", newline="\n").write(html)
print(f"Wrote {out}")
