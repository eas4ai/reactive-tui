#!/usr/bin/env python3
"""Run the focused pre-release C ABI safety consumers."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()


def build_jobs() -> str:
    """Keep builds within the workstation's agreed concurrency limit."""
    try:
        return str(min(8, max(1, int(os.environ.get("CARGO_BUILD_JOBS", "8")))))
    except ValueError:
        return "8"


def run_ffs_001() -> None:
    consumer = ROOT / "tests/pre_release_ffi_app_lifetime.c"
    if not consumer.is_file():
        raise RuntimeError(f"missing focused FFS-001 consumer: {consumer.relative_to(ROOT)}")

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    subprocess.run(
        ["cargo", "build", "--locked", "--features", "ffi", "--jobs", env["CARGO_BUILD_JOBS"]],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )

    with tempfile.TemporaryDirectory(prefix="ffs-001-", dir=TARGET) as scratch:
        binary = Path(scratch) / "ffi-app-lifetime"
        subprocess.run(
            [
                "clang",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic-errors",
                "-fsanitize=address,undefined",
                "-fno-omit-frame-pointer",
                "-pthread",
                "-Iinclude",
                str(consumer),
                f"-L{TARGET / 'debug'}",
                f"-Wl,-rpath,{TARGET / 'debug'}",
                "-lreactive_tui",
                "-o",
                str(binary),
            ],
            cwd=ROOT,
            env=env,
            check=True,
            timeout=60,
        )
        subprocess.run(
            [str(binary)],
            cwd=ROOT,
            env={**env, "ASAN_OPTIONS": "detect_leaks=1:halt_on_error=1"},
            check=True,
            timeout=30,
        )


def run_ffs_002() -> None:
    consumer = ROOT / "tests/pre_release_ffi_text_buffer.c"
    inventory = ROOT / "tests/ffi_export_inventory.rs"
    for required in (consumer, inventory):
        if not required.is_file():
            raise RuntimeError(f"missing focused FFS-002 input: {required.relative_to(ROOT)}")

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--features",
            "ffi",
            "--test",
            "ffi_export_inventory",
            "--jobs",
            env["CARGO_BUILD_JOBS"],
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    subprocess.run(
        ["cargo", "build", "--locked", "--features", "ffi", "--jobs", env["CARGO_BUILD_JOBS"]],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )

    with tempfile.TemporaryDirectory(prefix="ffs-002-", dir=TARGET) as scratch:
        binary = Path(scratch) / "ffi-text-buffer"
        subprocess.run(
            [
                "clang",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic-errors",
                "-fsanitize=address,undefined",
                "-fno-omit-frame-pointer",
                "-Iinclude",
                str(consumer),
                f"-L{TARGET / 'debug'}",
                f"-Wl,-rpath,{TARGET / 'debug'}",
                "-lreactive_tui",
                "-o",
                str(binary),
            ],
            cwd=ROOT,
            env=env,
            check=True,
            timeout=60,
        )
        subprocess.run(
            [str(binary)],
            cwd=ROOT,
            env={**env, "ASAN_OPTIONS": "detect_leaks=1:halt_on_error=1"},
            check=True,
            timeout=30,
        )


def main() -> int:
    requested = sys.argv[1:]
    if requested == ["FFS-001"]:
        run_ffs_001()
    elif requested == ["FFS-002"]:
        run_ffs_002()
    else:
        raise SystemExit("usage: check-pre-release-ffi-safety.py FFS-001|FFS-002")
    print(f"{requested[0]} focused C ABI safety checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
