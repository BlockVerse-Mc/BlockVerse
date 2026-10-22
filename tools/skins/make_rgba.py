#!/usr/bin/env python3
"""Converts the default skin PNGs to raw RGBA."""
import glob, os
from PIL import Image

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "crates", "bv-skins", "assets")
for png in sorted(glob.glob(os.path.join(root, "png", "*.png"))):
    im = Image.open(png).convert("RGBA")
    assert im.size == (64, 64), f"{png}: expected 64x64, got {im.size}"
    out = os.path.join(root, "rgba", os.path.basename(png)[:-4] + ".rgba")
    with open(out, "wb") as f:
        f.write(im.tobytes())
    print("wrote", out, os.path.getsize(out), "bytes")
