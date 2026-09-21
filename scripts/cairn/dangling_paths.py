#!/usr/bin/env python3
"""BAR-007: no file added or changed in the commitment range references a path
that `git ls-files` does not list.

Range: files changed between BASE and HEAD plus staged and unstaged changes.
BASE is `--base REF`, else $CAIRN_BASE, else the tag v1.0.0.
`--fixture PATH` scans one file (used to demonstrate the failing case).
"""

import os
import re
import subprocess
import sys
from pathlib import Path

from _common import ROOT, tracked_files

TOP = "docs|scripts|tests|src|manual|include|examples|crates|verification|benches|bindings|\\.github"
PATH_RE = re.compile(rf"(?<![A-Za-z0-9_./-])((?:{TOP})/[A-Za-z0-9_./-]+)")
SKIP_SUFFIX = (".lock",)


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def changed_files(base: str) -> list[str]:
    out = subprocess.run(["git", "diff", "--name-only", f"{base}...HEAD"], cwd=ROOT, capture_output=True, text=True).stdout
    out += subprocess.run(["git", "diff", "--name-only", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout
    out += subprocess.run(["git", "diff", "--name-only", "--cached"], cwd=ROOT, capture_output=True, text=True).stdout
    return sorted({l.strip() for l in out.splitlines() if l.strip()})


def dangling(path: Path, tracked: set[str], dirs: set[str]) -> list[str]:
    if not path.is_file() or path.suffix in SKIP_SUFFIX:
        return []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    bad = []
    for m in PATH_RE.finditer(text):
        ref = m.group(1).rstrip(".,:;)")
        if "*" in ref or "{" in ref or ref.endswith("/"):
            continue
        if ref in tracked or ref in dirs or any(t.startswith(ref + ".") for t in tracked):
            continue
        bad.append(f"{rel(path)}: {ref}")
    return bad


def main() -> int:
    tracked = tracked_files()
    dirs = {str(Path(p).parent) for p in tracked}
    dirs |= {d for p in tracked for d in map(str, Path(p).parents)}
    if len(sys.argv) > 2 and sys.argv[1] == "--fixture":
        targets = [Path(sys.argv[2]).resolve()]
    else:
        base = sys.argv[sys.argv.index("--base") + 1] if "--base" in sys.argv else os.environ.get("CAIRN_BASE", "v1.0.0")
        targets = [ROOT / f for f in changed_files(base)]
    bad = [b for t in targets for b in dangling(t, tracked, dirs)]
    if bad:
        print("BAR-007 violated: references to paths git does not track:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-007 holds: {len(targets)} changed files scanned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
