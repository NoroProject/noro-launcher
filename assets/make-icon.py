#!/usr/bin/env python3
"""Builds icon.png/icns/ico for the launcher.

With no arguments it draws an ATOM-style placeholder: a cream "N" framed on dark
navy. Once there is a real logo, drop it in as a square PNG and run
`./make-icon.py logo.png` — the other sizes follow from it.
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).parent
SIZE = 1024

# Theme tokens, from crates/frontend/src/theme.rs.
BG = (0x0D, 0x1B, 0x2E, 255)  # BG_WINDOW
CTA = (0xF3, 0xE7, 0xB3, 255)  # cream CTA
ACCENT = (0xE8, 0x5A, 0xA5, 255)  # crystal accent

# 5x5 pixel "N", following the Monocraft face used in the UI.
GLYPH = [
    "X...X",
    "XX..X",
    "X.X.X",
    "X..XX",
    "X...X",
]


def placeholder() -> Image.Image:
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # macOS doesn't round the icon itself and a square one stands out in the
    # dock. 224 of 1024 is close to the system icons.
    draw.rounded_rectangle([0, 0, SIZE - 1, SIZE - 1], radius=224, fill=BG)
    draw.rounded_rectangle(
        [96, 96, SIZE - 97, SIZE - 97], radius=96, outline=CTA, width=12
    )

    cell = 88
    glyph_w = cell * len(GLYPH[0])
    glyph_h = cell * len(GLYPH)
    left = (SIZE - glyph_w) // 2
    top = (SIZE - glyph_h) // 2
    for row, line in enumerate(GLYPH):
        for col, ch in enumerate(line):
            if ch != "X":
                continue
            x = left + col * cell
            y = top + row * cell
            # The diagonal takes the accent colour.
            color = ACCENT if 0 < row < 4 and col == row else CTA
            draw.rectangle([x, y, x + cell - 1, y + cell - 1], fill=color)

    return img


def build(source: Image.Image) -> None:
    source.save(HERE / "icon.png")

    # Windows picks a suitable size out of the ones embedded here.
    source.save(
        HERE / "icon.ico",
        sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)],
    )

    # iconutil builds .icns from an .iconset directory with exact filenames.
    iconset = HERE / "icon.iconset"
    iconset.mkdir(exist_ok=True)
    for base in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            px = base * scale
            suffix = "@2x" if scale == 2 else ""
            resized = source.resize((px, px), Image.LANCZOS)
            resized.save(iconset / f"icon_{base}x{base}{suffix}.png")

    subprocess.run(
        ["iconutil", "--convert", "icns", str(iconset), "--output", str(HERE / "icon.icns")],
        check=True,
    )
    for leftover in iconset.iterdir():
        leftover.unlink()
    iconset.rmdir()


if __name__ == "__main__":
    if len(sys.argv) > 1:
        image = Image.open(sys.argv[1]).convert("RGBA")
        if image.width != image.height:
            sys.exit(f"need a square PNG, got {image.width}x{image.height}")
        image = image.resize((SIZE, SIZE), Image.LANCZOS)
    else:
        image = placeholder()
    build(image)
    print("built: icon.png, icon.icns, icon.ico")
