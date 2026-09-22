#!/usr/bin/env python3
"""BAR-007: no tracked file references a repository path that `git ls-files`
does not list. Whole tree: deleting a target strands references in files that
did not change. `--range` limits the scan to files changed since --base/$CAIRN_BASE
(default tag v1.0.0); `--fixture PATH` scans one file.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

from _common import ROOT, tracked_files

TOP = "docs|scripts|tests|src|manual|include|examples|crates|verification|benches|bindings|\\.github"
PATH_RE = re.compile(rf"(?<![A-Za-z0-9_./-])(?:\.\./|\./)*((?:{TOP})/[A-Za-z0-9_./-]+)")
# Dated reports cite paths as they were, and the contract names paths it
# requires to exist later; neither is a live link.
DATED = ("docs/recon.md",)
CONTRACT_PREFIX = "docs/spec/"
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
    elif "--range" in sys.argv:
        base = sys.argv[sys.argv.index("--base") + 1] if "--base" in sys.argv else os.environ.get("CAIRN_BASE", "v1.0.0")
        targets = [ROOT / f for f in changed_files(base)]
    else:
        targets = [ROOT / f for f in sorted(tracked) if f not in DATED and not f.startswith(CONTRACT_PREFIX) and f.endswith((".md", ".py", ".sh", ".rs", ".toml", ".yml", ".yaml", ".json", ".mjs", ".ts", ".c", ".h"))
                   and not f.startswith(("crates/", ".github/"))]
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
