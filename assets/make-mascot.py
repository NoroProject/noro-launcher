#!/usr/bin/env python3
"""Draws the launcher mascot.

The figure is built from primitives on a small canvas and scaled up with nearest
neighbour, which keeps the pixels square and lets the shape be adjusted by
numbers rather than by counting characters in a text map.

Colours come from crates/frontend/src/theme.rs. Output goes to
crates/frontend/assets/, where rust_embed picks it up into the binary.
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).parent.parent / "crates" / "frontend" / "assets"
W, H = 44, 52          # canvas, in cells
SCALE = 8              # scale factor

ROBE_DARK = (0x1B, 0x25, 0x40, 255)
ROBE = (0x2C, 0x3A, 0x5E, 255)
ROBE_LIGHT = (0x3A, 0x4C, 0x78, 255)
FACE = (0x10, 0x17, 0x2B, 255)
HORN = (0xF3, 0xC9, 0x69, 255)
CREAM = (0xF3, 0xE7, 0xB3, 255)
ACCENT = (0xE8, 0x5A, 0xA5, 255)

# The chest mark: same N as the launcher glyph.
MARK = ["X...X", "XX..X", "X.X.X", "X..XX", "X...X"]


def base() -> Image.Image:
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # Horns before the hood, so it covers where they meet the head.
    d.polygon([(13, 2), (16, 3), (15, 11)], fill=HORN)
    d.polygon([(30, 2), (27, 3), (28, 11)], fill=HORN)

    # Robe: shoulders and skirt, widening towards the hem.
    d.polygon([(11, 26), (32, 26), (37, 47), (6, 47)], fill=ROBE)
    d.rectangle([6, 44, 37, 47], fill=ROBE_DARK)
    # Sleeves.
    d.polygon([(11, 28), (6, 30), (7, 41), (12, 39)], fill=ROBE_DARK)
    d.polygon([(32, 28), (37, 30), (36, 41), (31, 39)], fill=ROBE_DARK)

    # Hood.
    d.ellipse([8, 6, 35, 32], fill=ROBE)
    d.ellipse([10, 8, 33, 30], fill=ROBE_LIGHT)
    d.ellipse([11, 9, 32, 29], fill=ROBE)
    # Shadow inside the hood; the eyes sit on it.
    d.ellipse([13, 12, 30, 28], fill=FACE)

    # Chest mark.
    for row, line in enumerate(MARK):
        for col, ch in enumerate(line):
            if ch == "X":
                img.putpixel((19 + col, 33 + row), CREAM)
    return img


def eyes(img: Image.Image, state: str) -> None:
    d = ImageDraw.Draw(img)
    if state == "sleeping":
        d.rectangle([16, 20, 20, 20], fill=CREAM)
        d.rectangle([23, 20, 27, 20], fill=CREAM)
        return
    if state == "happy":
        # Squint: the lower row is shorter than the upper one.
        d.rectangle([16, 18, 20, 19], fill=CREAM)
        d.rectangle([23, 18, 27, 19], fill=CREAM)
        d.rectangle([17, 20, 19, 20], fill=CREAM)
        d.rectangle([24, 20, 26, 20], fill=CREAM)
        return
    if state == "loading":
        # Looking down, busy with something.
        d.rectangle([16, 20, 20, 22], fill=CREAM)
        d.rectangle([23, 20, 27, 22], fill=CREAM)
        return
    if state == "thinking":
        # One eye squinting.
        d.rectangle([16, 18, 20, 21], fill=CREAM)
        d.rectangle([23, 19, 27, 20], fill=CREAM)
        return
    d.rectangle([16, 18, 20, 21], fill=CREAM)
    d.rectangle([23, 18, 27, 21], fill=CREAM)


def extras(img: Image.Image, state: str) -> None:
    d = ImageDraw.Draw(img)
    if state == "happy":
        for x, y in ((37, 8), (39, 6), (39, 10), (5, 14)):
            d.rectangle([x, y, x + 1, y + 1], fill=CREAM)
    elif state == "loading":
        # The same three dots that animate in the "Loading…" label.
        for i, x in enumerate((15, 21, 27)):
            d.rectangle([x, 49, x + 2, 50], fill=ACCENT if i == 0 else ROBE_LIGHT)
    elif state == "thinking":
        d.rectangle([36, 10, 38, 12], fill=CREAM)
        d.rectangle([39, 6, 42, 9], fill=CREAM)
    elif state == "sleeping":
        d.rectangle([36, 10, 38, 11], fill=CREAM)
        d.rectangle([38, 6, 41, 8], fill=CREAM)


def main() -> None:
    for state in ("idle", "happy", "loading", "thinking", "sleeping"):
        img = base()
        eyes(img, state)
        extras(img, state)
        big = img.resize((W * SCALE, H * SCALE), Image.NEAREST)
        big.save(OUT / f"mascot-{state}.png")

    # Head on its own, for rows where the whole figure does not fit.
    head = base()
    eyes(head, "idle")
    head = head.crop((6, 0, 38, 32)).resize((32 * SCALE, 32 * SCALE), Image.NEAREST)
    head.save(OUT / "mascot-head.png")
    print("drawn:", ", ".join(sorted(p.name for p in OUT.glob("mascot-*.png"))))


if __name__ == "__main__":
    main()
