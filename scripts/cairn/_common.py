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


def run(cmd: list[str], timeout: int = 1800, env: dict | None = None, *, interleave: bool = False,
        stdin: str | None = None) -> subprocess.CompletedProcess:
    """Run `cmd` at the repository root in the mechanism environment. With
    `interleave`, stderr joins stdout in one stream, in the order written."""
    merged = {k: v for k, v in os.environ.items() if k in ENV_KEEP}
    merged["CARGO_BUILD_JOBS"] = JOBS
    # Incremental compilation off: rustc 1.95 intermittently panics with
    # "uninterned StableCrateId: StableCrateId(0)" reading metadata in
    # incremental builds (rust-lang/rust#149697, rust-clippy#14572), which
    # failed BAR-001 four times on 2026-09-23 with no code at fault. It does
    # not change what a check means; release builds are not incremental.
    merged["CARGO_INCREMENTAL"] = "0"
    if env:
        merged.update(env)
    print("$", " ".join(cmd), flush=True)
    return subprocess.run(cmd, cwd=ROOT, env=merged, text=True, input=stdin, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT if interleave else subprocess.PIPE, timeout=timeout)


def tracked_files() -> set[str]:
    """Paths git tracks; without a repository (an adversary projection has no
    .git) every file in the tree except build output and ignored caches."""
    out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True)
    if out.returncode == 0:
        return {p.decode() for p in out.stdout.split(b"\0") if p}
    skip = {".git", "target", "node_modules", "__pycache__", ".sudus", ".cairn"}
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
    """Regular .rs files under `dirs`; a directory named *.rs or a symlink
    that points nowhere is not a source."""
    files: list[Path] = []
    for d in dirs:
        p = ROOT / d
        if p.is_file():
            files.append(p)
        elif p.is_dir():
            files.extend(sorted(f for f in p.rglob("*.rs") if f.is_file()))
    return files


RAW_STRING = re.compile(r"[bc]?r(#*)\"")
CHAR = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|.)|[^\\'\n])'")
TEST_CFG = re.compile(r"#\[cfg\(\s*(?:test|all\(\s*test\b[^\]]*)\)\]")


def mask(text: str) -> tuple[str, str]:
    """Return (code, prose): `code` has comments removed and literal contents
    blanked; `prose` has comments removed and literals kept. Both keep every
    offset and newline of `text`."""
    code, prose = list(text), list(text)

    def blank(buf: list[str], start: int, end: int) -> None:
        for k in range(start, min(end, len(buf))):
            if buf[k] != "\n":
                buf[k] = " "

    def after_ident(i: int) -> bool:
        return i > 0 and (text[i - 1].isalnum() or text[i - 1] == "_")

    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            blank(code, i, end)
            blank(prose, i, end)
            i = end
            continue
        if text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(code, i, j)
            blank(prose, i, j)
            i = j
            continue
        raw = RAW_STRING.match(text, i)
        if raw and not after_ident(i):
            close = '"' + raw.group(1)
            end = text.find(close, raw.end())
            end = n if end < 0 else end
            blank(code, raw.end(), end)
            i = end + len(close)
            continue
        if text[i] == '"' or (text[i] in "bc" and text.startswith('"', i + 1) and not after_ident(i)):
            j = start = i + (1 if text[i] == '"' else 2)
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(code, start, j)
            i = j + 1
            continue
        if text[i] == "'":
            char = CHAR.match(text, i)
            if char:
                blank(code, i + 1, char.end() - 1)
                i = char.end()
                continue
        i += 1
    return "".join(code), "".join(prose)


def matching(code: str, i: int) -> int:
    """Index just past the bracket that closes the one at code[i]."""
    pairs = {"{": "}", "[": "]", "(": ")"}
    opening, closing = code[i], pairs[code[i]]
    depth, j = 1, i + 1
    while j < len(code) and depth:
        depth += code[j] == opening
        depth -= code[j] == closing
        j += 1
    return j


def strip_test_modules(text: str) -> str:
    """Blank every item under #[cfg(test)] or #[cfg(all(test, ...))], a
    module, function, impl or use, so probes see production code only.
    Braces are matched with comments and literals masked; the rest of the
    text, its offsets and its newlines are kept."""
    code, _ = mask(text)
    out = list(text)
    i = 0
    while m := TEST_CFG.search(code, i):
        j = m.end()
        while True:
            k = j
            while k < len(code) and code[k].isspace():
                k += 1
            if not code.startswith("#[", k):
                break
            j = matching(code, k + 1)
        brace, semi = code.find("{", j), code.find(";", j)
        end = matching(code, brace) if brace >= 0 and (semi < 0 or brace < semi) else (semi + 1 if semi >= 0 else len(code))
        for p in range(m.start(), end):
            if out[p] != "\n":
                out[p] = " "
        i = end
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
