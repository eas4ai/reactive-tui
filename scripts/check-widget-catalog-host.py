#!/usr/bin/env python3
"""Capture catalog pages in owned Kitty/Xvfb, never the developer desktop."""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("wgpu_host", ROOT / "scripts/check-wgpu-host.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def wait_page(host, size, labels):
    accepted = 0
    deadline = time.monotonic() + 15
    screen = ""
    last_error = "no remote-control state yet"
    while time.monotonic() < deadline:
        if host.process.poll() is not None:
            raise RuntimeError(f"owned catalog host closed before capture: {host.log_output()}")
        try:
            windows = json.loads(host.remote("ls"))[0]["tabs"][0]["windows"]
            screen = host.remote("get-text", "--extent", "screen")
            lines = screen.splitlines()
            ready = (windows[0]["columns"], windows[0]["lines"]) == size and lines
            ready = ready and "Widget Catalog" in lines[0] and "Ctrl+Q" in lines[-1]
            ready = ready and all(label in screen for label in labels)
            if "Braille subpixels" in labels:
                columns, rows = size
                count = (columns if columns < 80 else columns - 24) * (rows - (9 if columns < 80 else 6))
                ready = ready and sum("\u2800" <= char <= "\u28ff" for char in screen) == count
                ready = ready and sum("\u2801" <= char <= "\u28ff" for char in screen) > 30
            accepted = accepted + 1 if ready else 0
            if accepted >= 3:
                return
        except (RuntimeError, IndexError, KeyError, json.JSONDecodeError, subprocess.TimeoutExpired) as error:
            accepted = 0
            last_error = repr(error)
        time.sleep(0.2)
    raise RuntimeError(f"missing catalog host state at {size}: {labels}; last remote error: {last_error}\n{screen}")


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--requirement", choices=["all", "CAT-001", "CAT-002"], default="all")
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    destination = args.output or ROOT / "target/evidence/captures/catalog" / f"{stamp}-{os.getpid()}"
    destination.mkdir(parents=True, exist_ok=False)
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    artifacts = []
    with tempfile.TemporaryDirectory(prefix="reactive-catalog-host-") as private:
        host = module.Host(private)
        try:
            host.launch(*module.SIZES[0], [target / "debug/examples/widget_catalog"])
            wait_page(host, module.SIZES[0], ["Coverage"])
            for index, size in enumerate([*module.SIZES, module.SIZES[0]]):
                if index:
                    host.remote("resize-os-window", "--width", str(size[0]), "--height", str(size[1]), "--unit", "cells")
                for page, key, labels in [
                    ("overview", "1", ["Coverage"] + (["TerminalWidget"] if size[0] >= 80 else [])),
                    ("layout", "3", ["Column spans", "span 4", "span 2", "span 3"]
                     + (["Breadcrumb", "Accordion", "Tabs", "ScrollView", "Stack"] if size[0] >= 80 else [])),
                    ("motion", "7", ["Braille subpixels"]),
                ]:
                    if args.requirement == "CAT-001" and page == "motion":
                        continue
                    if args.requirement == "CAT-002" and page != "motion":
                        continue
                    if key != "1" or index:  # Overview is the startup page; wait for its socket first.
                        host.remote("send-key", key)
                    wait_page(host, size, labels)
                    suffix = "-return" if index == len(module.SIZES) else ""
                    path = destination / f"kitty-{page}-{size[0]}x{size[1]}{suffix}.png"
                    subprocess.run(["/usr/bin/import", "-display", host.env["DISPLAY"], "-window",
                                    "Reactive GPU acceptance", str(path)], check=True, timeout=10, env=host.env)
                    wait_page(host, size, labels)
                    info = module.artifact(path)
                    artifacts.append(info)
                    print(f"CATALOG HOST CAPTURE {json.dumps(info)}", flush=True)
            host.finish(quit_key=True)
            diagnostics = host.log_output()
        finally:
            host.close()
    manifest = destination / "host.json"
    manifest.write_text(json.dumps({"kitty": subprocess.check_output(["kitty", "--version"], text=True).strip(),
                                   "display": "owned Xvfb X11", "font": "DejaVu Sans Mono 12 pt",
                                   "host_gl": "software glyph renderer", "artifacts": artifacts,
                                   "host_stderr_excerpt": diagnostics}, indent=2) + "\n")
    print(f"CATALOG HOST MANIFEST {json.dumps(module.artifact(manifest))}", flush=True)
    print("REVIEW: inspect actual PNGs; text readiness does not prove visual quality", flush=True)


if __name__ == "__main__":
    run()
