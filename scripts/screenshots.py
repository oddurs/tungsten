#!/usr/bin/env python3
"""Renders tungsten's real ANSI output to PNGs on a dark and a light background.

    uv run --with pillow scripts/screenshots.py

Writes docs/screenshots/{dark,light}.png. Used to check the theme (cairn item
0019) on both kinds of terminal; re-run whenever crates/render/src/theme.rs
changes.
"""

import re
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / "tungsten"
QUERIES = ["60 mph * 2h 15min", "1/3 + 1/6", "3 m + 2 s", "5 kilometers in milez"]
FONT = "/System/Library/Fonts/Menlo.ttc"
SIZE = 15
THEMES = {
    "dark": {"bg": (30, 30, 30), "fg": (212, 212, 212)},
    "light": {"bg": (250, 250, 250), "fg": (30, 30, 30)},
}
LEVELS = [0, 95, 135, 175, 215, 255]


def xterm(i):
    if i >= 232:
        g = 8 + 10 * (i - 232)
        return (g, g, g)
    i -= 16
    return (LEVELS[i // 36], LEVELS[(i // 6) % 6], LEVELS[i % 6])


def spans(text, fg):
    """Yields (text, colour, bold) runs from ANSI-coloured text."""
    colour, bold = fg, False
    for part in re.split(r"(\x1b\[[0-9;]*m)", text):
        if not part.startswith("\x1b["):
            if part:
                yield part, colour, bold
            continue
        codes = [int(c) for c in part[2:-1].split(";") if c]
        i = 0
        while i < len(codes):
            c = codes[i]
            if c == 0:
                colour, bold = fg, False
            elif c == 1:
                bold = True
            elif c == 38 and codes[i + 1] == 5:
                colour = xterm(codes[i + 2])
                i += 2
            i += 1


def main():
    subprocess.run(["cargo", "build", "--release", "-q"], cwd=ROOT, check=True)
    out = []
    for q in QUERIES:
        r = subprocess.run(
            [BIN, "--color", "always", "--width", "72", *q.split()],
            capture_output=True,
            text=True,
        )
        out.append(f"$ w {q}")
        out.extend(r.stdout.rstrip("\n").split("\n"))
        out.append("")
    regular = ImageFont.truetype(FONT, SIZE, index=0)
    bold_font = ImageFont.truetype(FONT, SIZE, index=1)
    cw = regular.getlength("M")
    lh = int(SIZE * 1.45)
    w = int(cw * 76) + 40
    h = lh * len(out) + 40
    dest = ROOT / "docs" / "screenshots"
    dest.mkdir(parents=True, exist_ok=True)
    for name, t in THEMES.items():
        img = Image.new("RGB", (w, h), t["bg"])
        d = ImageDraw.Draw(img)
        for row, line in enumerate(out):
            x = 20
            for text, colour, bold in spans(line, t["fg"]):
                for ch in text:
                    d.text((x, 20 + row * lh), ch, font=bold_font if bold else regular, fill=colour)
                    x += cw
        img.save(dest / f"{name}.png")
        print(dest / f"{name}.png")


if __name__ == "__main__":
    main()
