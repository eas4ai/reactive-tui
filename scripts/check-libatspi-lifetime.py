#!/usr/bin/env python3
"""Build and verify the pinned, fixture-only libatspi lifetime repair."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "verification" / "libatspi"
TARGET = ROOT / "target" / "libatspi-lifetime"
METADATA = json.loads((FIXTURE / "source.json").read_text())
# At most 8 parallel jobs: the build shares the machine with other work.
JOBS = min(8, max(1, int(os.environ.get("CARGO_BUILD_JOBS", "8"))))


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def run(command: list[str], *, cwd: Path | None = None,
        env: dict[str, str] | None = None, timeout: int = 600,
        capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command, cwd=cwd or ROOT, env=env, check=False, timeout=timeout,
        text=True, stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def require(command: list[str], *, cwd: Path | None = None,
            env: dict[str, str] | None = None, timeout: int = 600) -> None:
    result = run(command, cwd=cwd, env=env, timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"command failed with exit {result.returncode}: {command}")


def download_archive(archive: Path) -> None:
    if archive.exists() and sha256(archive) == METADATA["sha256"]:
        return
    archive.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=archive.parent, delete=False) as output:
        temporary = Path(output.name)
        with urllib.request.urlopen(METADATA["url"], timeout=60) as response:
            shutil.copyfileobj(response, output)
    if sha256(temporary) != METADATA["sha256"]:
        temporary.unlink()
        raise RuntimeError("downloaded libatspi source archive has the wrong SHA-256")
    os.replace(temporary, archive)


def extract_archive(archive: Path, destination: Path) -> None:
    staging = destination.with_name(destination.name + "-extract")
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    with tarfile.open(archive, "r:xz") as source:
        source.extractall(staging, filter="data")
    extracted = staging / f"at-spi2-core-{METADATA['version']}"
    if not extracted.is_dir():
        raise RuntimeError("pinned archive did not contain the expected source directory")
    os.replace(extracted, destination)
    staging.rmdir()


def configure_and_build(source: Path, build: Path, *, sanitize: bool) -> Path:
    environment = os.environ.copy()
    environment["CFLAGS"] = "-O1 -g -fno-omit-frame-pointer"
    setup = [
        "meson", "setup", str(build), str(source), "--wrap-mode=nodownload",
        "--buildtype=debugoptimized", "-Dgtk2_atk_adaptor=false",
        "-Ddocs=false", "-Dintrospection=disabled", "-Dx11=enabled",
        "-Ddbus_glib=disabled", "-Ddefault_bus=dbus-daemon", "-Duse_systemd=false",
    ]
    if sanitize:
        setup.append("-Db_sanitize=address")
    require(setup, env=environment)
    require(["ninja", "-C", str(build), "-j", str(JOBS),
             "atspi/libatspi.so.0.0.1"], env=environment)
    candidates = sorted((build / "atspi").glob("libatspi.so.0*"))
    libraries = [path for path in candidates if path.is_file()]
    if not libraries:
        raise RuntimeError("isolated build did not produce libatspi.so.0")
    return libraries[0]


def compile_reproducer(source: Path, build: Path, output: Path) -> None:
    cflags = subprocess.check_output(
        ["pkg-config", "--cflags", "glib-2.0", "gobject-2.0", "dbus-1"],
        text=True,
    ).split()
    libraries = subprocess.check_output(
        ["pkg-config", "--libs", "glib-2.0", "gobject-2.0", "dbus-1"],
        text=True,
    ).split()
    require([
        "cc", "-std=c11", "-O1", "-g", "-fno-omit-frame-pointer",
        "-fsanitize=address", "-Wl,--export-dynamic", "-o", str(output),
        str(FIXTURE / "reproducer.c"), "-I", str(source),
        "-I", str(source / "atspi"), "-I", str(build),
        "-I", str(build / "atspi"), *cflags, "-L", str(build / "atspi"),
        f"-Wl,-rpath,{build / 'atspi'}", "-latspi", "-ldl", *libraries,
    ])


def exported_symbols(library: Path) -> set[tuple[str, str]]:
    output = subprocess.check_output(
        ["nm", "-D", "--defined-only", str(library)], text=True,
    )
    symbols = set()
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 3 and not fields[-1].startswith("__odr_asan."):
            symbols.add((fields[-2], fields[-1]))
    return symbols


def build_inputs_digest() -> str:
    digest = hashlib.sha256()
    for path in [Path(__file__), FIXTURE / "source.json",
                 FIXTURE / "state-set-lifetime.patch", FIXTURE / "reproducer.c"]:
        digest.update(path.name.encode())
        digest.update(path.read_bytes())
    for command in (["cc", "--version"], ["meson", "--version"], ["ninja", "--version"]):
        digest.update(subprocess.check_output(command))
    return digest.hexdigest()


def prepare() -> tuple[Path, Path, Path, Path]:
    TARGET.mkdir(parents=True, exist_ok=True)
    archive = TARGET / METADATA["archive"]
    download_archive(archive)
    fingerprint = build_inputs_digest()
    stamp = TARGET / "build.sha256"
    original_source = TARGET / "source-original"
    fixed_source = TARGET / "source-fixed"
    original_build = TARGET / "build-original"
    fixed_build = TARGET / "build-fixed"
    runtime_build = TARGET / "build-runtime"
    original_reproducer = TARGET / "reproducer-original"
    fixed_reproducer = TARGET / "reproducer-fixed"
    expected = [original_build / "atspi" / "libatspi.so.0",
                fixed_build / "atspi" / "libatspi.so.0",
                runtime_build / "atspi" / "libatspi.so.0",
                original_reproducer, fixed_reproducer]
    if not stamp.exists() or stamp.read_text().strip() != fingerprint or not all(
            path.exists() for path in expected):
        for path in [original_source, fixed_source, original_build, fixed_build, runtime_build,
                     original_reproducer, fixed_reproducer]:
            if path.is_dir():
                shutil.rmtree(path)
            elif path.exists():
                path.unlink()
        extract_archive(archive, original_source)
        shutil.copytree(original_source, fixed_source, symlinks=True)
        require(["patch", "--batch", "--forward", "--fuzz=0", "-p1", "-i",
                 str(FIXTURE / "state-set-lifetime.patch")], cwd=fixed_source)
        original_library = configure_and_build(original_source, original_build, sanitize=True)
        fixed_library = configure_and_build(fixed_source, fixed_build, sanitize=True)
        runtime_library = configure_and_build(fixed_source, runtime_build, sanitize=False)
        compile_reproducer(original_source, original_build, original_reproducer)
        compile_reproducer(fixed_source, fixed_build, fixed_reproducer)
        if not (exported_symbols(original_library) == exported_symbols(fixed_library) ==
                exported_symbols(runtime_library)):
            raise RuntimeError("lifetime patch changed the libatspi dynamic symbol surface")
        stamp.write_text(fingerprint + "\n")
    return original_build, fixed_build, runtime_build, original_reproducer, fixed_reproducer


def execute_reproducer(binary: Path, build: Path, mode: str) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    library_dir = str((build / "atspi").resolve())
    environment["LD_LIBRARY_PATH"] = library_dir + (
        ":" + environment["LD_LIBRARY_PATH"] if environment.get("LD_LIBRARY_PATH") else "")
    environment["RTUI_EXPECTED_LIBATSPI_DIR"] = library_dir + "/"
    environment["ASAN_OPTIONS"] = "detect_leaks=0:halt_on_error=1:abort_on_error=1"
    return run([str(binary), mode], env=environment, capture=True, timeout=30)


def system_library() -> Path:
    directory = Path(subprocess.check_output(
        ["pkg-config", "--variable=libdir", "atspi-2"], text=True,
    ).strip())
    return (directory / "libatspi.so.0").resolve()


def verify_lifetime(record_review: bool) -> None:
    installed = system_library()
    installed_before = sha256(installed)
    original_build, fixed_build, runtime_build, original_binary, fixed_binary = prepare()
    results: dict[str, dict[str, object]] = {}
    for mode in ["contains", "get-states"]:
        original = execute_reproducer(original_binary, original_build, mode)
        original_text = original.stdout + original.stderr
        if original.returncode == 0 or "heap-use-after-free" not in original_text.lower():
            raise RuntimeError(f"unpatched {mode} did not demonstrate the state-set use-after-free")
        fixed = execute_reproducer(fixed_binary, fixed_build, mode)
        if fixed.returncode or "AddressSanitizer" in fixed.stderr or f"PASS {mode}" not in fixed.stdout:
            raise RuntimeError(f"patched {mode} did not pass cleanly under AddressSanitizer")
        runtime = execute_reproducer(fixed_binary, runtime_build, mode)
        if runtime.returncode or f"PASS {mode}" not in runtime.stdout:
            raise RuntimeError(f"fixture runtime {mode} did not load and retain the repaired library")
        results[mode] = {
            "original_exit": original.returncode,
            "original_stdout": original.stdout,
            "original_stderr": original.stderr,
            "fixed_exit": fixed.returncode,
            "fixed_stdout": fixed.stdout,
            "fixed_stderr": fixed.stderr,
            "runtime_exit": runtime.returncode,
            "runtime_stdout": runtime.stdout,
            "runtime_stderr": runtime.stderr,
        }
        print(f"PASS unpatched {mode} exposes the state-set use-after-free", flush=True)
        print(f"PASS patched {mode} retains and releases the state set under AddressSanitizer", flush=True)
        print(f"PASS fixture runtime {mode} resolves the isolated repaired library", flush=True)

    original_library = (original_build / "atspi" / "libatspi.so.0").resolve()
    fixed_library = (fixed_build / "atspi" / "libatspi.so.0").resolve()
    runtime_library = (runtime_build / "atspi" / "libatspi.so.0").resolve()
    if not (exported_symbols(original_library) == exported_symbols(fixed_library) ==
            exported_symbols(runtime_library)):
        raise RuntimeError("patched and unpatched libatspi exports differ")
    if sha256(installed) != installed_before:
        raise RuntimeError("the installed libatspi changed during isolated verification")
    asan_runtime = Path(subprocess.check_output(
        ["cc", "-print-file-name=libasan.so"], text=True,
    ).strip()).resolve()
    if not asan_runtime.is_file():
        raise RuntimeError("compiler AddressSanitizer runtime was not found")
    environment_record = {
        "version": METADATA["version"],
        "archive_sha256": METADATA["sha256"],
        "library_dir": str(runtime_library.parent),
        "asan_runtime": str(asan_runtime),
        "system_library": str(installed),
        "system_sha256": installed_before,
        "dynamic_symbols_equal": True,
    }
    (TARGET / "environment.json").write_text(json.dumps(environment_record, indent=2) + "\n")
    if record_review:
        review = ROOT / "target" / "evidence" / "libatspi-lifetime"
        review.mkdir(parents=True, exist_ok=True)
        (review / "result.json").write_text(json.dumps({
            "source": METADATA,
            "environment": environment_record,
            "reproducers": results,
        }, indent=2) + "\n")
    print("PASS patched and original libatspi dynamic exports are identical", flush=True)
    print("PASS installed desktop libatspi remained byte-for-byte unchanged", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--record-review", action="store_true")
    arguments = parser.parse_args()
    verify_lifetime(arguments.record_review)
