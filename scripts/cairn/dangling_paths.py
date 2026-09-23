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
# Dated reports and recorded inventories cite paths as they were (the ABI
# baselines list retired modules on purpose), and the contract names paths it
# requires to exist later; none of these is a live link.
DATED = ("docs/recon.md",)
# The Sudus ledger's outputs and briefs record what a run printed, including
# the missing paths a violating example names on purpose; .cairn/ is the
# layout's former name.
LEDGER_PREFIXES = (".sudus/", ".cairn/")
# A vendored crate's changelog records that crate's own history.
DATED_NAMES = ("CHANGELOG.md",)
DATED_PREFIXES = ("scripts/abi/baselines/",)
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


def package_roots(path: Path, tracked: set[str]) -> list[str]:
    """Directories above `path` that hold a package manifest; a reference in a
    file under a package resolves against that package too, the way its own
    tooling reads it (a package.json script names its build script relative
    to the package, not the repository)."""
    roots = []
    for parent in Path(rel(path)).parents:
        for manifest in ("package.json", "Cargo.toml", "pyproject.toml"):
            if str(parent / manifest) in tracked and str(parent) != ".":
                roots.append(str(parent))
    return roots


def exists(ref: str, tracked: set[str], dirs: set[str]) -> bool:
    return ref in tracked or ref in dirs or any(t.startswith(ref + ".") for t in tracked)


def dangling(path: Path, tracked: set[str], dirs: set[str]) -> list[str]:
    if not path.is_file() or path.suffix in SKIP_SUFFIX:
        return []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    roots = package_roots(path, tracked)
    bad = []
    for m in PATH_RE.finditer(text):
        ref = m.group(1).rstrip(".,:;)")
        if "*" in ref or "{" in ref or ref.endswith("/"):
            continue
        if exists(ref, tracked, dirs) or any(exists(f"{root}/{ref}", tracked, dirs) for root in roots):
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
        targets = [ROOT / f for f in sorted(tracked) if f not in DATED and not f.startswith(DATED_PREFIXES) and not (f.startswith("crates/") and f.endswith(DATED_NAMES)) and not f.startswith(CONTRACT_PREFIX) and not f.startswith(LEDGER_PREFIXES) and f.endswith((".md", ".py", ".sh", ".rs", ".toml", ".yml", ".yaml", ".json", ".mjs", ".ts", ".c", ".h"))
                   and not f.startswith(".github/")]
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
