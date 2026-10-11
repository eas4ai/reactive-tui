#!/usr/bin/python3
"""Capture isolated X11 hosts; compare their rendered pixels, not protocol text."""
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import time

from PIL import Image


def guard_session(control_fd, command):
    """Keep one owned process group tied to the capture driver's lifetime."""
    try:
        child = subprocess.Popen(command, close_fds=False)
    except OSError as error:
        print(f"cannot start guarded process: {error}", file=sys.stderr, flush=True)
        return 127
    while child.poll() is None:
        if select.select([control_fd], [], [], 0.05)[0] and not os.read(control_fd, 1):
            # The capture driver disappeared before normal cleanup. This process and
            # the guarded command share a private group, so one signal reaps both
            # the launcher and every descendant that retained that group.
            os.killpg(os.getpgrp(), signal.SIGKILL)
    return child.returncode


def start_owned(command, **options):
    """Start a private process group that cannot outlive this capture driver."""
    control_reader, control_writer = os.pipe()
    inherited = tuple(options.pop("pass_fds", ()))
    wrapper = [sys.executable, "-B", str(Path(__file__).resolve()),
               "--guard-session", str(control_reader), *command]
    try:
        process = subprocess.Popen(
            wrapper, pass_fds=(control_reader, *inherited),
            start_new_session=True, **options)
    except BaseException:
        os.close(control_reader)
        os.close(control_writer)
        raise
    os.close(control_reader)
    process.capture_guard_writer = control_writer
    return process


def stop(process):
    if process is None:
        return
    guard_writer = getattr(process, "capture_guard_writer", None)
    try:
        # Every owned host/display starts a private session. D-Bus portal helpers
        # can survive their launcher; terminate that group even after it exits.
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=5)
    finally:
        if guard_writer is not None:
            os.close(guard_writer)
            process.capture_guard_writer = None


def run(host, output, protocol=None, fixture=None):
    output.mkdir(parents=True, exist_ok=True)
    binary = Path(fixture or "target/debug/examples/image_host_probe").resolve()
    colors = {"red": (255, 0, 0), "blue": (0, 0, 255),
              "green": (0, 255, 0), "yellow": (255, 255, 0)}
    with tempfile.TemporaryDirectory(prefix="rtui-image-host-") as temp:
        temp = Path(temp)
        command = temp / "stage"
        command.write_text("0")
        program = [str(binary), str(command)] + ([protocol] if protocol else [])
        reader, writer = os.pipe()
        server = child = None
        try:
            with (output / "xvfb.log").open("wb") as log:
                  server = start_owned(
                      ["Xvfb", "-displayfd", str(writer), "-screen", "0", "1000x700x24", "-nolisten", "tcp"],
                      pass_fds=(writer,), stdout=log, stderr=log)
            os.close(writer)
            writer = None
            if not select.select([reader], [], [], 10)[0]:
                raise RuntimeError("Xvfb failed to allocate a display")
            display = ":" + os.read(reader, 64).decode().strip()
            env = os.environ.copy()
            for key in ("WAYLAND_DISPLAY", "KITTY_WINDOW_ID", "TERM_PROGRAM", "ITERM_SESSION_ID"):
                env.pop(key, None)
            env.update(DISPLAY=display, LIBGL_ALWAYS_SOFTWARE="1", GALLIUM_DRIVER="llvmpipe",
                       LP_NATIVE_VECTOR_WIDTH="128", GALLIUM_OVERRIDE_CPU_CAPS="sse2", GDK_BACKEND="x11",
                       GSK_RENDERER="cairo", MESA_SHADER_CACHE_DISABLE="true",
                       LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu",
                       LIBGL_DRIVERS_PATH="/usr/lib/x86_64-linux-gnu/dri",
                       XDG_CONFIG_HOME=str(temp / "config"))
            if host == "kitty":
                args = ["kitty", "--config", "NONE", "--override", "linux_display_server=x11",
                        "--override", "initial_window_width=800", "--override", "initial_window_height=600",
                        "--override", "remember_window_size=no", "--override", "background=#000000",
                        "--dump-bytes", str((output / "terminal.bin").resolve()), *program]
            elif host == "ghostty":
                args = ["ghostty", "--config-default-files=false", "--gtk-single-instance=false",
                        "--background=000000", "-e", *program]
            elif host == "gnome":
                env["GSETTINGS_BACKEND"] = "memory"
                args = ["dbus-run-session", "--", "gnome-terminal", "--wait", "--hide-menubar",
                        "--geometry=80x24", "--", *program]
            elif host == "xterm":
                args = ["xterm", "-xrm", "XTerm*decTerminalID: 340", "-bg", "black",
                        "-fg", "white", "-geometry", "80x24", "-e", *program]
            elif host == "wezterm":
                args = ["wezterm", "--skip-config", "start", "--always-new-process",
                        "--no-auto-connect", "--", *program]
            else:
                raise ValueError("expected kitty, ghostty, gnome, xterm or wezterm")
            with (output / "host.log").open("wb") as log:
                  child = start_owned(args, env=env, stdout=log, stderr=log)
            results = []
            for stage in range(3):
                pending = temp / "next"
                pending.write_text(str(stage))
                pending.replace(command)
                time.sleep(4)
                if child.poll() is not None:
                    raise RuntimeError(f"host exited early: {child.returncode}")
                screenshot = output / f"stage-{stage}.png"
                subprocess.run(["/usr/bin/import", "-window", "root", str(screenshot)], env=env, check=True, timeout=10)
                pixels = Image.open(screenshot).convert("RGB")
                measurements = {}
                if protocol and protocol.endswith("-full") and host == "xterm":
                    measurements["label_ink"] = {"count": sum(1 for y in range(20) for x in range(180)
                        if min(pixels.getpixel((x, y))) > 150)}
                for name, color in colors.items():
                    points = [(x, y) for y in range(pixels.height) for x in range(pixels.width)
                              if pixels.getpixel((x, y)) == color]
                    measurements[name] = {"count": len(points), "bounds":
                        [min(x for x, _ in points), min(y for _, y in points),
                         max(x for x, _ in points), max(y for _, y in points)] if points else None}
                if protocol and "behind" in protocol and stage < 2:
                    bounds = measurements["red" if stage == 0 else "green"]["bounds"]
                    if bounds:
                        region = pixels.crop((bounds[0], bounds[1], bounds[2] + 1, bounds[3] + 1))
                        counts = {color: count for count, color in region.getcolors(region.width * region.height)}
                        measurements["image_text"] = {"white": counts.get((255, 255, 255), 0),
                                                      "black": counts.get((0, 0, 0), 0)}
                if host == "gnome" and protocol is None:
                    # This fixture fixes an 80x24 default-profile window. Exclude
                    # the stage label, window border and scrollbar from the ink box.
                    points = [(x, y) for y in range(20, 400) for x in range(1, 700)
                              if min(pixels.getpixel((x, y))) > 100]
                    measurements["fallback_ink"] = {"count": len(points), "bounds":
                        [min(x for x, _ in points), min(y for _, y in points),
                         max(x for x, _ in points), max(y for _, y in points)] if points else None}
                results.append(measurements)
            pending = temp / "next"
            pending.write_text("3")
            pending.replace(command)
            child.wait(timeout=10)
            (output / "pixels.json").write_text(json.dumps(results, indent=2) + "\n")
            assert child.returncode == 0, child.returncode
            if host == "gnome" and protocol is None:
                first, moved, removed = [item["fallback_ink"] for item in results]
                assert first["count"] > 100 and moved["count"] > 50, results
                assert moved["bounds"][0] > first["bounds"][2], results
                assert moved["bounds"][1] > first["bounds"][3], results
                assert removed["count"] == 0, results
                print("gnome: fallback image ink, movement and removal PASS; decoded glyphs require visual review")
                return
            assert results[0]["red"]["count"] > 100 and results[0]["blue"]["count"] > 100, results[0]
            assert results[1]["green"]["count"] > 100 and results[1]["yellow"]["count"] > 100, results[1]
            assert results[1]["red"]["count"] == results[1]["blue"]["count"] == 0, results[1]
            if protocol and "behind" in protocol:
                assert all(r["image_text"]["white"] > 0 and r["image_text"]["black"] == 0
                           for r in results[:2]), "background image left a hole behind text"
            if protocol and protocol.endswith("-full"):
                assert results[0]["red"]["count"] > 20000 and results[1]["green"]["count"] > 20000, results
                assert results[0]["red"]["bounds"] == results[1]["green"]["bounds"], results
                if host == "xterm":
                    assert all(stage["label_ink"]["count"] > 20 for stage in results), "image output scrolled away the label"
            else:
                assert results[1]["green"]["bounds"][0] > results[0]["red"]["bounds"][2], results
            if protocol in ("app-chafa", "app-viu"):
                assert results[1]["green"]["count"] > results[0]["red"]["count"], "external image did not resize"
            assert all(results[2][name]["count"] == 0 for name in colors), results[2]
            change = "animated frame advance" if protocol and protocol.startswith("app-gif") else "source update"
            print(f"{host} ({protocol or 'App'}): real image pixels, {change}, movement and removal PASS")
        finally:
            stop(child)
            stop(server)
            os.close(reader)
            if writer is not None:
                os.close(writer)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--guard-session":
        raise SystemExit(guard_session(int(sys.argv[2]), sys.argv[3:]))
    run(sys.argv[1], Path(sys.argv[2]), sys.argv[3] if len(sys.argv) > 3 else None,
        sys.argv[4] if len(sys.argv) > 4 else None)
