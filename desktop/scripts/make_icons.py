#!/usr/bin/env python3
"""Generate the Aether GUI icon set.

The mark is a stylised "Æ" on the same teal-to-emerald gradient the connected
state uses, so the tray icon and the in-app logo read as the same object.

Run from the project root:

    python3 scripts/make_icons.py

Writes every size Tauri's bundler expects into src-tauri/icons/, including the
multi-resolution .ico used by the executable and the tray.

Small sizes get a lighter weight and are drawn at the target resolution rather
than downscaled from 512px. A heavy Æ shrunk to 16px closes its counters and
turns into a white blob; rendering per-size keeps the apertures open.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ICONS = Path(__file__).resolve().parent.parent / "src-tauri" / "icons"

# Matches .grad-live in styles.css.
GRAD_FROM = (6, 182, 212)
GRAD_TO = (16, 185, 129)
GLYPH = (255, 255, 255)

# Sizes Tauri's bundler looks for.
PNG_SIZES = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
}

# Every size Windows may ask for, from the tray up to Explorer's largest tile.
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

# Below this, the glyph is set lighter so its counters survive.
SMALL_THRESHOLD = 48

# At or below this, the Æ ligature stops working: it packs three counters into the
# width of one letter, and at 16-24px they close no matter how the weight is tuned.
# A bare A keeps a single open counter and stays identifiable, which is the whole
# job of a tray icon.
TINY_THRESHOLD = 24

FONT_CANDIDATES_BOLD = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    "C:/Windows/Fonts/segoeuib.ttf",
]


def lerp(a: int, b: int, t: float) -> int:
    return round(a + (b - a) * t)


def rounded_mask(size: int, radius_ratio: float = 0.22) -> Image.Image:
    """Rounded-square mask, matching Windows 11's app tiles.

    Supersampled and downscaled, because PIL's rounded_rectangle aliases badly on
    the arcs at icon sizes. Tiny sizes get a proportionally smaller radius: 22% of
    16px is a 3.5px arc that eats the corners and reads as a blurred blob.
    """
    if size <= TINY_THRESHOLD:
        radius_ratio = 0.16

    scale = 8 if size <= 64 else 4
    big = size * scale

    mask = Image.new("L", (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (0, 0, big - 1, big - 1),
        radius=int(big * radius_ratio),
        fill=255,
    )

    return mask.resize((size, size), Image.LANCZOS)


def gradient(size: int) -> Image.Image:
    """Diagonal gradient, matching the 135deg CSS gradients in the UI.

    At tray sizes there are too few pixels across the diagonal to resolve a ramp,
    and the light end costs contrast against the white glyph, so tiny icons get a
    flat mid-tone instead.
    """
    if size <= TINY_THRESHOLD:
        mid = tuple(lerp(a, b, 0.5) for a, b in zip(GRAD_FROM, GRAD_TO))
        return Image.new("RGB", (size, size), mid)

    steps = 256
    strip = Image.new("RGB", (steps, 1))
    pixels = strip.load()

    for i in range(steps):
        t = i / (steps - 1)
        pixels[i, 0] = (
            lerp(GRAD_FROM[0], GRAD_TO[0], t),
            lerp(GRAD_FROM[1], GRAD_TO[1], t),
            lerp(GRAD_FROM[2], GRAD_TO[2], t),
        )

    # A diagonal gradient is a horizontal one sampled along x+y, so build a strip
    # and index into it rather than computing every pixel in Python.
    stretched = strip.resize((size * 2, 1), Image.BILINEAR)
    row = stretched.load()

    diagonal = Image.new("RGB", (size, size))
    out = diagonal.load()
    denominator = max(1, (size - 1) * 2)

    for y in range(size):
        for x in range(size):
            out[x, y] = row[round((x + y) / denominator * (size * 2 - 1)), 0]

    return diagonal


def load_font(pixel_size: int, bold: bool = True) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    """First available bold sans on the system.

    `bold` is accepted for readability at call sites but only bold faces are
    listed: light strokes dissolve at tray sizes, which the small entries proved.
    """
    del bold
    for path in FONT_CANDIDATES_BOLD:
        if Path(path).exists():
            return ImageFont.truetype(path, pixel_size)
    return ImageFont.load_default()


def draw_glyph(image: Image.Image, size: int) -> None:
    """Centre the wordmark on its actual drawn ink.

    Font metrics are not a reliable guide: `textbbox` includes ascent padding that
    sits well above the cap height, and the amount varies per font. Drawing first
    and measuring the pixels actually covered is the only approach that centres
    correctly whichever font the machine happens to have.

    Bold is used at every size. A light weight was tried for the small entries on
    the theory that thinner strokes leave wider counters, but the opposite happens
    below 32px: a 1px stroke lands half-way between pixel rows, antialiases to grey,
    and the glyph dissolves. A heavy stroke snaps to whole pixels and stays crisp.
    """
    tiny = size <= TINY_THRESHOLD

    # Æ packs three counters into one letter's width. Verified against the actual
    # pixel grid: it holds at 32px and closes below that, so tray sizes fall back
    # to a bare A, whose single apex counter survives down to 16px.
    text = "A" if tiny else "Æ"

    # A is narrower than Æ, so it can take more of the tile before crowding.
    coverage = 0.70 if tiny else 0.58 if size < SMALL_THRESHOLD else 0.64
    font = load_font(int(size * coverage), bold=True)

    # Draw onto an oversized scratch layer so nothing is clipped before it is
    # measured, then move the ink into place.
    pad = size
    scratch = Image.new("RGBA", (size + pad * 2, size + pad * 2), (0, 0, 0, 0))
    ImageDraw.Draw(scratch).text((pad, pad), text, font=font, fill=GLYPH)

    ink = scratch.getbbox()
    if ink is None:
        return

    glyph = scratch.crop(ink)

    # The Æ's A is a diagonal wedge and its E is three solid bars, so the visual
    # mass sits right of the bounding-box centre; a small leftward nudge corrects
    # for it. A bare A is symmetric and needs none.
    optical_x = 0.0 if tiny else size * 0.014

    x = round((size - glyph.width) / 2 - optical_x)
    # A hair above true centre: capitals read low when placed dead centre.
    y = round((size - glyph.height) / 2 - size * 0.008)

    image.alpha_composite(glyph, (x, y))


def render(size: int) -> Image.Image:
    base = gradient(size).convert("RGBA")
    base.putalpha(rounded_mask(size))

    # Composite the glyph over the masked tile so the rounded alpha is preserved.
    draw_glyph(base, size)
    return base


def main() -> None:
    ICONS.mkdir(parents=True, exist_ok=True)

    for name, size in PNG_SIZES.items():
        render(size).save(ICONS / name, "PNG")
        print(f"wrote {name} ({size}x{size})")

    # Render each .ico entry at its own size rather than letting PIL downscale a
    # single 256px image, so the small entries keep their open counters.
    frames = [render(size) for size in ICO_SIZES]
    frames[-1].save(
        ICONS / "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ICO_SIZES],
        append_images=frames[:-1],
    )
    print(f"wrote icon.ico ({', '.join(f'{s}x{s}' for s in ICO_SIZES)})")


if __name__ == "__main__":
    main()
