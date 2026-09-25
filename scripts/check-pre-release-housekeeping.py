#!/usr/bin/env python3
"""Reject ignored tracked files and stale repository rules."""
from pathlib import Path
import fnmatch
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
# These are intentional future build/tool outputs, not removed product paths.
LOCAL_NOISE = {
    "target", "build", "dist", "node_modules", "__pycache__", "reference",
    ".idea", ".vscode", ".nova", ".zencoder", ".agent-os", ".claude",
    ".gitnexus", "CLAUDE.md", "test_image.png", ".DS_Store", "Thumbs.db",
}


def git(root: Path, *arguments: str, data: bytes | None = None):
    result = subprocess.run(["git", *arguments], cwd=root, input=data,
                            capture_output=True, timeout=30, check=False)
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr.decode(errors="replace").strip())
    return result


def stale_ignore_rules(root: Path, tracked: set[str]) -> list[str]:
    errors = []
    for name in sorted(tracked):
        if Path(name).name != ".gitignore":
            continue
        base = Path(name).parent
        for number, line in enumerate((root / name).read_text().splitlines(), 1):
            pattern = line.strip()
            if not pattern or pattern.startswith("#"):
                continue
            pattern = pattern.removeprefix("!").strip("/")
            prefix = re.split(r"[*?\[]", pattern, maxsplit=1)[0].rstrip("/")
            if not prefix or ("/" not in pattern and any(c in pattern for c in "*?[")):
                continue
            if prefix in LOCAL_NOISE:
                continue
            relative = (base / prefix).as_posix()
            if not any(p == relative or p.startswith(relative + "/") for p in tracked):
                errors.append(f"{name}:{number}: stale ignore rule: {line}")
    return errors


def stale_package_rules(root: Path, tracked: set[str]) -> list[str]:
    errors = []
    for name in sorted(tracked):
        if Path(name).name != "Cargo.toml":
            continue
        try:
            package = tomllib.loads((root / name).read_text()).get("package", {})
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(f"{name}: invalid package document: {error}")
            continue
        base = Path(name).parent
        members = [Path(p).relative_to(base).as_posix() for p in tracked
                   if Path(p).is_relative_to(base)]
        for key in ("include", "exclude"):
            patterns = package.get(key, [])
            if not isinstance(patterns, list) or any(not isinstance(p, str) for p in patterns):
                errors.append(f"{name}: invalid package {key} rules")
                continue
            for pattern in patterns:
                normalized = pattern.removeprefix("!").lstrip("/")
                if not any(fnmatch.fnmatchcase(p, normalized) or
                           p.startswith(normalized.rstrip("/") + "/") for p in members):
                    errors.append(f"{name}: stale package {key} rule: {pattern}")
    return errors


def inspect_repository(root: Path) -> list[str]:
    result = git(root, "ls-files", "-z")
    if result.returncode:
        raise RuntimeError("Could not enumerate tracked files")
    tracked = set(result.stdout.decode().strip("\0").split("\0")) - {""}
    ignored = git(root, "check-ignore", "--no-index", "--stdin", "-z",
                  data="\0".join(sorted(tracked)).encode() + b"\0")
    errors = [f"tracked file is ignored: {name}" for name in
              ignored.stdout.decode().strip("\0").split("\0") if name]
    errors += stale_ignore_rules(root, tracked) + stale_package_rules(root, tracked)
    return errors


def main() -> int:
    if sys.argv[1:] != ["RID-003"]:
        print("usage: check-pre-release-housekeeping.py RID-003", file=sys.stderr)
        return 2
    tests = subprocess.run([sys.executable, "-B", "scripts/test-pre-release-housekeeping.py"],
                           cwd=ROOT, timeout=60, check=False)
    if tests.returncode:
        return tests.returncode
    try:
        errors = inspect_repository(ROOT)
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        errors = [str(error)]
    if errors:
        print("RID-003 housekeeping failed:", file=sys.stderr)
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("RID-003 housekeeping passed; the decision queue is empty.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
