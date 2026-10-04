#!/usr/bin/env python3
"""PIX-001's screenshot check (docs/spec/pixel-looks.md): the widget catalog's
Input page in kitty on a private X display (Xvfb), captured with ImageMagick's
import; the background pixel of a button's label cell, of a text input's text
cell and of a card's text cell must equal the picture pixel beside it within 2
of 255 in every channel, so a look's text reads as if drawn into the picture.

Exit 0 when every sampled cell matches, 1 with the mismatch on the last line,
and 2 when no private display can be had (the check is then unverified).

Not written yet: the pixel-widget-looks commitment fills this in with the
owned Xvfb and kitty host of scripts/check-wgpu-host.py.
"""
import sys

sys.stderr.write("pixel-looks-screenshot.py: not written yet\n")
sys.exit(2)
