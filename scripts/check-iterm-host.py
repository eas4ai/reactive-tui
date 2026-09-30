#!/usr/bin/env python3
"""Measure iTerm2 image pixels on a dedicated macOS desktop.

Uses iTerm2's --command launch option and captures only its own window.
Never alters macOS privacy permissions or replaces an existing iTerm2 session.
"""
import argparse
import hashlib
import io
import json
import platform
import runpy
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE_URL = "https://iterm2.com/downloads/stable/iTerm2-3_7_0.zip"
ARCHIVE_SHA256 = "14b5131e9134d0012466574fba6d69fb9ef84eee66660ee861e2da483089574a"
COLORS = {"red": (255, 0, 0), "blue": (0, 0, 255),
          "green": (0, 255, 0), "yellow": (255, 255, 0)}

# Title-bar chrome (traffic lights) in points. The shadowless (-o) capture puts
# the ~28 pt iTerm title bar at the top of the shot; cropping just the bar is
# a safe underestimate because the rows between it and the image (padding and
# the white stage label) hold no dominant-channel colors. Calibrated against
# the -o captures; the guard in chrome_rows fails loudly if geometry shifts.
CHROME_POINTS = 28


def chrome_rows(screenshot, bounds):
    """Top rows to skip so traffic lights never enter the measurement."""
    from PIL import Image
    with Image.open(screenshot) as source:
        width, height = source.size
    try:
        scale = width / float((bounds or {}).get("Width", 0) or 0)
    except (TypeError, ValueError):
        scale = 0
    if not scale > 0:
        raise RuntimeError(
            f"Cannot scale title-bar chrome without window width: {bounds!r}")
    chrome = round(CHROME_POINTS * scale)
    if not 0 < chrome < height // 3:
        raise RuntimeError(
            f"Title-bar chrome {chrome} implausible for {width}x{height}: {bounds!r}")
    return chrome


def measurements(screenshot, *, exact_srgb=True, chrome=0):
    # Dominant-channel regions verify geometry only. The developer approved
    # iTerm 3.7's color/transparency exclusion; exact measurements stay recorded.
    from PIL import Image, ImageCms
    with Image.open(screenshot) as source:
        pixels = source.convert("RGB")
        if source.info.get("icc_profile"):
            pixels = ImageCms.profileToProfile(pixels,
                ImageCms.ImageCmsProfile(io.BytesIO(source.info["icc_profile"])),
                ImageCms.createProfile("sRGB"), outputMode="RGB")
        if chrome:
            pixels = pixels.crop((0, chrome, pixels.width, pixels.height))
        result = {}
        for name, color in COLORS.items():
            points = [(x, y) for y in range(pixels.height) for x in range(pixels.width)
                      if (pixels.getpixel((x, y)) == color if exact_srgb else
                          all(value > 192 if expected else value < 128
                              for value, expected in zip(pixels.getpixel((x, y)), color)))]
            result[name] = {"count": len(points), "bounds":
                [min(x for x, _ in points), min(y for _, y in points),
                 max(x for x, _ in points), max(y for _, y in points)] if points else None}
        return result


def verify_pixels(results):
    assert results[0]["red"]["count"] > 100 and results[0]["blue"]["count"] > 100, results[0]
    assert results[1]["green"]["count"] > 100 and results[1]["yellow"]["count"] > 100, results[1]
    assert results[1]["red"]["count"] == results[1]["blue"]["count"] == 0, results[1]
    assert results[1]["green"]["bounds"][0] > results[0]["red"]["bounds"][2], results
    assert results[1]["green"]["bounds"][1] > results[0]["red"]["bounds"][3], results
    assert all(results[2][name]["count"] == 0 for name in COLORS), results[2]


def owned_windows(pid, directory):
    """Query only windows belonging to the iTerm2 process started here."""
    script = '''import AppKit
import Foundation
let pid = Int(CommandLine.arguments[1])!
let windows = CGWindowListCopyWindowInfo([.optionAll, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
let owned = windows.filter { ($0[kCGWindowOwnerPID as String] as? Int) == pid }
let data = try JSONSerialization.data(withJSONObject: owned, options: [.prettyPrinted, .sortedKeys])
FileHandle.standardOutput.write(data)
'''
    result = subprocess.run(["swift", "-e", script, str(pid)],
                            capture_output=True, timeout=30)
    (directory / "startup-windows.json").write_bytes(result.stdout)
    (directory / "startup-windows.stderr").write_bytes(result.stderr)
    result.check_returncode()
    return json.loads(result.stdout)


def startup_diagnostics(pid, directory):
    """Capture only windows belonging to the iTerm2 process started here."""
    for window in owned_windows(pid, directory):
        window_id = window.get("kCGWindowNumber")
        if isinstance(window_id, int) and window.get("kCGWindowIsOnscreen"):
            result = subprocess.run(["/usr/sbin/screencapture", "-x", "-l", str(window_id),
                                     str(directory / f"startup-{window_id}.png")],
                                    capture_output=True, timeout=10)
            (directory / f"startup-{window_id}.stderr").write_bytes(result.stderr)


def capture(app, directory, mode, executable):
    directory.mkdir(parents=True, exist_ok=True)
    existing = subprocess.run(["pgrep", "-x", "iTerm2"], capture_output=True)
    if existing.returncode != 1:
        raise RuntimeError("iTerm2 is already running or process inspection failed")
    child = None
    with tempfile.TemporaryDirectory(prefix="rtui-iterm-probe-") as work:
        work = Path(work)
        stage, ready, status = [work / name for name in ("stage", "ready", "status")]
        identity = uuid.uuid4().hex
        title = "RTUI-IMAGE-" + identity
        suite = "org.reactive-tui.image-probe." + identity
        runner = work / "run.py"
        runner.write_text(
            "import json, os, pathlib, subprocess, sys, time\n"
            f"stage = pathlib.Path({str(stage)!r})\n"
            "size = os.get_terminal_size()\n"
            "assert size.columns >= 24 and size.lines >= 12, size\n"
            f"sys.stdout.write({chr(27) + ']0;' + title + chr(7)!r})\n"
            "sys.stdout.flush()\n"
            f"pathlib.Path({str(ready)!r}).write_text(json.dumps(list(size)))\n"
            "deadline = time.monotonic() + 20\n"
            "while not stage.exists():\n"
            "    if time.monotonic() > deadline: raise RuntimeError('driver did not start')\n"
            "    time.sleep(.05)\n"
            f"result = subprocess.run({[str(executable), str(stage), mode]!r}, stderr=subprocess.PIPE, timeout=50)\n"
            f"pathlib.Path({str(directory / 'fixture-stderr.txt')!r}).write_bytes(result.stderr)\n"
            f"pathlib.Path({str(status)!r}).write_text(str(result.returncode))\n"
        )
        wrapper = work / "run.sh"
        wrapper.write_text("#!/bin/sh\nexec " + shlex.join([sys.executable, str(runner)]) + "\n")
        wrapper.chmod(0o700)
        try:
            with (directory / "host.log").open("wb") as log:
                # iTerm 3.7 asks before displaying inline images. Authorize our
                # generated fixture only in this process's disposable defaults
                # suite. Cocoa argument defaults do not modify the user's suite.
                child = subprocess.Popen([str(app / "Contents/MacOS/iTerm2"),
                    "-suite", suite,
                    "-NoSyncSuppressDownloadConfirmation", "YES",
                    "-NoSyncSuppressDownloadConfirmation_selection", "0",
                    "--command=" + shlex.quote(str(wrapper))], stdout=log, stderr=log)
            (directory / "host-settings.json").write_text(json.dumps({
                "suite": suite, "inline_image_permission": "allow generated fixture",
                "permission_source": "process argument defaults"}, indent=2) + "\n")
            deadline = time.monotonic() + 25
            while not ready.exists() or not ready.read_text().strip():
                if child.poll() is not None or time.monotonic() >= deadline:
                    try:
                        startup_diagnostics(child.pid, directory)
                    except (OSError, ValueError, subprocess.SubprocessError) as error:
                        (directory / "startup-diagnostics-error.txt").write_text(str(error))
                    raise RuntimeError("iTerm2 did not create the probe window; see host.log")
                time.sleep(.1)
            (directory / "terminal-size.json").write_text(ready.read_text())
            while True:
                windows = [window for window in owned_windows(child.pid, directory)
                    if window.get("kCGWindowIsOnscreen") and window.get("kCGWindowLayer") == 0
                    and title in window.get("kCGWindowName", "")]
                if len(windows) == 1:
                    break
                if len(windows) > 1 or child.poll() is not None or time.monotonic() >= deadline:
                    startup_diagnostics(child.pid, directory)
                    raise RuntimeError("Expected exactly one owned iTerm2 window with the probe title")
                time.sleep(.1)
            window = windows[0]
            window_id = str(window.get("kCGWindowNumber"))
            if not window_id.isdecimal():
                raise RuntimeError("Invalid iTerm2 window ID")
            shots = []
            for number in range(3):
                pending = work / "next"
                pending.write_text(str(number))
                pending.replace(stage)
                time.sleep(4)
                if child.poll() is not None or status.exists():
                    raise RuntimeError("iTerm2 or image fixture exited before capture")
                screenshot = directory / f"stage-{number}.png"
                subprocess.run(["/usr/sbin/screencapture", "-x", "-o", "-l", window_id, str(screenshot)],
                               check=True, timeout=10)
                shots.append(screenshot)
            pending = work / "next"
            pending.write_text("3")
            pending.replace(stage)
            deadline = time.monotonic() + 10
            while not status.exists():
                if time.monotonic() >= deadline:
                    raise RuntimeError("Image fixture did not exit")
                time.sleep(.05)
            code = status.read_text()
            (directory / "fixture-exit.txt").write_text(code)
            if code != "0":
                raise RuntimeError("Image fixture failed: " + code)
            # Pixel analysis runs after the fixture exits so its cost never
            # burns the probe's 45 s watchdog, and skips the title-bar chrome
            # (traffic lights) that screencapture -l includes.
            bounds = window.get("kCGWindowBounds", {})
            chrome = chrome_rows(shots[0], bounds)
            (directory / "chrome.json").write_text(json.dumps(
                {"chrome_rows": chrome, "window_bounds": bounds}) + "\n")
            results = [measurements(shot, chrome=chrome) for shot in shots]
            geometry = [measurements(shot, exact_srgb=False, chrome=chrome) for shot in shots]
            (directory / "pixels.json").write_text(json.dumps(results, indent=2) + "\n")
            (directory / "geometry-pixels.json").write_text(json.dumps(geometry, indent=2) + "\n")
            return results, geometry
        finally:
            if child is not None and child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
            # iTerm may save other startup preferences in its unique suite.
            cleanup = '''import Foundation
let suite = CommandLine.arguments[1]
for name in [suite, suite + ".private"] {
    UserDefaults.standard.removePersistentDomain(forName: name)
}
'''
            subprocess.run(["swift", "-e", cleanup, suite], check=True, timeout=30)


def prepare_archive(work, supplied=None):
    path = supplied.resolve() if supplied else work / "iterm.zip"
    if not supplied:
        subprocess.run(["curl", "--fail", "--location", "--max-time", "60",
                        "--max-filesize", str(80 * 1024 * 1024), "--output", str(path),
                        ARCHIVE_URL], check=True, timeout=65)
    if path.stat().st_size > 80 * 1024 * 1024:
        raise RuntimeError("iTerm2 archive exceeds size limit")
    with path.open("rb") as stream:
        archive = stream.read(80 * 1024 * 1024 + 1)
    if len(archive) > 80 * 1024 * 1024:
        raise RuntimeError("iTerm2 archive exceeds size limit")
    if hashlib.sha256(archive).hexdigest() != ARCHIVE_SHA256:
        raise RuntimeError("iTerm2 archive digest mismatch")
    if supplied:
        path = work / "iterm.zip"
        path.write_bytes(archive)
    return path


def run(args):
    if platform.system() != "Darwin" or not args.dedicated_desktop:
        raise RuntimeError("Requires macOS and --dedicated-desktop")
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    executable = args.executable or runpy.run_path(str(ROOT / "scripts/check-widget-platforms.py"))["build_probe"](output / "build.out")
    with tempfile.TemporaryDirectory(prefix="rtui-iterm-install-") as work:
        work = Path(work)
        path = prepare_archive(work, args.archive)
        # iTerm 3.7.0's LetsMove accepts an Applications path component. Keep the
        # bundle private while avoiding its first-launch relocation dialog.
        applications = work / "Applications"
        applications.mkdir()
        subprocess.run(["ditto", "-x", "-k", str(path), str(applications)], check=True, timeout=30)
        app = applications / "iTerm.app"
        for mode in ["app-iterm", "app-auto", "surface-iterm", "iterm", "app-ascii"]:
            exact, geometry = capture(app, output / mode, mode, executable)
            results = exact if args.require_exact_srgb else geometry
            if mode == "app-ascii":
                try:
                    verify_pixels(results)
                except AssertionError:
                    if any(result[name]["count"] for result in results for name in COLORS):
                        raise RuntimeError("ASCII negative has unexpected image colors")
                else:
                    raise RuntimeError("ASCII negative incorrectly passed the image pixel assertions")
                print("iTerm2: ASCII violating image case rejected", flush=True)
            else:
                verify_pixels(results)
                print(f"iTerm2 {mode}: image presence, update, movement and removal passed", flush=True)
        (output / "host.json").write_text(json.dumps({"version": "3.7.0",
            "archive_sha256": ARCHIVE_SHA256, "platform": platform.platform(),
            "color_transparency": "unsupported; approved api-011-api-014-api-020",
            "measurement": "exact-srgb" if args.require_exact_srgb else "dominant-channel geometry"}, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dedicated-desktop", action="store_true")
    parser.add_argument("--require-exact-srgb", action="store_true",
                        help="Run the retained strict color diagnostic; iTerm 3.7 is known to fail")
    parser.add_argument("--executable", type=Path, help="Cargo-reported image_host_probe executable")
    parser.add_argument("--archive", type=Path, help="Optional local archive; pinned SHA-256 is always required")
    parser.add_argument("--output", default="target/evidence/widget-platforms/darwin/iterm-host")
    run(parser.parse_args())
