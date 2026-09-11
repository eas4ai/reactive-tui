#!/usr/bin/env python3
"""Check decoded images and real host pixels across the retained image routes."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import runpy
import shutil

ROOT = Path(__file__).resolve().parents[1]
HOSTS = (
    ("kitty", None), ("ghostty", None),
    ("xterm", "app-sixel"), ("xterm", "app-sixel-full"),
    ("wezterm", "app-iterm"), ("wezterm", "app-iterm-full"),
    ("kitty", "app-chafa"), ("kitty", "app-viu"), ("gnome", "app-auto"),
    ("kitty", "kitty"), ("xterm", "sixel"), ("wezterm", "iterm"),
    ("kitty", "surface-kitty"), ("xterm", "surface-sixel"),
    ("wezterm", "surface-iterm"), ("gnome", "surface-fallback"),
    ("kitty", "surface-kitty-behind"), ("xterm", "surface-sixel-behind"),
    ("xterm", "surface-sixel-full"), ("kitty", "surface-kitty-retry"),
    ("kitty", "app-gif"), ("xterm", "app-gif-sixel"),
    ("wezterm", "app-gif-iterm"), ("kitty", "app-gif-chafa"),
    ("kitty", "app-gif-viu"),
)


def main():
    os.chdir(ROOT)
    os.environ["CARGO_INCREMENTAL"] = "0"
    os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
    # Share the existing bounded process-group runner; it kills surviving children
    # on timeout and retains output before reporting an error.
    execute = runpy.run_path(str(ROOT / "scripts/check-widget-platforms.py"))["execute"]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = ROOT / "docs/analysis/api-image-hosts" / stamp
    output.mkdir(parents=True)
    for tool in ("kitty", "ghostty", "xterm", "wezterm", "gnome-terminal", "chafa", "viu", "Xvfb"):
        path = shutil.which(tool)
        if path is None:
            raise RuntimeError("Image host acceptance requires executable on PATH: " + tool)
        print(tool + ": " + path, flush=True)
    for tool in ("kitty", "ghostty", "wezterm", "chafa", "viu"):
        print(execute([tool, "--version"], output / (tool + "-version.out"), 10), flush=True)
    for index, arguments in enumerate((
        ("--lib", "widgets::display::image::"),
        ("--lib", "platform::image::"),
        ("--lib", "core::surface::"),
        ("--lib", "backend::suprtui::graphics::"),
        ("--test", "api_widget_behavior", "image_acceptance::"),
    )):
        log = execute(["cargo", "test", "--locked", *arguments, "--", "--test-threads=1"],
                      output / f"tests-{index}.out", 300)
        print(log, flush=True)
        if not re.search(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", log):
            raise RuntimeError("Image selector did not execute passing tests: " + str(arguments))
    print(execute(["python3", "-B", "scripts/check-widget-platforms.py", "--verify"],
                  output / "native-records.out", 180), flush=True)
    print(execute(["cargo", "build", "--locked", "--example", "image_host_probe"],
                  output / "build.out", 300), flush=True)
    driver = ["/usr/bin/python3", "-B", "tests/api_widget_behavior/image_host_capture.py"]
    # A fallback-only run must reach the pixel assertions and fail them. Missing
    # hosts, failed startup and timeouts cannot satisfy this negative control.
    negative = output / "forced-ascii-negative"
    rejected = False
    try:
        execute(driver + ["kitty", str(negative), "app-ascii"], output / "negative.out", 90)
    except RuntimeError:
        rejected = True
    diagnostic = (output / "negative.out").read_text()
    pixels = json.loads((negative / "pixels.json").read_text())
    if (not rejected or "AssertionError" not in diagnostic
            or not all(stage[color]["count"] == 0 for stage in pixels
                       for color in ("red", "blue", "green", "yellow"))):
        raise RuntimeError("Forced ASCII did not demonstrate rejection by the exact-color check")
    print("PASS negative control: forced ASCII rejected by exact-color assertions", flush=True)
    for host, mode in HOSTS:
        name = host + "-" + (mode or "app-auto")
        directory = output / name
        command = driver + [host, str(directory)] + ([mode] if mode else [])
        print(execute(command, output / (name + ".out"), 90), flush=True)
        print(name + " pixels: " + (directory / "pixels.json").read_text(), flush=True)
    print("PASS API-014: decoded pixels, protocols, lifecycle and all host routes", flush=True)
    print("Retained captures: " + str(output.relative_to(ROOT)), flush=True)


if __name__ == "__main__":
    main()
