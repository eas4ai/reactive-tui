#!/usr/bin/env python3
"""Owned Xvfb/Kitty host: no connection to the developer's desktop or sockets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SIZES = [(60, 24), (144, 50), (200, 60)]


def stop(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=3)


class Host:
    def __init__(self, directory):
        self.directory = directory
        self.server = subprocess.Popen(
            ["Xvfb", "-displayfd", "1", "-screen", "0", "3000x1600x24", "-nolisten", "tcp"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True,
        )
        if not select.select([self.server.stdout], [], [], 5)[0]:
            stop(self.server)
            raise RuntimeError("private X server did not start")
        display = self.server.stdout.readline().decode().strip()
        if not display.isdigit():
            stop(self.server)
            raise RuntimeError("invalid private display number")
        self.env = dict(os.environ, DISPLAY=f":{display}", WAYLAND_DISPLAY="", LIBGL_ALWAYS_SOFTWARE="1")
        # Never inherit another Kitty instance's remote-control destination.
        for key in ["KITTY_LISTEN_ON", "KITTY_WINDOW_ID", "KITTY_PID"]:
            self.env.pop(key, None)
        self.process = None
        self.counter = 0

    def launch(self, columns, rows, command):
        self.counter += 1
        self.socket = f"unix:{self.directory}/kitty-{self.counter}.sock"
        self.log_path = Path(self.directory) / f"kitty-{self.counter}.stderr"
        # Animation warnings can fill an undrained PIPE and freeze remote
        # control. The child owns its file descriptor after this scope closes.
        with self.log_path.open("wb") as diagnostics:
            self.process = subprocess.Popen(
                ["kitty", "--config", "NONE", "--listen-on", self.socket,
                 "--title", "Reactive GPU acceptance", "-o", "allow_remote_control=socket-only",
                 "-o", "linux_display_server=x11", "-o", "remember_window_size=no",
                 "-o", f"initial_window_width={columns}c", "-o", f"initial_window_height={rows}c",
                 "-o", "window_padding_width=0", "-o", "font_family=DejaVu Sans Mono",
                 "-o", "font_size=12", *map(str, command)],
                env=self.env, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=diagnostics,
                start_new_session=True,
            )

    def log_output(self):
        with self.log_path.open("rb") as diagnostics:
            return diagnostics.read(65536).decode(errors="replace")

    def remote(self, *args):
        result = subprocess.run(["kitty", "@", "--to", self.socket, *args],
                                env=self.env, text=True, capture_output=True, timeout=10)
        if result.returncode:
            raise RuntimeError(result.stderr.strip())
        return result.stdout

    def frame_ready(self, columns, rows, mode):
        windows = json.loads(self.remote("ls"))[0]["tabs"][0]["windows"]
        screen = self.remote("get-text", "--extent", "screen")
        compact = columns < 80
        count = (columns if compact else columns - 24) * (rows - (9 if compact else 6))
        lines = screen.splitlines()
        return bool(lines and (windows[0]["columns"], windows[0]["lines"]) == (columns, rows)
                    and mode in screen and screen.count("▀") == count
                    and "Reactive TUI · Widget Catalog" in lines[0]
                    and "Ctrl+Q" in lines[-1]
                    and "Starting graphics" not in screen and "Preparing viewport" not in screen)

    def wait_frame(self, columns, rows, mode):
        deadline = time.monotonic() + 15
        last_error = None
        accepted = 0
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                  raise RuntimeError(f"Kitty closed before frame: {self.log_output()}")
            try:
                # Kitty geometry changes before the application redraws. One
                # sample may still contain an accepted pre-resize screen.
                accepted = accepted + 1 if self.frame_ready(columns, rows, mode) else 0
                if accepted >= 3:
                    return
            except (RuntimeError, IndexError, KeyError, json.JSONDecodeError, subprocess.TimeoutExpired) as error:
                last_error = error  # Startup socket/host state may not be ready yet.
                accepted = 0
            time.sleep(0.2)
        raise RuntimeError(f"missing stable full {mode} host frame at {columns}x{rows}") from last_error

    def capture(self, columns, rows, path):
        self.wait_frame(columns, rows, "GPU ·")
        subprocess.run(["/usr/bin/import", "-display", self.env["DISPLAY"], "-window", "Reactive GPU acceptance", str(path)],
                       check=True, timeout=10, env=self.env)
        if not self.frame_ready(columns, rows, "GPU ·"):
            raise RuntimeError("host frame became incomplete during capture; artifact is not accepted")

    def finish(self, quit_key=False):
        if quit_key:
            self.remote("send-key", "ctrl+q")
        try:
            code = self.process.wait(timeout=30)
            if code:
                raise RuntimeError(f"Kitty/app exit {code}: {self.log_output()}")
        finally:
            stop(self.process)
            self.process = None

    def close(self):
        if self.process is not None:
            stop(self.process)
        stop(self.server)


def artifact(path):
    return {"path": str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def run(destination, captures_only=False):
    destination.mkdir(parents=True, exist_ok=False)
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    examples = target / "debug" / "examples"
    with tempfile.TemporaryDirectory(prefix="reactive-wgpu-host-") as private:
        host = Host(private)
        try:
            metadata = {"kitty": subprocess.check_output(["kitty", "--version"], text=True).strip(),
                        "display": "private Xvfb X11", "host_gl": "LIBGL_ALWAYS_SOFTWARE=1 (host glyph renderer only)",
                        "font": "DejaVu Sans Mono 12 pt", "artifacts": []}
            host.launch(60, 24, [examples / "widget_catalog", "--motion"])
            for index, (columns, rows) in enumerate([*SIZES, SIZES[0]]):
                if index:
                    host.remote("resize-os-window", "--width", str(columns), "--height", str(rows), "--unit", "cells")
                suffix = "-return" if index == len(SIZES) else ""
                path = destination / f"kitty-gpu-{columns}x{rows}{suffix}.png"
                host.capture(columns, rows, path)
                metadata["artifacts"].append(artifact(path))
                print(f"HOST CAPTURE {columns}x{rows} {json.dumps(artifact(path))}", flush=True)
            host.finish(quit_key=True)
            if not captures_only:
                for columns, rows in SIZES:
                    for mode in ["GPU", "CPU"]:
                        path = destination / f"benchmark-{mode.lower()}-{columns}x{rows}.json"
                        command = [examples / "wgpu_benchmark", "--columns", columns, "--rows", rows,
                                   "--seconds", "1", "--report", path]
                        if mode == "CPU":
                            command.append("--cpu")
                        host.launch(columns, rows, command)
                        host.finish()
                        data = json.loads(path.read_text())
                        assert data["mode"] == mode, "fallback cannot replace GPU measurement"
                        metadata["artifacts"].append(artifact(path))
                        print(f"HOST BENCHMARK {columns}x{rows} {mode} {json.dumps(data)}", flush=True)
            manifest = destination / "host.json"
            manifest.write_text(json.dumps(metadata, indent=2) + "\n")
            print(f"HOST MANIFEST {json.dumps(artifact(manifest))}", flush=True)
        finally:
            host.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True, help="new artifact directory; never overwritten")
    parser.add_argument("--captures-only", action="store_true")
    args = parser.parse_args()
    run(args.output.resolve(), args.captures_only)
