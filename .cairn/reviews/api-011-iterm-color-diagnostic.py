#!/usr/bin/env python3
"""Diagnostic only: compare untagged, sRGB-chunk and ICC PNGs in iTerm."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from collections import Counter
from PIL import Image

root = Path.cwd()
spec = importlib.util.spec_from_file_location("host", root / "scripts/check-iterm-host.py")
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)
output = root / "docs/analysis/widget-platforms/darwin/color-diagnostic"
output.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix="rtui-color-diagnostic-") as work:
    work = Path(work)
    archive = work / "iterm.zip"
    subprocess.run(["curl", "--fail", "--location", "--max-time", "60", "--output", str(archive), host.ARCHIVE_URL], check=True, timeout=65)
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == host.ARCHIVE_SHA256
    applications = work / "Applications"
    applications.mkdir()
    subprocess.run(["ditto", "-x", "-k", str(archive), str(applications)], check=True, timeout=30)
    host.ROOT = work
    fixture = work / "target/debug/examples/image_host_probe"
    fixture.parent.mkdir(parents=True)
    fixture.write_text("#!" + sys.executable + "\n" + r'''import base64, io, pathlib, sys, time
from PIL import Image, ImageCms, PngImagePlugin
stage_path, mode = pathlib.Path(sys.argv[1]), sys.argv[2]
last = None
started = time.monotonic()
try:
    sys.stdout.write("\x1b[?1049h\x1b[40m\x1b[2J")
    while time.monotonic() - started < 45:
        stage = int(stage_path.read_text())
        if stage == 3: break
        if stage != last:
            sys.stdout.write("\x1b[40m\x1b[2J\x1b[H")
            if stage < 2:
                colors = [(255,0,0,255),(0,0,255,255)] if stage == 0 else [(0,255,0,255),(255,255,0,255)]
                image = Image.new("RGBA", (56,68))
                image.putdata([(colors[x >= 28] if y < 28 else (0,0,0,0)) for y in range(68) for x in range(56)])
                options = {}
                if mode == "srgb":
                    info = PngImagePlugin.PngInfo()
                    info.add(b"sRGB", bytes([0]))
                    options["pnginfo"] = info
                if mode == "icc":
                    options["icc_profile"] = ImageCms.ImageCmsProfile(ImageCms.createProfile("sRGB")).tobytes()
                data = io.BytesIO()
                image.save(data, format="PNG", **options)
                encoded = base64.b64encode(data.getvalue()).decode()
                if mode == "untagged-default": sys.stdout.write("\x1b[0m")
                sys.stdout.write("\x1b[3;3H" if stage == 0 else "\x1b[7;13H")
                sys.stdout.write(f"\x1b]1337;File=size={len(data.getvalue())};width=56px;height=68px;inline=1;preserveAspectRatio=0:{encoded}\x07")
            sys.stdout.flush()
            last = stage
        time.sleep(.05)
    else:
        raise RuntimeError("diagnostic watchdog expired")
finally:
    sys.stdout.write("\x1b[0m\x1b[?1049l")
    sys.stdout.flush()
''')
    fixture.chmod(0o700)
    # Compare the host renderer independently of the library encoder.
    original_popen = subprocess.Popen
    software = False
    def launch(command, *args, **kwargs):
        if software and isinstance(command, list) and command[0].endswith("/iTerm2"):
            command = command[:1] + ["-UseMetal", "NO"] + command[1:]
        return original_popen(command, *args, **kwargs)
    subprocess.Popen = launch
    summary = {}
    for mode in ["untagged", "software"]:
        software = mode == "software"
        directory = output / mode
        host.capture(applications / "iTerm.app", directory, mode)
        summary[mode] = {}
        for stage in range(3):
            with Image.open(directory / f"stage-{stage}.png") as image:
                colors = Counter(image.convert("RGB").getdata())
                summary[mode][stage] = colors.most_common(12)
    (output / "comparison.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
