#!/usr/bin/env python3
"""PIX-001's screenshot check (docs/spec/pixel-looks.md): the widget catalog's
Input page in kitty on a private X display (Xvfb), captured with ImageMagick's
import; the background of a button's label cell, of a text input's text cell
and of a card's text cell must equal the picture pixel beside it within 2 of
255 in every channel, so a look's text reads as if drawn into the picture;
then, with the pointer over the Save button, whose fill is drawn at 90
percent, its label cell must still equal the picture beside it.

The catalog is built with `wgpu-graphics` and shown at 240 by 60 cells on
the owned host of scripts/check-wgpu-host.py (never the developer's desktop).
For each sampled text the last glyph's cell is compared with the cell after
it, which holds no glyph and shows the picture: the cell's background is its
most common color, which a glyph never is.

Exit 0 when every sampled cell matches, 1 with the mismatch on the last line,
and 2 when no private display can be had (the check is then unverified).
"""
import collections
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SIZE = (240, 60)
JOBS = os.environ.get("CARGO_BUILD_JOBS", "8")
TOLERANCE = 2
# The text whose last cell is sampled, and what it is.
SAMPLES = [
    ("Save", "the primary button's label"),
    ("Cancel", "the button's label"),
    ("Reactive TUI", "the text input's text"),
    ("TextInput", "the card's text"),
]


def unverified(reason):
    print(f"pixel-looks-screenshot.py: no private display: {reason}")
    return 2


def host_module():
    spec = importlib.util.spec_from_file_location("wgpu_host", ROOT / "scripts/check-wgpu-host.py")
    assert spec is not None and spec.loader is not None, "scripts/check-wgpu-host.py is missing"
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build_catalog():
    """The catalog example with pixel looks; its path."""
    metadata = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps"],
                              cwd=ROOT, capture_output=True, text=True, check=True)
    target = Path(json.loads(metadata.stdout)["target_directory"])
    build = subprocess.run(["cargo", "build", "--locked", "--example", "widget_catalog",
                            "--features", "wgpu-graphics", "--jobs", JOBS],
                           cwd=ROOT, capture_output=True, text=True)
    if build.returncode:
        raise RuntimeError(f"the catalog did not build: {build.stderr[-1500:]}")
    return target / "debug/examples/widget_catalog"


def wait_text(host, labels, deadline_seconds=20):
    """The screen once it shows the page with `labels`, stable for three reads."""
    deadline = time.monotonic() + deadline_seconds
    accepted, screen, last_error = 0, "", "no remote-control state yet"
    while time.monotonic() < deadline:
        if host.process.poll() is not None:
            raise RuntimeError(f"kitty closed before the page showed: {host.log_output()}")
        try:
            windows = json.loads(host.remote("ls"))[0]["tabs"][0]["windows"]
            screen = host.remote("get-text", "--extent", "screen")
            lines = screen.splitlines()
            ready = (windows[0]["columns"], windows[0]["lines"]) == SIZE and bool(lines)
            ready = ready and "Widget Catalog" in lines[0] and "Ctrl+Q" in lines[-1]
            ready = ready and all(label in screen for label in labels)
            accepted = accepted + 1 if ready else 0
            if accepted >= 3:
                return screen
        except (RuntimeError, IndexError, KeyError, json.JSONDecodeError, subprocess.TimeoutExpired) as error:
            accepted = 0
            last_error = repr(error)
        time.sleep(0.2)
    raise RuntimeError(f"the page with {labels} never showed; last remote error: {last_error}\n{screen}")


def pixels_of(path):
    """The screenshot as (width, height, rows of RGB triples)."""
    try:
        from PIL import Image  # type: ignore
    except ImportError:
        Image = None
    if Image is not None:
        with Image.open(path) as image:
            image = image.convert("RGB")
            width, height = image.size
            data = image.tobytes()
    else:
        magick = shutil.which("magick") or shutil.which("convert") or "/usr/bin/convert"
        identify = subprocess.run([magick, str(path), "-format", "%w %h", "info:"],
                                  capture_output=True, text=True, check=True)
        width, height = map(int, identify.stdout.split())
        data = subprocess.run([magick, str(path), "-depth", "8", "rgb:-"],
                              capture_output=True, check=True).stdout
    rows = [[tuple(data[(y * width + x) * 3:(y * width + x) * 3 + 3]) for x in range(width)]
            for y in range(height)]
    return width, height, rows


def cell_background(rows, cell, column, row):
    """The most common color of the cell at `column`, `row`: its background."""
    (cell_w, cell_h) = cell
    colors = collections.Counter(
        rows[y][x]
        for y in range(row * cell_h, (row + 1) * cell_h)
        for x in range(column * cell_w, (column + 1) * cell_w)
    )
    return colors.most_common(1)[0][0]


def main():
    for tool in ("Xvfb", "kitty"):
        if shutil.which(tool) is None:
            return unverified(f"{tool} is not installed")
    if not Path("/usr/bin/import").is_file():
        return unverified("ImageMagick's /usr/bin/import is not installed")
    module = host_module()
    try:
        binary = build_catalog()
    except (RuntimeError, subprocess.CalledProcessError) as error:
        print(f"pixel-looks-screenshot.py: {error}")
        return 1
    with tempfile.TemporaryDirectory(prefix="pixel-looks-shot-") as private:
        try:
            host = module.Host(private)
        except RuntimeError as error:
            return unverified(str(error))
        shot = Path(private) / "input-page.png"
        hovered_shot = Path(private) / "input-page-hovered.png"
        try:
            host.launch(*SIZE, [binary])
            wait_text(host, ["Coverage"])
            host.remote("send-key", "2")
            screen = wait_text(host, [text for text, _ in SAMPLES])
            # The looks' pictures follow the cells by a frame or two.
            time.sleep(1.5)
            screen = wait_text(host, [text for text, _ in SAMPLES])
            subprocess.run(["/usr/bin/import", "-display", host.env["DISPLAY"], "-window",
                            "Reactive GPU acceptance", str(shot)], check=True, timeout=10, env=host.env)
            after = host.remote("get-text", "--extent", "screen")
            if after != screen:
                raise RuntimeError("the screen changed while it was captured")
            width, height, rows = pixels_of(shot)
            # The pointer over the middle of the Save label: an SGR motion
            # report at its cell, as the terminal sends it.
            save = next(((row, line.index("Save")) for row, line in enumerate(screen.splitlines())
                         if row > 0 and "Save" in line), None)
            hovered_rows = None
            if save is not None:
                host.remote("send-text", f"\x1b[<35;{save[1] + 3};{save[0] + 1}M")
                time.sleep(1.5)
                subprocess.run(["/usr/bin/import", "-display", host.env["DISPLAY"], "-window",
                                "Reactive GPU acceptance", str(hovered_shot)], check=True, timeout=10, env=host.env)
                hovered_rows = pixels_of(hovered_shot)[2]
            host.finish(quit_key=True)
        except (RuntimeError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
            print(f"pixel-looks-screenshot.py: {error}")
            return 1
        finally:
            host.close()
    columns, lines = SIZE
    # The captured window is the cells plus at most a pixel or two of frame.
    cell = (width // columns, height // lines)
    if cell[0] < 4 or cell[1] < 8 or width - cell[0] * columns > 2 or height - cell[1] * lines > 2:
        print(f"pixel-looks-screenshot.py: the window is {width}x{height} pixels, not {columns}x{lines} cells")
        return 1
    text_rows = screen.splitlines()
    problems = []
    for needle, what in SAMPLES:
        # Row 0 is the catalog's header, which also says Reactive TUI.
        found = next(((row, line.index(needle)) for row, line in enumerate(text_rows)
                      if row > 0 and needle in line), None)
        if found is None:
            problems.append(f"{what} ({needle!r}) is not on the Input page")
            continue
        row, column = found
        last = column + len(needle) - 1
        text_background = cell_background(rows, cell, last, row)
        picture = cell_background(rows, cell, last + 1, row)
        gap = max(abs(a - b) for a, b in zip(text_background, picture))
        print(f"{what}: cell ({last}, {row}) background {text_background}, picture beside it {picture}, gap {gap}")
        if gap > TOLERANCE:
            problems.append(f"{what}: the background {text_background} of cell ({last}, {row}) differs from the "
                            f"picture pixel {picture} beside it by {gap} of 255")
    # Under the pointer the button's fill is drawn at 90 percent over what is
    # under it, and its label cells take that fill: the label's first cell
    # and the padding cell before it, which holds no glyph, are one color.
    found = next(((row, line.index("Save")) for row, line in enumerate(text_rows)
                  if row > 0 and "Save" in line), None)
    if found is None or hovered_rows is None:
        problems.append("the Save button is not on the Input page to put the pointer over")
    else:
        row, column = found
        resting = cell_background(rows, cell, column, row)
        label = cell_background(hovered_rows, cell, column, row)
        picture = cell_background(hovered_rows, cell, column - 1, row)
        gap = max(abs(a - b) for a, b in zip(label, picture))
        print(f"the hovered primary button's label: cell ({column}, {row}) background {label}, "
              f"picture beside it {picture}, gap {gap} (at rest {resting})")
        if label == resting:
            problems.append(f"the pointer over the Save button did not change its fill ({label})")
        elif gap > TOLERANCE:
            problems.append(f"the hovered primary button's label: the background {label} of cell ({column}, {row}) "
                            f"differs from the picture pixel {picture} beside it by {gap} of 255")
    if problems:
        print(problems[-1])
        return 1
    print(f"every sampled text cell matches the picture beside it within {TOLERANCE} of 255 "
          f"({width}x{height} pixels, cells of {cell[0]}x{cell[1]})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
