#!/usr/bin/env python3
"""BAR-007: no tracked file references a repository path that `git ls-files`
does not list. Whole tree: deleting a target strands references in files that
did not change. `--range` limits the scan to files changed since --base/$CAIRN_BASE
(default tag v1.0.0); `--fixture PATH` scans one file.

Every tracked text file is read, the CI workflows and the specification
included, except the records that cite paths as they were (below). Four
forms of reference are checked:

- a path under any top-level directory the tree has or once had, with any
  leading `./` and `../` resolved from the file's own directory; in
  Markdown a trailing slash names a directory that must exist, while in
  code it is a prefix and names nothing. A directory the tree once had is
  one git history deleted a file from, or one a mechanism declares as an
  input, so deleting a whole directory is caught in the files that still
  point into it, with or without a repository;
- in code, a file name joined to the repository root (`ROOT / "name"`),
  and in CI workflows, shell scripts and manifests, a file an interpreter
  or a `--config`-style option is given, or a `./name` a step runs; each
  is read from the root, and names under `target/` are build output;
- in Markdown, every link target that is not a URL or an anchor, resolved
  from the file's directory, so a top-level file such as a README is
  checked too;
- in Markdown, a bare upper-case file name such as README.md, which must
  exist beside the file, at the root, in its package or, for a name that
  gives no directory, anywhere in the tree.
"""

import json
import os
import posixpath
import re
import subprocess
import sys
import urllib.parse
from pathlib import Path

from _common import ROOT, tracked_files

# Dated reports and recorded inventories cite paths as they were (the ABI
# baselines list retired modules on purpose); none of these is a live link.
DATED = ("docs/recon.md", "docs/decisions.jsonl")
# The Sudus ledger's outputs and briefs record what a run printed, including
# the missing paths a violating example names on purpose; .cairn/ is the
# layout's former name.
LEDGER_PREFIXES = (".sudus/", ".cairn/")
# A vendored crate's changelog records that crate's own history.
DATED_NAMES = ("CHANGELOG.md",)
DATED_PREFIXES = ("scripts/abi/baselines/",)
# Captured program output and screen goldens record what was printed.
RECORD_SUFFIXES = (".out", ".ansi", ".screen")
RECORD_PREFIXES = ("tests/snapshots/",)
# Not documents: ignore patterns name untracked paths by design, a patch
# names the paths of the tree it patches, and lock files and images hold no
# links.
SKIP_NAMES = (".gitignore",)
SKIP_SUFFIX = (".lock", ".patch", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".exe", ".dll", ".so", ".wasm")
# References that name another repository's tree, with the reason. Each
# path is written in parts so this table does not itself cite it.
EXTERNAL = {
    ("docs/spec/charts.md", "/".join(("crates", "component", "src", "chart"))):
        "a directory of gpui-kit 0.6.6, the model the charts follow",
}
# Installed at build time, never tracked.
INSTALLED = "/node_modules/"
# A file name as a script or a CI step gives it: no leading slash, variable
# or home directory, and an extension.
FILE_NAME = r"[A-Za-z0-9_][A-Za-z0-9_./-]*\.[A-Za-z0-9]+"
ROOTED = re.compile(rf"""\b(?:ROOT|REPO|REPO_ROOT|repo_root)\s*(?:/\s*|\.joinpath\(\s*)["']({FILE_NAME})["']""")
RUN_WORD = re.compile(
    rf"""(?:(?:^[ \t]*|[;&|(]\s*|\brun:\s*|\s)(?:python3?|bash|sh|node|pwsh|ruby|perl)(?:\s+-[A-Za-z]+)*\s+"""
    rf"""|--(?:config|manifest-path|file|config-file)[=\s]+"""
    rf"""|(?:^[ \t]*|[;&|]\s*|\brun:\s*)\./)({FILE_NAME})(?![A-Za-z0-9_/.-])""", re.M)
COMMAND_SUFFIXES = (".sh", ".yml", ".yaml", ".toml", ".json")
COMMAND_NAMES = ("Makefile", "justfile")
BUILD_OUTPUT = "target/"
LINK = re.compile(r"\[[^\]\n]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
BARE_NAME = re.compile(r"(?<![A-Za-z0-9_./-])([A-Z][A-Z0-9_-]*\.md)\b")


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


def former_directories(tracked: set[str]) -> set[str]:
    """Top-level directories the tree may no longer have: every one git
    history deleted a file from (a move counts as a deletion), and every
    top-level input a mechanism declares, which is all an adversary
    projection without history can know."""
    found = set()
    try:
        out = subprocess.run(["git", "log", "--format=", "--name-only", "--no-renames", "--diff-filter=D", "-z"],
                             cwd=ROOT, capture_output=True)
    except FileNotFoundError:
        out = None
    if out is not None and out.returncode == 0:
        found |= {p.strip().split("/", 1)[0] for p in out.stdout.decode(errors="replace").split("\0") if "/" in p}
    for layout in (".sudus", ".cairn"):
        for declaration in sorted((ROOT / layout / "mechanisms").glob("*.json")):
            try:
                inputs = json.loads(declaration.read_text())["definition"]["inputs"]
            except (OSError, ValueError, KeyError, TypeError):
                continue
            for entry in inputs:
                top = entry.split("/", 1)[0]
                if "/" in entry or ("." not in top and top not in tracked):
                    found.add(top)
    return {d for d in found if d and d not in (".", "..")}


def path_pattern(tracked: set[str], former: set[str] = frozenset()) -> re.Pattern:
    """A path under any top-level directory the tree has or once had; the
    leading ./ and ../ segments are captured, so a link of the wrong depth
    is caught."""
    top = sorted({p.split("/", 1)[0] for p in tracked if "/" in p} | set(former))
    names = "|".join(re.escape(t) for t in top)
    return re.compile(rf"(?<![A-Za-z0-9_./-])((?:\.\./|\./)*)((?:{names})/[A-Za-z0-9_./-]*)")


def dangling(path: Path, tracked: set[str], dirs: set[str], pattern: re.Pattern | None = None) -> list[str]:
    if not path.is_file() or path.suffix in SKIP_SUFFIX or path.name in SKIP_NAMES:
        return []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    if "\0" in text:
        return []
    pattern = pattern or path_pattern(tracked)
    name = rel(path)
    base = posixpath.dirname(name)
    markdown = path.suffix == ".md"
    roots = package_roots(path, tracked)
    here = lambda ref: exists(ref, tracked, dirs) or any(exists(f"{root}/{ref}", tracked, dirs) for root in roots)
    bad = []
    for m in pattern.finditer(text):
        climb, ref = m.group(1), m.group(2).rstrip(".,:;)`'\"")
        if "*" in ref or "{" in ref or "<" in ref or "$" in ref or INSTALLED in f"/{ref}":
            continue
        if (name, ref.rstrip("/")) in EXTERNAL:
            continue
        if ref.endswith("/") and not markdown:
            continue
        target = ref.rstrip("/")
        if "../" in climb:
            target = posixpath.normpath(posixpath.join(base, climb + target))
            if not target.startswith("../") and exists(target, tracked, dirs):
                continue
            bad.append(f"{name}: {climb}{ref} (resolves to {target})")
            continue
        if not here(target):
            bad.append(f"{name}: {ref}")
    if markdown:
        for m in LINK.finditer(text):
            link = m.group(1)
            if re.match(r"^[a-z][a-z0-9+.-]*:", link, re.I) or link.startswith("#"):
                continue
            local = urllib.parse.unquote(link.split("#", 1)[0].split("?", 1)[0])
            if not local:
                continue
            target = posixpath.normpath(local.lstrip("/") if local.startswith("/") else posixpath.join(base, local))
            if target.startswith("../") or not exists(target.rstrip("/"), tracked, dirs):
                bad.append(f"{name}: link {link} (resolves to {target})")
        basenames = {posixpath.basename(t) for t in tracked}
        for m in BARE_NAME.finditer(text):
            bare = m.group(1)
            if not (here(posixpath.join(base, bare)) or here(bare) or bare in basenames):
                bad.append(f"{name}: {bare}")
    named = [m.group(1) for m in ROOTED.finditer(text)]
    if name.startswith(".github/") or path.suffix in COMMAND_SUFFIXES or path.name in COMMAND_NAMES:
        named += [m.group(1) for m in RUN_WORD.finditer(text)]
    for ref in dict.fromkeys(named):
        target = posixpath.normpath(ref)
        if target.startswith(BUILD_OUTPUT) or (name, target) in EXTERNAL:
            continue
        if not (here(target) or exists(posixpath.join(base, target), tracked, dirs)):
            bad.append(f"{name}: {ref} (a file the root has no copy of)")
    return bad


def scanned(f: str) -> bool:
    return not (f in DATED or f.startswith(DATED_PREFIXES) or f.startswith(LEDGER_PREFIXES)
                or (f.startswith("crates/") and f.endswith(DATED_NAMES))
                or f.endswith(RECORD_SUFFIXES) or f.startswith(RECORD_PREFIXES))


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
        targets = [ROOT / f for f in sorted(tracked) if scanned(f)]
    pattern = path_pattern(tracked, former_directories(tracked))
    bad = [b for t in targets for b in dangling(t, tracked, dirs, pattern)]
    if bad:
        print("BAR-007 violated: references to paths git does not track:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-007 holds: {len(targets)} files scanned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
