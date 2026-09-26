#!/usr/bin/env python3
"""Run real clipboard tools on dedicated desktops and verify recorded coverage."""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import platform
import select
import shutil
import signal
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
RECORDS = ROOT / "target/evidence/clipboard-platforms"
PLATFORMS = {"wayland": "Linux", "xsel": "Linux", "xclip": "Linux", "macos": "Darwin", "windows": "Windows"}
INPUTS = (
    "Cargo.toml", "Cargo.lock", "build.rs", "src", "crates", "tests/api_clipboard.rs",
    "tests/clipboard_platform.rs", "tests/windows_platform.rs", "scripts/check-clipboard-platforms.py",
    ".github/workflows/clipboard-platforms.yml",
)


def committed_inputs():
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "--", *INPUTS], cwd=ROOT)
    clean = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", *INPUTS], cwd=ROOT)
    return not untracked and clean.returncode == 0


def input_digest(committed):
    digest = hashlib.sha256()
    command = ["git", "ls-tree", "-r", "--name-only", "HEAD", "--", *INPUTS] if committed else ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", *INPUTS]
    names = subprocess.check_output(command, cwd=ROOT, text=True).splitlines()
    for name in sorted(set(names)):
        # Git supplies canonical newlines across Windows and Unix checkouts.
        data = subprocess.check_output(["git", "show", "HEAD:" + name], cwd=ROOT) if committed else (ROOT / name).read_bytes()
        digest.update(name.encode() + b"\0" + len(data).to_bytes(8, "big") + data)
    return digest.hexdigest()


def captured(command, env, timeout=300):
    with tempfile.TemporaryFile() as output:
        child = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            status = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name == "posix":
                os.killpg(child.pid, signal.SIGKILL)
            else:
                child.kill()
            child.wait()
            raise RuntimeError("command exceeded its deadline: " + command[0]) from None
        output.seek(0)
        text = output.read().decode(errors="replace")
    if status:
        diagnostic = ROOT / "target" / "clipboard-platform-failure.log"
        diagnostic.parent.mkdir(parents=True, exist_ok=True)
        diagnostic.write_bytes(text.encode())
        raise RuntimeError(f"command exited {status}: {command}\n{text[-12000:]}")
    return text


def build_probe(env):
    text = captured(["cargo", "test", "--locked", "--test", "clipboard_platform", "--no-run", "--message-format=json"], env, timeout=900)
    for line in text.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") == "compiler-artifact" and message.get("target", {}).get("name") == "clipboard_platform" and message.get("executable"):
            return message["executable"]
    raise RuntimeError("Cargo did not report the clipboard platform executable")


def stop(child):
    if child.poll() is None:
        if os.name == "posix":
            os.killpg(child.pid, signal.SIGTERM)
        else:
            child.terminate()
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            if os.name == "posix":
                os.killpg(child.pid, signal.SIGKILL)
            else:
                child.kill()
            child.wait()


def stop_linux_probe_children(run_id):
    marker = b"RTUI_CLIPBOARD_PLATFORM_RUN=" + run_id.encode() + b"\0"
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if marker in (entry / "environ").read_bytes():
                os.kill(int(entry.name), signal.SIGKILL)
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            # Processes can exit while scanning; other users may hide environ.
            continue


@contextlib.contextmanager
def linux_desktop(backend, directory, base_env):
    env = dict(base_env)
    for name in ("DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS", "DBUS_STARTER_ADDRESS", "DBUS_STARTER_BUS_TYPE"):
        env.pop(name, None)
    runtime = directory / "runtime"
    runtime.mkdir(mode=0o700)
    env.update(XDG_RUNTIME_DIR=str(runtime), XDG_CONFIG_HOME=str(directory / "config"),
               XDG_DATA_HOME=str(directory / "data"), XDG_CACHE_HOME=str(directory / "cache"))
    with (directory / "desktop.log").open("wb") as log:
        if backend == "wayland":
            config = directory / "bus.conf"
            # No service activation: a private test compositor needs no desktop portals.
            config.write_text(f'<busconfig><type>session</type><listen>unix:tmpdir={runtime}</listen><policy context="default"><allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/></policy></busconfig>')
            env.update(QT_QPA_PLATFORM="offscreen", LIBGL_ALWAYS_SOFTWARE="1")
            child = subprocess.Popen(["/usr/bin/dbus-run-session", "--config-file=" + str(config), "--",
                                      "/usr/bin/kwin_wayland", "--virtual", "--no-lockscreen", "--no-global-shortcuts",
                                      "--no-kactivities", "--width", "320", "--height", "200", "--socket", "clipboard-test"],
                                     env=env, stdout=log, stderr=log, start_new_session=True)
            try:
                deadline = time.monotonic() + 15
                while not (runtime / "clipboard-test").exists():
                    if child.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("private Wayland desktop did not start")
                    time.sleep(0.05)
                env["WAYLAND_DISPLAY"] = "clipboard-test"
                yield env
            finally:
                stop(child)
        else:
            read_fd, write_fd = os.pipe()
            child = subprocess.Popen(["/usr/bin/Xvfb", "-displayfd", str(write_fd), "-screen", "0", "320x200x24", "-nolisten", "tcp", "-ac"],
                                     env=env, pass_fds=(write_fd,), stdout=log, stderr=log, start_new_session=True)
            os.close(write_fd)
            try:
                if not select.select([read_fd], [], [], 15)[0]:
                    raise RuntimeError("private X11 desktop did not start")
                display = os.read(read_fd, 128).decode().strip()
                if not display.isdigit():
                    raise RuntimeError("Xvfb did not return a display number")
                env["DISPLAY"] = ":" + display
                yield env
            finally:
                os.close(read_fd)
                stop(child)


def run_backend(args):
    if platform.system() != PLATFORMS[args.backend]:
        raise RuntimeError(f"{args.backend} requires a real {PLATFORMS[args.backend]} runner")
    committed = committed_inputs()
    if not committed and not args.allow_dirty:
        raise RuntimeError("commit the clipboard inputs before recording platform evidence")
    if args.backend in ("macos", "windows") and not args.dedicated_desktop:
        raise RuntimeError("use --dedicated-desktop only on a disposable test desktop; the probe replaces its clipboard")
    initial_digest = input_digest(committed)
    env = {**os.environ, "CARGO_INCREMENTAL": "0"}
    executable = build_probe(env)
    process_output = captured(["cargo", "test", "--locked", "--lib", "hooks::clipboard_process::tests", "--", "--test-threads=1"], env)
    if "2 passed" not in process_output:
        raise RuntimeError("native process lifecycle checks did not execute")
    if args.backend == "windows":
        adapter = captured(["cargo", "test", "--locked", "--test", "windows_platform", "--", "--nocapture"], env)
        if "RTUI_WINDOWS_ADAPTER_OK" not in adapter or "1 passed" not in adapter:
            raise RuntimeError("native Windows adapter checks did not execute")
        process_output += "\n" + adapter
    run_id = str(uuid.uuid4())
    env.update(RTUI_CLIPBOARD_DEDICATED_SESSION="1", RTUI_CLIPBOARD_EXPECT_BACKEND=args.backend, RTUI_CLIPBOARD_PLATFORM_RUN=run_id)
    command = [executable, "--ignored", "--exact", "clipboard_platform_roundtrip", "--nocapture"]
    tools = {"wayland": ["wl-copy", "wl-paste"], "xsel": ["xsel"], "xclip": ["xclip"], "macos": ["pbcopy", "pbpaste"], "windows": ["powershell"]}[args.backend]
    tool_paths = {name: shutil.which(name) for name in tools}
    if not all(tool_paths.values()):
        raise RuntimeError("missing native clipboard tool: " + repr(tool_paths))
    with tempfile.TemporaryDirectory(prefix="clipboard-platform-", dir=ROOT / "target") as scratch:
        directory = Path(scratch)
        try:
            if platform.system() == "Linux":
                with linux_desktop(args.backend, directory, env) as desktop_env:
                    command_path = directory / "bin"
                    command_path.mkdir()
                    for name, path in tool_paths.items():
                        (command_path / name).symlink_to(path)
                    # wl-copy uses cat internally when reading redirected stdin.
                    (command_path / "cat").symlink_to("/usr/bin/cat")
                    desktop_env["PATH"] = str(command_path)
                    output = captured(command, desktop_env, timeout=30)
            else:
                output = captured(command, env, timeout=30)
        finally:
            if platform.system() == "Linux":
                stop_linux_probe_children(run_id)
    marker = f"RTUI_CLIPBOARD_PLATFORM_OK backend={args.backend} cases=5"
    if marker not in output or "1 passed" not in output:
        raise RuntimeError("platform round-trip cases did not execute")
    if initial_digest != input_digest(committed) or (committed and not committed_inputs()):
        raise RuntimeError("clipboard inputs changed during the platform run")
    args.output.mkdir(parents=True, exist_ok=True)
    out_name = args.backend + ".out"
    process_name = args.backend + ".process.out"
    (args.output / out_name).write_bytes(output.encode())
    (args.output / process_name).write_bytes(process_output.encode())
    record = {
        "backend": args.backend, "system": platform.system(), "platform": platform.platform(),
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "committed_inputs": committed, "inputs_digest": initial_digest, "result": "pass",
        "output": out_name, "output_digest": hashlib.sha256(output.encode()).hexdigest(),
        "process_output": process_name, "process_output_digest": hashlib.sha256(process_output.encode()).hexdigest(),
        "tools": {name: {"path": path, "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest()} for name, path in tool_paths.items()},
    }
    (args.output / (args.backend + ".json")).write_text(json.dumps(record, indent=2) + "\n")
    print(f"{args.backend}: five native round trips and two process lifecycle checks passed; committed inputs: {committed}")


def verify():
    if not committed_inputs():
        raise RuntimeError("clipboard platform inputs are not committed")
    digest = input_digest(True)
    for backend, system in PLATFORMS.items():
        path = RECORDS / (backend + ".json")
        if not path.exists():
            raise RuntimeError(f"missing real {system} clipboard evidence for {backend}")
        record = json.loads(path.read_text())
        if record.get("backend") != backend or record.get("system") != system or record.get("result") != "pass" or not record.get("committed_inputs") or record.get("inputs_digest") != digest:
            raise RuntimeError("stale or invalid platform evidence: " + backend)
        for key in ("output", "process_output"):
            name = record[key]
            if Path(name).name != name:
                raise RuntimeError("platform output must be a local filename")
            data = (RECORDS / name).read_bytes()
            if hashlib.sha256(data).hexdigest() != record[key + "_digest"]:
                raise RuntimeError("damaged platform output: " + backend)
        if f"RTUI_CLIPBOARD_PLATFORM_OK backend={backend} cases=5" not in (RECORDS / record["output"]).read_text() or "2 passed" not in (RECORDS / record["process_output"]).read_text():
            raise RuntimeError("platform output lacks executed behavior checks: " + backend)
        if backend == "windows" and "RTUI_WINDOWS_ADAPTER_OK" not in (RECORDS / record["process_output"]).read_text():
            raise RuntimeError("Windows adapter checks are missing")
    print("All five clipboard backends have current native platform evidence")


if __name__ == "__main__":
    os.chdir(ROOT)
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--backend", choices=PLATFORMS)
    mode.add_argument("--verify", action="store_true")
    parser.add_argument("--output", type=Path, default=RECORDS)
    parser.add_argument("--allow-dirty", action="store_true", help="development probe only; its record cannot satisfy acceptance")
    parser.add_argument("--dedicated-desktop", action="store_true")
    arguments = parser.parse_args()
    if arguments.verify:
        verify()
    else:
        run_backend(arguments)
