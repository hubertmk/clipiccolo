"""Clipiccolo brand assets from one geometry source ("Due punti").

    python brand/build.py

Writes the SVG masters in brand/, the PNG exports in brand/png/, and the app icons in
src-tauri/icons/ (PNGs + a multi-size icon.ico whose 16-24 px entries use the simplified mark, icon.icns on
Apple's icon grid for macOS, and the DMG background). Needs Pillow and fontTools (the wordmark is outlined
from Bricolage Grotesque Bold, brand/src/).
"""
import io
import math
import os
import shutil
import struct
import subprocess

from fontTools.pens.basePen import BasePen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from PIL import Image, ImageChops, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ICONS = os.path.join(ROOT, "src-tauri", "icons")
PNG = os.path.join(HERE, "png")

# ---- Palette ------------------------------------------------------------------------------
ESPRESSO = "#2A1F18"   # tile, ink
CREMA = "#F6EFE3"      # the big dot on the tile, ink on dark
PISTACHIO = "#8DB443"  # the small dot (decorative; text uses #4F6E1E)

# ---- Mark geometry (1024 grid, tile inset 64, radius 208 = Dettino's tile) -----------------
TILE = (64, 64, 960, 960, 208)
FULL = {  # >= 32 px: big dot holding a play triangle, small dot on the same baseline
    "big": (405, 524, 196),
    "tri": [(371, 456), (483, 524), (371, 592)],  # clockwise on screen
    "tri_round": 15,
    "small": (735, 640, 80),
}
SMALL = {  # 16-24 px: two dots only, larger, same baseline
    "big": (400, 520, 236),
    "tri": None,
    "small": (752, 644, 112),
}


def rounded_poly(verts, r, steps=24):
    """Outline of a convex clockwise polygon grown by r with round joins, as points."""
    pts = []
    n = len(verts)
    for i in range(n):
        p0, p1, p2 = verts[i - 1], verts[i], verts[(i + 1) % n]
        # (dy, -dx) is the outward normal of an edge when the order is clockwise on screen
        nx_in, ny_in = (p1[1] - p0[1]), -(p1[0] - p0[0])
        nx_out, ny_out = (p2[1] - p1[1]), -(p2[0] - p1[0])
        a_in, a_out = math.atan2(ny_in, nx_in), math.atan2(ny_out, nx_out)
        while a_out < a_in:
            a_out += 2 * math.pi
        for k in range(steps + 1):
            a = a_in + (a_out - a_in) * k / steps
            pts.append((p1[0] + r * math.cos(a), p1[1] + r * math.sin(a)))
    return pts


def tri_path(verts, r):
    """SVG path for the rounded triangle (lines + arcs)."""
    n = len(verts)
    segs = []
    for i in range(n):
        p0, p1, p2 = verts[i - 1], verts[i], verts[(i + 1) % n]
        def nrm(a, b):
            nx, ny = (b[1] - a[1]), -(b[0] - a[0])
            l = math.hypot(nx, ny)
            return nx / l, ny / l
        n_in, n_out = nrm(p0, p1), nrm(p1, p2)
        a = (p1[0] + r * n_in[0], p1[1] + r * n_in[1])
        b = (p1[0] + r * n_out[0], p1[1] + r * n_out[1])
        segs.append((a, b))
    d = f"M{segs[-1][1][0]:.1f} {segs[-1][1][1]:.1f}"
    for a, b in segs:
        d += f"L{a[0]:.1f} {a[1]:.1f}A{r} {r} 0 0 1 {b[0]:.1f} {b[1]:.1f}"
    return d + "Z"


def circle_path(cx, cy, r, ccw=False):
    s = 0 if ccw else 1
    return f"M{cx - r} {cy}A{r} {r} 0 1 {s} {cx + r} {cy}A{r} {r} 0 1 {s} {cx - r} {cy}Z"


# ---- SVG marks ----------------------------------------------------------------------------
def svg_tile(g, title):
    x0, y0, x1, y1, rx = TILE
    cx, cy, r = g["big"]
    sx, sy, sr = g["small"]
    tri = (f'\n  <path fill="{ESPRESSO}" d="{tri_path(g["tri"], g["tri_round"])}"/>' if g["tri"] else "")
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">\n'
            f'  <title>{title}</title>\n'
            f'  <rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" rx="{rx}" fill="{ESPRESSO}"/>\n'
            f'  <circle cx="{cx}" cy="{cy}" r="{r}" fill="{CREMA}"/>{tri}\n'
            f'  <circle cx="{sx}" cy="{sy}" r="{sr}" fill="{PISTACHIO}"/>\n</svg>\n')


def bare_paths(g=FULL):
    """Big dot with the triangle knocked out (even-odd), and the small dot."""
    cx, cy, r = g["big"]
    big = circle_path(cx, cy, r) + tri_path(g["tri"], g["tri_round"])
    return big, g["small"]


def bare_viewbox(g=FULL, pad=24):
    cx, cy, r = g["big"]
    sx, sy, sr = g["small"]
    x0, y0 = cx - r - pad, cy - r - pad
    return x0, y0, (sx + sr + pad) - x0, (cy + r + pad) - y0


def svg_bare(ink, title):
    big, (sx, sy, sr) = bare_paths()
    x0, y0, w, h = bare_viewbox()
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {y0} {w} {h}" width="{w}" height="{h}">\n'
            f'  <title>{title}</title>\n'
            f'  <path fill="{ink}" fill-rule="evenodd" d="{big}"/>\n'
            f'  <circle cx="{sx}" cy="{sy}" r="{sr}" fill="{PISTACHIO}"/>\n</svg>\n')


# ---- Wordmark: "clipiccolo", outlined, second i-dot smaller and pistachio --------------------
TRACK = -22          # font units per glyph (upm 1000), tight like the siblings' headlines
DOT2_SCALE = 0.66    # the piccolo dot


class FlatPen(BasePen):
    """Collects glyph outlines as flattened polygons (for rasterizing the wordmark)."""

    def __init__(self, glyphSet=None):
        super().__init__(glyphSet)
        self.contours, self._cur = [], []

    def _moveTo(self, p):
        self._cur = [p]

    def _lineTo(self, p):
        self._cur.append(p)

    def _qCurveToOne(self, p1, p2):
        p0 = self._cur[-1]
        for k in range(1, 9):
            t = k / 8
            self._cur.append(((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
                              (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1]))

    def _curveToOne(self, p1, p2, p3):
        p0 = self._cur[-1]
        for k in range(1, 11):
            t = k / 10
            a, b, c, d = (1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t * t, t ** 3
            self._cur.append((a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
                              a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1]))

    def _closePath(self):
        if len(self._cur) > 2:
            self.contours.append(self._cur)
        self._cur = []

    _endPath = _closePath


def wordmark(flat=None):
    """Outlined wordmark. Returns (svg path d, dots, width, top, bottom); also feeds `flat` (a FlatPen) if given."""
    font = TTFont(os.path.join(HERE, "src", "bricolage-700.woff"))
    gs, cmap = font.getGlyphSet(), font.getBestCmap()

    # the i's dot is its upper contour: measure it, then draw i without it
    rec = DecomposingRecordingPen(gs)
    gs[cmap[ord("i")]].draw(rec)
    contours, cur = [], []
    for op, args in rec.value:
        cur.append((op, args))
        if op in ("closePath", "endPath"):
            contours.append(cur)
            cur = []
    def bounds(c):
        bp = BoundsPen(gs)
        for op, args in c:
            getattr(bp, op)(*args)
        return bp.bounds
    contours.sort(key=lambda c: bounds(c)[1])
    stem, dot = contours[0], contours[-1]
    dx0, dy0, dx1, dy1 = bounds(dot)
    dot_d = dx1 - dx0
    dot_cx_in_glyph = (dx0 + dx1) / 2
    dot_bottom = dy0

    pen = SVGPathPen(gs)
    x = 0
    dots = []
    i_seen = 0
    for ch in "clipiccolo":
        name = cmap[ord(ch)]
        for target in (pen, flat) if flat is not None else (pen,):
            t = TransformPen(target, (1, 0, 0, -1, x, 0))
            if ch == "i":
                for op, args in stem:
                    getattr(t, op)(*args)
            else:
                gs[name].draw(t)
        if ch == "i":
            i_seen += 1
            d = dot_d if i_seen == 1 else dot_d * DOT2_SCALE
            dots.append((x + dot_cx_in_glyph, -(dot_bottom + d / 2), d / 2, i_seen == 2))
        x += gs[name].width + TRACK
    width = x - TRACK
    top = -(dot_bottom + dot_d) - 10
    bottom = 146 + 10  # p descender
    return pen.getCommands(), dots, width, top, bottom


def svg_wordmark(ink, title):
    d, dots, w, top, bottom = wordmark()
    circles = "".join(
        f'\n  <circle cx="{cx:.1f}" cy="{cy:.1f}" r="{r:.1f}" fill="{PISTACHIO if small else ink}"/>'
        for cx, cy, r, small in dots)
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-10 {top:.0f} {w + 20:.0f} {bottom - top:.0f}" '
            f'width="{w + 20:.0f}" height="{bottom - top:.0f}">\n  <title>{title}</title>\n'
            f'  <path fill="{ink}" d="{d}"/>{circles}\n</svg>\n')


def svg_lockup(ink, title):
    """Tile + wordmark. The tile spans the wordmark's ascender to just below the baseline."""
    d, dots, w, top, bottom = wordmark()
    tile_h = 860
    tile_y = -730
    scale = tile_h / 1024
    gap = 230
    tx = tile_h + gap
    x0, y0, x1, y1, rx = TILE
    cx, cy, r = FULL["big"]
    sx, sy, sr = FULL["small"]
    circles = "".join(
        f'<circle cx="{cx2 + tx:.1f}" cy="{cy2:.1f}" r="{r2:.1f}" fill="{PISTACHIO if small else ink}"/>'
        for cx2, cy2, r2, small in dots)
    total_w = tx + w
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 {tile_y - 20} {total_w + 20:.0f} {tile_h + 40}" '
            f'width="{total_w + 20:.0f}" height="{tile_h + 40}">\n  <title>{title}</title>\n'
            f'  <g transform="translate(0 {tile_y}) scale({scale:.5f}) translate(-{x0} -{y0})">'
            f'<rect x="{x0}" y="{y0}" width="{x1 - x0}" height="{y1 - y0}" rx="{rx}" fill="{ESPRESSO}"/>'
            f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{CREMA}"/>'
            f'<path fill="{ESPRESSO}" d="{tri_path(FULL["tri"], FULL["tri_round"])}"/>'
            f'<circle cx="{sx}" cy="{sy}" r="{sr}" fill="{PISTACHIO}"/></g>\n'
            f'  <g transform="translate({tx} 0)"><path fill="{ink}" d="{d}"/></g>{circles}\n</svg>\n')


# ---- Raster -------------------------------------------------------------------------------
def render_tile(size, g):
    ss = max(8, 2048 // size) if size < 2048 else 1
    S = size * ss
    k = S / 1024
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    dr = ImageDraw.Draw(im)
    x0, y0, x1, y1, rx = TILE
    dr.rounded_rectangle([x0 * k, y0 * k, x1 * k, y1 * k], radius=rx * k, fill=ESPRESSO)
    cx, cy, r = g["big"]
    dr.ellipse([(cx - r) * k, (cy - r) * k, (cx + r) * k, (cy + r) * k], fill=CREMA)
    if g["tri"]:
        dr.polygon([(px * k, py * k) for px, py in rounded_poly(g["tri"], g["tri_round"])], fill=ESPRESSO)
    sx, sy, sr = g["small"]
    dr.ellipse([(sx - sr) * k, (sy - sr) * k, (sx + sr) * k, (sy + sr) * k], fill=PISTACHIO)
    return im.resize((size, size), Image.LANCZOS) if ss > 1 else im


def png_bytes(im):
    b = io.BytesIO()
    im.save(b, "PNG", optimize=True)
    return b.getvalue()


def write_ico(path, sizes):
    imgs = [(s, png_bytes(render_tile(s, SMALL if s <= 24 else FULL))) for s in sizes]
    head = struct.pack("<HHH", 0, 1, len(imgs))
    off = 6 + 16 * len(imgs)
    entries, data = b"", b""
    for s, b in imgs:
        entries += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(b), off + len(data))
        data += b
    with open(path, "wb") as f:
        f.write(head + entries + data)


def render_mac_icon(size):
    """macOS app icon: the tile on Apple's grid (824 of 1024, so 100 px of air around it), transparent outside."""
    inner = max(1, round(size * 824 / 896))  # render_tile draws the tile 896/1024 wide; here it should be 824/1024
    tile = render_tile(inner, SMALL if size <= 16 else FULL)
    im = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    im.paste(tile, ((size - inner) // 2, (size - inner) // 2), tile)
    return im


def write_icns(path):
    """Apple icon file with PNG entries for every size macOS asks for (16 to 1024, 1x and 2x)."""
    types = (("icp4", 16), ("ic11", 32), ("icp5", 32), ("ic12", 64), ("icp6", 64), ("ic07", 128), ("ic13", 256),
             ("ic08", 256), ("ic14", 512), ("ic09", 512), ("ic10", 1024))
    pngs = {}
    body = b""
    for kind, size in types:
        data = pngs.setdefault(size, png_bytes(render_mac_icon(size)))
        body += kind.encode("ascii") + struct.pack(">I", 8 + len(data)) + data
    with open(path, "wb") as f:
        f.write(b"icns" + struct.pack(">I", 8 + len(body)) + body)


def render_wordmark(height, ink, ss=4):
    """The outlined wordmark as an RGBA image `height` px tall (descender included), on transparent."""
    flat = FlatPen()
    _, dots, w, top, bottom = wordmark(flat)
    k = height * ss / (bottom - top)
    W, H = int((w + 20) * k), height * ss
    def xy(p):
        return ((p[0] + 10) * k, (p[1] - top) * k)
    acc = Image.new("1", (W, H), 0)
    for c in flat.contours:  # even-odd fill: XOR each contour in, so counters stay open
        m = Image.new("1", (W, H), 0)
        ImageDraw.Draw(m).polygon([xy(p) for p in c], fill=1)
        acc = ImageChops.logical_xor(acc, m)
    im = Image.new("RGBA", (W, H), ink)
    im.putalpha(acc.convert("L"))
    dr = ImageDraw.Draw(im)
    for cx, cy, r, small in dots:
        X, Y = xy((cx, cy))
        dr.ellipse([X - r * k, Y - r * k, X + r * k, Y + r * k], fill=PISTACHIO if small else ink)
    return im.resize((W // ss, height), Image.LANCZOS)


def draw_tracked(dr, xy, text, font, fill, track):
    """Draws text with letter-spacing (`track` in px); positions come from prefix widths, so kerning survives."""
    x, y = xy
    for i, ch in enumerate(text):
        dr.text((x + font.getlength(text[:i]) + i * track, y), ch, font=font, fill=fill)
    return x + font.getlength(text) + (len(text) - 1) * track


DESK, MUTED = "#F3EDE2", "#6E6157"
FONT_BOLD = os.path.join(HERE, "src", "bricolage-700.woff")
FONT_MONO = os.path.join(HERE, "src", "dm-mono-400.woff")


def headline(dr, x, y, size, lines, ink=ESPRESSO, lead=1.04):
    """Two-line display headline at -0.035em with the pistachio dot as its final period. Returns the last baseline."""
    f = ImageFont.truetype(FONT_BOLD, size)
    asc = f.getmetrics()[0]
    end = x
    for i, line in enumerate(lines):
        end = draw_tracked(dr, (x, y + i * size * lead), line, f, ink, -0.035 * size)
    base = y + (len(lines) - 1) * size * lead + asc
    r = size * 0.085
    dr.ellipse([end + size * 0.03, base - 2 * r, end + size * 0.03 + 2 * r, base], fill=PISTACHIO)
    return base


def installer_sidebar():
    """NSIS welcome/finish sidebar, 164 x 314: crema desk, tile, wordmark, the headline at the foot."""
    ss = 3
    im = Image.new("RGB", (164 * ss, 314 * ss), DESK)
    tile = render_tile(64 * ss, FULL)
    im.paste(tile, (20 * ss, 36 * ss), tile)
    wm = render_wordmark(26 * ss, ESPRESSO)
    im.paste(wm, (18 * ss, 116 * ss), wm)
    headline(ImageDraw.Draw(im), 22 * ss, 238 * ss, 19 * ss, ["Big clip in.", "Piccolo clip out"])
    return im.resize((164, 314), Image.LANCZOS)


def installer_header():
    """NSIS page header, 150 x 57, on the installer's white header band: the tile at the right."""
    im = Image.new("RGB", (150, 57), "#FFFFFF")
    tile = render_tile(40, FULL)
    im.paste(tile, (150 - 40 - 10, 8), tile)
    return im


def dmg_background(scale):
    """The macOS disk image window, 660 x 400: lockup at the top, the arrow between the app and Applications
    (icon centres at x 180 and 480, y 190 in tauri.macos.conf.json), one line of help below."""
    W, H = 660 * scale, 400 * scale
    im = Image.new("RGB", (W, H), DESK)
    dr = ImageDraw.Draw(im)
    tile = render_tile(28 * scale, FULL)
    im.paste(tile, (32 * scale, 30 * scale), tile)
    wm = render_wordmark(19 * scale, ESPRESSO)
    im.paste(wm, (68 * scale, 33 * scale), wm)
    y = 190 * scale
    x0, x1 = 268 * scale, 392 * scale
    dr.line([(x0, y), (x1, y)], fill=MUTED, width=max(1, int(1.5 * scale)))
    a = 9 * scale
    dr.line([(x1 - a, y - a), (x1, y), (x1 - a, y + a)], fill=MUTED, width=max(1, int(1.5 * scale)), joint="curve")
    mono = ImageFont.truetype(FONT_MONO, 13 * scale)
    text = "Drag Clipiccolo into Applications, then open it from there"
    tw = dr.textlength(text, font=mono)
    dr.text(((W - tw) / 2, 300 * scale), text, font=mono, fill=MUTED)
    return im


def social_card():
    """1280 x 640 link preview (site og:image, GitHub social preview): lockup, headline, the two dots on its baseline."""
    ss = 2
    W, H = 1280 * ss, 640 * ss
    im = Image.new("RGB", (W, H), DESK)
    dr = ImageDraw.Draw(im)
    x = 96 * ss
    tile = render_tile(56 * ss, FULL)
    im.paste(tile, (x, 76 * ss), tile)
    wm = render_wordmark(40 * ss, ESPRESSO)
    im.paste(wm, (x + 72 * ss, 80 * ss), wm)
    base = headline(dr, x - 4 * ss, 228 * ss, 104 * ss, ["Big clip in.", "Piccolo clip out"])
    mono = ImageFont.truetype(FONT_MONO, 26 * ss)
    dr.text((x, base + 56 * ss), "Free · Windows 10/11 · open source", font=mono, fill=MUTED)
    # the mark's two dots, large, sitting on the headline's last baseline (the Shared Baseline idea)
    (bcx, bcy, br), (scx, scy, sr) = FULL["big"], FULL["small"]
    k = 132 * ss / br                    # big dot: 132 px radius on the 1280 card
    ox = W - 96 * ss - (scx + sr) * k    # small dot's right edge on the right margin
    oy = base - (bcy + br) * k           # both dots sit on the headline's last baseline
    big = [((px) * k + ox, (py) * k + oy) for px, py in rounded_poly(FULL["tri"], FULL["tri_round"])]
    dr.ellipse([(bcx - br) * k + ox, (bcy - br) * k + oy, (bcx + br) * k + ox, (bcy + br) * k + oy], fill=ESPRESSO)
    dr.polygon(big, fill=DESK)
    dr.ellipse([(scx - sr) * k + ox, (scy - sr) * k + oy, (scx + sr) * k + ox, (scy + sr) * k + oy], fill=PISTACHIO)
    return im.resize((1280, 640), Image.LANCZOS)


def main():
    os.makedirs(PNG, exist_ok=True)
    out = {
        "mark.svg": svg_tile(FULL, "Clipiccolo"),
        "mark-small.svg": svg_tile(SMALL, "Clipiccolo (16-24 px)"),
        "mark-bare.svg": svg_bare(ESPRESSO, "Clipiccolo"),
        "mark-bare-dark.svg": svg_bare(CREMA, "Clipiccolo"),
        "wordmark.svg": svg_wordmark(ESPRESSO, "clipiccolo"),
        "wordmark-dark.svg": svg_wordmark(CREMA, "clipiccolo"),
        "lockup.svg": svg_lockup(ESPRESSO, "Clipiccolo"),
        "lockup-dark.svg": svg_lockup(CREMA, "Clipiccolo"),
    }
    for name, svg in out.items():
        with open(os.path.join(HERE, name), "w", encoding="utf-8", newline="\n") as f:
            f.write(svg)

    for s in (16, 24, 32, 48, 64, 128, 180, 256, 512, 1024):
        render_tile(s, SMALL if s <= 24 else FULL).save(os.path.join(PNG, f"mark-{s}.png"), optimize=True)

    for name, s in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512)):
        render_tile(s, FULL).save(os.path.join(ICONS, name), optimize=True)
    write_ico(os.path.join(ICONS, "icon.ico"), (16, 20, 24, 32, 40, 48, 64, 128, 256))

    installer_sidebar().save(os.path.join(ICONS, "installer-sidebar.bmp"))
    installer_header().save(os.path.join(ICONS, "installer-header.bmp"))

    write_icns(os.path.join(ICONS, "icon.icns"))
    # DMG background: 1x and 2x PNGs, folded into one HiDPI TIFF where macOS' tiffutil is available.
    bg1, bg2 = os.path.join(PNG, "dmg-background.png"), os.path.join(PNG, "dmg-background@2x.png")
    dmg_background(1).save(bg1, optimize=True)
    dmg_background(2).save(bg2, optimize=True)
    if shutil.which("tiffutil"):
        subprocess.run(["tiffutil", "-cathidpicheck", bg1, bg2, "-out", os.path.join(ICONS, "dmg-background.tiff")], check=True)
    else:
        print("note: tiffutil not found (not macOS); src-tauri/icons/dmg-background.tiff left as is")
    render_wordmark(160, ESPRESSO).save(os.path.join(PNG, "wordmark-160.png"), optimize=True)
    social = social_card()
    social.save(os.path.join(PNG, "social.png"), optimize=True)
    site = os.path.join(ROOT, "site")
    if os.path.isdir(site):
        social.save(os.path.join(site, "og.png"), optimize=True)
    print("wrote", ", ".join(out), "+ png/ (incl. social.png, dmg-background) + src-tauri/icons/ (incl. installer BMPs, icon.icns, dmg-background.tiff) + site/og.png")


if __name__ == "__main__":
    main()
