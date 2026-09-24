#!/usr/bin/env python3
"""Compile public Rust/C consumers and exercise their real terminal sessions."""
import errno
import fcntl
import json
import os
from pathlib import Path
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "verification/api-entry-points"
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()


def execute(command, timeout=180):
    print("+", " ".join(map(str, command)), flush=True)
    subprocess.run(command, cwd=ROOT, check=True, timeout=timeout)


def cargo_library_artifacts(command):
    """Use Cargo's selected artifacts, including fresh dependencies, rather than stale glob matches."""
    command = [*command, "--message-format=json-render-diagnostics"]
    print("+", " ".join(map(str, command)), flush=True)
    result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, text=True, timeout=180)
    libraries = {}
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            print(line, flush=True)
            continue
        if message.get("reason") == "compiler-artifact":
            for name in message.get("filenames", []):
                if name.endswith(".rlib"):
                    libraries.setdefault(message["target"]["name"], set()).add(Path(name))
        elif message.get("reason") == "compiler-message":
            print(message["message"].get("rendered") or line, flush=True)
    result.check_returncode()
    return libraries


class Terminal:
    def __init__(self, command, screen_reader, capture, size):
        self.master, self.slave = os.openpty()
        self.child = None
        self.output = bytearray()
        self.capture = capture
        self.screen_reader = screen_reader
        self.size = size
        try:
            os.set_blocking(self.master, False)
            self.resize(size)
            self.original = termios.tcgetattr(self.slave)
            # A controlling terminal is required by the public DirectTty route.
            launcher = ("import os,fcntl,termios,sys; os.setsid(); "
                        "fcntl.ioctl(0,termios.TIOCSCTTY,0); os.execv(sys.argv[1],sys.argv[1:])")
            env = {**os.environ, "TERM": "xterm-256color", "RUST_BACKTRACE": "0"}
            for name in ("DBUS_SESSION_BUS_ADDRESS", "AT_SPI_BUS_ADDRESS", "DBUS_STARTER_ADDRESS",
                         "TERM_PROGRAM", "KITTY_WINDOW_ID", "WEZTERM_PANE", "WT_SESSION"):
                env.pop(name, None)
            self.child = subprocess.Popen(
                [sys.executable, "-B", "-c", launcher, *map(str, command)],
                stdin=self.slave, stdout=self.slave, stderr=self.slave, env=env,
            )
        except BaseException:
            self.close()
            raise

    def resize(self, size):
        self.size = size
        width, height = size
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))
        if self.child is not None:
            os.kill(self.child.pid, signal.SIGWINCH)

    def read(self, timeout=0.05):
        if not select.select([self.master], [], [], timeout)[0]:
            return False
        try:
            data = os.read(self.master, 65536)
        except BlockingIOError:
            return False
        if not data:
            return False
        self.output.extend(data)
        assert len(self.output) < 8 * 1024 * 1024, "entry-point output exceeded 8 MiB"
        return True

    def screen(self):
        self.capture.write_bytes(self.output)
        width, height = self.size
        return subprocess.check_output(
            [str(self.screen_reader), str(self.capture), str(height), str(width)],
            text=True, timeout=5,
        )

    def wait_for(self, text, complete=lambda screen: True):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            self.read()
            screen = self.screen()
            if text in screen and complete(screen):
                return screen
            if self.child.poll() is not None:
                break
        raise AssertionError(f"entry point did not paint {text!r}; screen={self.screen()!r}; "
                             f"output={bytes(self.output[-2000:])!r}")

    def send(self, data):
        assert os.write(self.master, data) == len(data)

    def wait_marker(self, name):
        marker = f"\x1b]900;{name}\x07".encode()
        deadline = time.monotonic() + 8
        while marker not in self.output and time.monotonic() < deadline:
            self.read()
            if self.child.poll() is not None:
                break
        assert marker in self.output, (name, bytes(self.output[-2000:]))

    def finish(self, expected_error=False, build_error=False, host_modes=True):
        deadline = time.monotonic() + 8
        while self.child.poll() is None and time.monotonic() < deadline:
            self.read()
        assert self.child.poll() is not None, "entry point did not exit"
        drain_deadline = time.monotonic() + 2
        while self.read(0):
            assert time.monotonic() < drain_deadline, "entry-point output did not stop after exit"
        self.capture.write_bytes(self.output)
        expected = 1 if expected_error else 0
        assert self.child.returncode == expected, (self.child.returncode, expected, bytes(self.output[-2000:]))
        marker = (b"entry point controlled root error" if expected_error else
                  b"ENTRY_POINT_BUILD_ERROR" if build_error else b"ENTRY_POINT_CLEAN_EXIT")
        assert marker in self.output, bytes(self.output[-2000:])
        try:
            restored = termios.tcgetattr(self.slave)
        except termios.error as error:
            if sys.platform != "darwin" or error.args[0] != errno.ENOTTY:
                raise
            # Darwin revokes the slave when its controlling session exits. The
            # open master retains the same tty and its actual termios settings.
            restored = termios.tcgetattr(self.master)
        assert restored == self.original, "entry point left raw mode active"
        if host_modes:
            assert b"\x1b[?1049l" in self.output, "entry point did not leave alternate screen"
            assert b"\x1b[?25h" in self.output, "entry point did not restore the cursor"

    def close(self):
        if self.child is not None and self.child.poll() is None:
            os.killpg(self.child.pid, signal.SIGKILL)
            self.child.wait(timeout=5)
        os.close(self.master)
        os.close(self.slave)
        self.capture.write_bytes(self.output)


def host_workflow(binary, screen, directory, route, size, error):
    capture = directory / f"rust-{route}-{size[0]}x{size[1]}-{'error' if error else 'exit'}.bin"
    terminal = Terminal([binary, route], screen, capture, size)
    try:
        painted = terminal.wait_for("state alpha clicks0", lambda frame: "South" in frame)
        assert "e\u0301界" in painted, f"entry point split Unicode clusters: {painted!r}"
        # vt100 does not compose ZWJ emoji: subsequent cell addressing overwrites
        # its extra laptop cell. Require the intact emitted cluster separately.
        assert "👩\u200d💻".encode() in terminal.output, "entry point split the emitted ZWJ cluster"
        terminal.send(b"\x1b[<0;2;2M\x1b[<0;2;2m")
        terminal.wait_for("state alpha clicks1")
        terminal.send(b"x")
        terminal.wait_for("state omega clicks1")
        terminal.send(b"r")
        painted = terminal.wait_for("state reversed clicks1", lambda frame:
                                    "North" in frame and "South" in frame and
                                    frame.index("South") < frame.index("North"))
        assert painted.index("South") < painted.index("North"), painted
        resized = (48, 12) if size == (32, 8) else (32, 8)
        terminal.resize(resized)
        terminal.wait_for(f"size {resized[0]} {resized[1]}")
        terminal.send(b"e")
        painted = terminal.wait_for("state empty clicks1", lambda frame:
                                    "North" not in frame and "South" not in frame and "界" not in frame)
        assert "North" not in painted and "South" not in painted and "界" not in painted, painted
        terminal.send(b"f" if error else b"\x03")
        terminal.finish(expected_error=error)
    finally:
        terminal.close()
    print(f"PASS Rust {route} {size}: paint, input, Unicode, update, reorder, removal, resize, "
          f"{'error' if error else 'exit'} and restoration", flush=True)


def native_workflow(binary, screen, directory, size, missing_root=False, reselect=False):
    mode = "missing-root" if missing_root else "reselect" if reselect else "app"
    terminal = Terminal([binary, mode], screen, directory / f"c-{mode}-{size[0]}.bin", size)
    try:
        if not missing_root:
            painted = terminal.wait_for(f"NATIVE size {size[0]} {size[1]}", lambda frame: "e\u0301界" in frame)
            assert "e\u0301界" in painted, painted
            resized = (48, 12) if size == (32, 8) else (32, 8)
            terminal.resize(resized)
            terminal.wait_for(f"NATIVE size {resized[0]} {resized[1]}")
            terminal.send(b"\x03")
        terminal.finish(build_error=missing_root)
    finally:
        terminal.close()
    print(f"PASS C SuprTUI {size}: {mode}, callback ownership and restoration", flush=True)


def manual_workflow(binary, screen, directory, route):
    terminal = Terminal([binary, route], screen, directory / f"manual-{route}.bin", (32, 8))
    try:
        if route == "direct":
            terminal.wait_marker("READY")
            terminal.send(b"xyz")  # One native read must retain all three events.
        elif route == "unix":
            for index in range(2):
                terminal.wait_marker(f"RAW{index}")
                mode = termios.tcgetattr(terminal.slave)
                assert not mode[3] & (termios.ICANON | termios.ECHO), "clone did not retain raw mode"
                terminal.send(b"x")
                terminal.wait_marker(f"CLOSED{index}")
                assert termios.tcgetattr(terminal.slave) == terminal.original, "last clone did not restore mode"
                terminal.send(b"\n")
        terminal.finish(host_modes=route != "unix")
        if route != "unix":
            output = bytes(terminal.output)
            def part(name):
                return f"\x1b]900;{name}\x07".encode()
            def segment(first, last):
                return output.split(part(first), 1)[1].split(part(last), 1)[0]
            def painted(name):
                terminal.capture.write_bytes(output.split(part(name), 1)[0])
                return subprocess.check_output([str(screen), str(terminal.capture), "8", "32"], text=True, timeout=5)
            assert "FIRST" in painted("FIRST")
            assert not segment("FIRST", "UNCHANGED"), "unchanged manual frame emitted output"
            assert "SECOND" in painted("SECOND") and "FIRST" not in painted("SECOND")
            assert not painted("REMOVED").strip(), "root removal left old cells"
            if route == "crossterm":
                assert not segment("BASE", "INCREMENTAL")
                assert segment("INCREMENTAL", "FULL"), "disabled output optimization had no effect"
                assert painted("BASE") == painted("FULL")
                assert not segment("FULL", "INCREMENTAL_AGAIN")
                assert "frame:" in painted("DEBUG") and "frame:" not in painted("NO_DEBUG")
    finally:
        terminal.close()
    print(f"PASS manual {route}: patches/options, input batch or clone ownership, cleanup", flush=True)


def queue_input_burst(terminal, release):
    payload = b"a" * 2048
    try:
        queued = os.write(terminal.master, payload)
    except BlockingIOError:
        queued = 0
    if sys.platform.startswith("linux"):
        assert queued == len(payload), f"Linux queued-burst control accepted only {queued} bytes"
    delivery = "fully-queued" if queued == len(payload) else "streamed"
    # Publish the initial count atomically: the consumer must not see a partial record.
    pending = release.with_suffix(".pending")
    pending.write_text(str(queued))
    pending.replace(release)
    sent = queued
    deadline = time.monotonic() + 2
    while sent < len(payload):
        assert time.monotonic() < deadline, f"input producer stalled after {sent} bytes"
        assert terminal.child.poll() is None, "burst consumer exited before all input was sent"
        if not select.select([], [terminal.master], [], max(0, min(0.05, deadline - time.monotonic())))[1]:
            continue
        try:
            sent += os.write(terminal.master, payload[sent:])
        except BlockingIOError:
            continue
    print(f"BURST_DELIVERY {delivery} initially_queued={queued} total_sent={sent}", flush=True)
    return queued, delivery


def input_burst_workflow(binary, screen, directory, release):
    terminal = Terminal([binary, release], screen, directory / "crossterm-input-burst.bin", (32, 8))
    try:
        terminal.wait_marker("READY")
        # Linux retains the strict >1024-byte queued defect probe. Smaller Unix PTYs
        # stream the remaining bytes after release without lowering the required count.
        queued, delivery = queue_input_burst(terminal, release)
        terminal.finish(host_modes=False)
        assert f"BURST_DELIVERY {delivery} initially_queued={queued}".encode() in terminal.output
        assert b"COUNT 2048" in terminal.output, bytes(terminal.output[-2000:])
        assert b"ZERO_POLL_US " in terminal.output, bytes(terminal.output[-2000:])
    finally:
        terminal.close()
    print("PASS public Crossterm: 2048 bytes, exhausted zero-timeout poll and restoration", flush=True)


def main():
    os.chdir(ROOT)
    execute([sys.executable, "-B", str(SOURCE / "check-harness.py")], timeout=35)
    execute(["cargo", "test", "--locked", "--manifest-path",
             "crates/reactive-tui-crossterm/Cargo.toml", "--lib", "--features",
             "event-stream", "readiness_tests", "--", "--test-threads=1"])
    # Build default test dependencies first: a later non-FFI build replaces the
    # shared library in debug/deps, which Darwin records as its install name.
    execute(["cargo", "test", "--locked", "--test", "suprtui_renderer", "--no-run"])
    libraries = cargo_library_artifacts(["cargo", "build", "--locked", "--features", "ffi"])
    crossterm_artifacts = libraries.get("crossterm", set())
    assert len(crossterm_artifacts) == 1, f"expected one selected Crossterm library: {crossterm_artifacts}"
    crossterm = next(iter(crossterm_artifacts))
    artifacts = list((TARGET / "debug/deps").glob("libvt100-*.rlib"))
    assert artifacts, "Cargo did not build the declared vt100 dependency"
    vt100 = max(artifacts, key=lambda path: path.stat().st_mtime_ns)
    captured = ROOT / "target/evidence/api-entry-points" / time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    captured.mkdir(parents=True)
    failures = []
    with tempfile.TemporaryDirectory(prefix="api-entry-points-", dir=TARGET) as temporary:
        directory = Path(temporary)
        for name in ("legacy", "host", "manual", "screen", "input-burst"):
            dependency = ("crossterm=" + str(crossterm) if name == "input-burst" else
                          "vt100=" + str(vt100) if name == "screen" else
                          "reactive_tui=" + str(TARGET / "debug/libreactive_tui.rlib"))
            command = ["rustc", "--edition=2021", str(SOURCE / f"{name}.rs"), "--extern", dependency,
                       "-L", "dependency=" + str(TARGET / "debug/deps"), "-o", str(directory / name)]
            if name == "legacy":
                command.append("--test")
            execute(command)
        def attempt(name, function):
            try:
                function()
            except (AssertionError, OSError, subprocess.SubprocessError) as error:
                failures.append(name)
                print(f"FAIL {name}: {error}", flush=True)
        attempt("legacy App state/errors", lambda: execute([str(directory / "legacy"), "--test-threads=1"]))
        for route in ("suprtui", "crossterm", "crossterm-surface", "direct"):
            for size, error in (((32, 8), False), ((48, 12), True)):
                attempt(f"Rust {route} {size}", lambda route=route, size=size, error=error:
                        host_workflow(directory / "host", directory / "screen", captured, route, size, error))
        for route in ("crossterm", "direct", "unix"):
            attempt(f"manual {route}", lambda route=route:
                    manual_workflow(directory / "manual", directory / "screen", captured, route))
        attempt("public Crossterm queued input burst", lambda:
                input_burst_workflow(directory / "input-burst", directory / "screen", captured,
                                     directory / "release-input-burst"))
        native = directory / "native"
        command = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", "-Iinclude", str(SOURCE / "native.c"),
                   "-L" + str(TARGET / "debug"), "-Wl,-rpath," + str(TARGET / "debug"),
                   "-lreactive_tui", "-o", str(native)]
        attempt("C SuprTUI construction compiles and links", lambda: execute(command))
        if native.exists():
            for size in ((32, 8), (48, 12)):
                attempt(f"C SuprTUI {size}", lambda size=size:
                        native_workflow(native, directory / "screen", captured, size))
            attempt("C missing root restores terminal", lambda:
                    native_workflow(native, directory / "screen", captured, (32, 8), missing_root=True))
            attempt("C terminal re-selection retains ownership", lambda:
                    native_workflow(native, directory / "screen", captured, (32, 8), reselect=True))
    print("Terminal captures:", captured, flush=True)
    if failures:
        raise SystemExit("Entry-point failures: " + ", ".join(failures))
    print("All public entry-point workflows passed", flush=True)


if __name__ == "__main__":
    main()
