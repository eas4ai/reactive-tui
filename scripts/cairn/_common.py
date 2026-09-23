"""Shared helpers for the Cairn mechanism scripts under scripts/cairn/."""

from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
JOBS = os.environ.get("CARGO_BUILD_JOBS", "8")


# The environment a mechanism sees: enough to find the toolchain and the
# target directory, nothing that changes what a check means (REGENERATE,
# RUSTFLAGS and the CARGO_* build knobs are dropped on purpose).
ENV_KEEP = ("PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "LC_CTYPE", "TERM", "TMPDIR",
            "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "CARGO_TARGET_DIR", "SSL_CERT_FILE", "SSL_CERT_DIR")


def run(cmd: list[str], timeout: int = 1800, env: dict | None = None) -> subprocess.CompletedProcess:
    merged = {k: v for k, v in os.environ.items() if k in ENV_KEEP}
    merged["CARGO_BUILD_JOBS"] = JOBS
    if env:
        merged.update(env)
    print("$", " ".join(cmd), flush=True)
    return subprocess.run(cmd, cwd=ROOT, env=merged, text=True, capture_output=True, timeout=timeout)


def tracked_files() -> set[str]:
    """Paths git tracks; without a repository (an adversary projection has no
    .git) every file in the tree except build output and ignored caches."""
    out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True)
    if out.returncode == 0:
        return {p.decode() for p in out.stdout.split(b"\0") if p}
    skip = {".git", "target", "node_modules", "__pycache__", ".cairn"}
    found = set()
    for path in ROOT.rglob("*"):
        rel = path.relative_to(ROOT)
        if not path.is_file() or any(part in skip for part in rel.parts):
            continue
        found.add(str(rel))
    return found


def report(req: str, ok: bool, why: str = "") -> bool:
    """Print the exact result line Cairn parses, then the reason on its own line."""
    print(f"cairn: {req}: {'pass' if ok else 'fail'}", flush=True)
    why = " ".join(why.split())
    if why:
        print(f"  {req} reason: {why[:400]}", flush=True)
    return ok


def rust_sources(*dirs: str) -> list[Path]:
    files: list[Path] = []
    for d in dirs:
        p = ROOT / d
        if p.is_file():
            files.append(p)
        elif p.is_dir():
            files.extend(sorted(p.rglob("*.rs")))
    return files


def strip_test_modules(text: str) -> str:
    """Drop `#[cfg(test)] mod ... { ... }` bodies so probes see production code only."""
    out = []
    i = 0
    pat = re.compile(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{")
    while True:
        m = pat.search(text, i)
        if not m:
            out.append(text[i:])
            break
        out.append(text[i:m.start()])
        depth, j = 1, m.end()
        while j < len(text) and depth:
            depth += text[j] == "{"
            depth -= text[j] == "}"
            j += 1
        i = j
    return "".join(out)


def cargo_test_filtered(binary: str | None, substring: str, features: list[str] | None = None, release: bool = False,
                        package: str | None = None) -> tuple[bool, str]:
    """Run the tests whose names contain `substring` in one test binary, or
    in the package's library unit tests when `binary` is None."""
    cmd = ["cargo", "test", "--locked", "--jobs", JOBS]
    if package:
        cmd += ["-p", package]
    cmd += ["--test", binary] if binary else ["--lib"]
    if release:
        cmd.append("--release")
    if features:
        cmd += ["--features", ",".join(features)]
    cmd += ["--", substring]
    r = run(cmd)
    out = r.stdout + r.stderr
    ran = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", r.stdout)
    if not ran:
        # Say why no summary printed: cargo's error, what caused it (a signal,
        # a binary that could not start) and the last lines of output.
        lines = [l.strip() for l in out.splitlines() if l.strip()]
        why = [l for l in lines if l.startswith(("error", "Caused by")) or "signal" in l or "overflowed" in l]
        tail = [l for l in lines[-4:] if l not in why]
        detail = "; ".join(why[:4] + tail) or "no output"
        return False, f"{binary or 'the library tests'} did not run: {detail}"[:600]
    passed, failed = int(ran.group(1)), int(ran.group(2))
    if passed + failed == 0:
        return False, f"no test matched {substring!r} in {binary or 'the library tests'}"
    if failed == 0 and r.returncode == 0:
        return True, f"{passed} passed"
    # The stated violation is the panic message: the line after each "panicked at".
    lines = out.splitlines()
    messages = [lines[i + 1].strip() for i, l in enumerate(lines) if "panicked at" in l and i + 1 < len(lines)]
    return False, "; ".join(m for m in messages if m)[:400] or f"{failed} failed"


def finish(results: dict[str, tuple[bool, str]]) -> int:
    ok_all = True
    for req, (ok, why) in results.items():
        ok_all &= report(req, ok, why)
    return 0 if ok_all else 1
