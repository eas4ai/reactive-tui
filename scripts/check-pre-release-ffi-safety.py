#!/usr/bin/env python3
"""Run the focused pre-release C ABI safety consumers."""

from __future__ import annotations

import os
from pathlib import Path
import re
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


def run_ffs_001_rust_checks(env: dict[str, str]) -> None:
    """Prove the Rust ownership and callback representations."""
    for target in (["--test", "ffi_app_representation"], ["--doc"]):
        subprocess.run(
            [
                "cargo",
                "test",
                "--locked",
                "--features",
                "ffi",
                *target,
                "--jobs",
                env["CARGO_BUILD_JOBS"],
            ],
            cwd=ROOT,
            env=env,
            check=True,
            timeout=600,
        )


def run_ffs_001() -> None:
    consumer = ROOT / "tests/pre_release_ffi_app_lifetime.c"
    if not consumer.is_file():
        raise RuntimeError(f"missing focused FFS-001 consumer: {consumer.relative_to(ROOT)}")

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    run_ffs_001_rust_checks(env)
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


def run_ffs_003() -> None:
    consumer = ROOT / "tests/pre_release_ffi_ownership.c"
    if not consumer.is_file():
        raise RuntimeError(f"missing focused FFS-003 consumer: {consumer.relative_to(ROOT)}")

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--features",
            "ffi",
            "--lib",
            "callback_replacement_is_synchronized_and_reentrant",
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

    with tempfile.TemporaryDirectory(prefix="ffs-003-", dir=TARGET) as scratch:
        binary = Path(scratch) / "ffi-ownership"
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


def run_ffs_004() -> None:
    consumer = ROOT / "tests/pre_release_ffi_buffer_ownership.c"
    if not consumer.is_file():
        raise RuntimeError(f"missing focused FFS-004 consumer: {consumer.relative_to(ROOT)}")
    pointer_contract = (ROOT / "src/ffi/pointer.rs").read_text()
    required_limits = ("does not prove", "allocation", "liveness", "ownership", "readability")
    missing_limits = [phrase for phrase in required_limits if phrase not in pointer_contract]
    if missing_limits:
        raise RuntimeError(f"pointer validation contract omits limits: {missing_limits}")

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    subprocess.run(
        ["cargo", "build", "--locked", "--features", "ffi", "--jobs", env["CARGO_BUILD_JOBS"]],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )

    with tempfile.TemporaryDirectory(prefix="ffs-004-", dir=TARGET) as scratch:
        binary = Path(scratch) / "ffi-buffer-ownership"
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


def run_ffs_005() -> None:
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = build_jobs()
    env.setdefault("RUSTC_BOOTSTRAP", "1")

    subprocess.run(
        ["python3", "-B", "scripts/generate-native-header.py", "--verify"],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    subprocess.run(
        ["python3", "-B", "scripts/check-c-binding-abi.py"],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )

    public_header = (ROOT / "include/reactive_tui.h").read_text()
    match = re.search(
        r"^\s*\* ```c\s*$\n(?P<body>.*?)^\s*\* ```\s*$",
        public_header,
        re.MULTILINE | re.DOTALL,
    )
    if match is None:
        raise RuntimeError("include/reactive_tui.h has no compilable C example")
    example = "\n".join(
        re.sub(r"^\s*\* ?", "", line) for line in match.group("body").splitlines()
    )
    with tempfile.TemporaryDirectory(prefix="ffs-005-", dir=TARGET) as scratch:
        source = Path(scratch) / "header-example.c"
        binary = Path(scratch) / "header-example"
        source.write_text(example + "\n")
        subprocess.run(
            [
                "clang",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic-errors",
                "-Iinclude",
                str(source),
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
        ["python3", "-B", "scripts/check-typescript-abi.py"],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )

    policy = " ".join((ROOT / "manual/ffi-and-typescript.md").read_text().split())
    required_policy = (
        "does not prove that it is allocated or live",
        "caller-supplied length does not control deallocation",
        "Self-parenting is rejected",
        "Optional root and cleanup callbacks may be null",
    )
    missing_policy = [text for text in required_policy if text not in policy]
    if missing_policy:
        raise RuntimeError("manual ABI policy is incomplete: " + repr(missing_policy))


def main() -> int:
    requested = sys.argv[1:]
    if requested == ["FFS-001"]:
        run_ffs_001()
    elif requested == ["FFS-002"]:
        run_ffs_002()
    elif requested == ["FFS-003"]:
        run_ffs_003()
    elif requested == ["FFS-004"]:
        run_ffs_004()
    elif requested == ["FFS-005"]:
        run_ffs_005()
    else:
        raise SystemExit(
            "usage: check-pre-release-ffi-safety.py "
            "FFS-001|FFS-002|FFS-003|FFS-004|FFS-005"
        )
    print(f"{requested[0]} focused C ABI safety checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
