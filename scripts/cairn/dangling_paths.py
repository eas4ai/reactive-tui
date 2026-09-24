#!/usr/bin/env python3
"""BAR-007: no tracked file references a repository path that `git ls-files`
does not list. Whole tree: deleting a target strands references in files that
did not change. `--range` limits the scan to files changed since --base/$CAIRN_BASE
(default tag v1.0.0); `--fixture PATH` scans one file.

Every tracked text file is read, the CI workflows and the specification
included, except the records that cite paths as they were (below). These
forms of reference are checked:

- a path under any top-level directory the tree has or once had, with any
  leading `./` and `../` resolved from the file's own directory; in
  Markdown a trailing slash names a directory that must exist, while in
  code it is a prefix and names nothing. A directory the tree once had is
  one git history deleted a file from, or one a mechanism declares as an
  input, so deleting a whole directory is caught in the files that still
  point into it, with or without a repository;
- in a Python script, every path it builds from a known base, read by
  path_expressions.py without running it: `ROOT / "a" / "b"`, joinpath,
  os.path.join, `Path(__file__).parents[N] / "x"`, a name holding such a
  path (`HERE / "x"`), open("x"), a relative path kept as data
  ("tools/gen.py"), and a command list's script, --config-style file and
  cargo target;
- in Rust, a chain of literal `.join("x")` calls from
  env!("CARGO_MANIFEST_DIR") or a name bound to it, and
  concat!(env!("CARGO_MANIFEST_DIR"), "/x"), read from the file's package;
- in CI workflows, shell scripts and manifests, a file an interpreter or a
  `--config`-style option is given, a `./name` a step runs, and a path
  after $ROOT/ or $(dirname "$0")/; each is read from the root (or the
  script's directory), and names under `target/` are build output;
- anywhere, a cargo --example, --test, --bench or --bin target, which some
  tracked package must declare or cargo must find in its examples/,
  tests/, benches/ or src/bin/;
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
import tomllib
import urllib.parse
from pathlib import Path

import path_expressions
from _common import ROOT, mask, tracked_files

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
# A path after a shell variable holding the root, or after the script's own
# directory.
SHELL_ROOTED = re.compile(r"""\$\{?(?:ROOT|REPO|REPO_ROOT|ROOT_DIR|PROJECT_ROOT|GITHUB_WORKSPACE)\}?/([A-Za-z0-9_.][A-Za-z0-9_./-]*)""")
SHELL_HERE = re.compile(r"""\$\(\s*dirname\s+"?\$(?:0|\{BASH_SOURCE(?:\[0\])?\})"?\s*\)"?/([A-Za-z0-9_.][A-Za-z0-9_./-]*)""")
# A cargo target a command names; the name must follow the option directly.
CARGO_TARGET = re.compile(
    r"\bcargo(?:[ \t]+\+[\w.-]+)?[ \t]+[a-z][a-z-]*\b[^\n;&|`]*?(?<=\s)--(example|test|bench|bin)(?:=|[ \t]+)"
    r"([A-Za-z0-9_][A-Za-z0-9_-]*)(?![A-Za-z0-9_<>-])")
# Rust: a path joined to the package directory.
MANIFEST_DIR = r'env!\(\s*"CARGO_MANIFEST_DIR"\s*\)'
MANIFEST_BASE = rf"(?:(?:std::path::)?(?:PathBuf::from|Path::new)\(\s*{MANIFEST_DIR}\s*\))"
MANIFEST_LET = re.compile(rf"\blet\s+(?:mut\s+)?(\w+)\s*(?::\s*[\w:<>&' ]+)?=\s*&?{MANIFEST_BASE}\s*;")
MANIFEST_JOIN = re.compile(r'\s*\.join\(\s*"([^"\\\n]*)"\s*\)')
MANIFEST_CONCAT = re.compile(rf'concat!\(\s*{MANIFEST_DIR}\s*,\s*"(/[^"\\\n]*)"')
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


def package_of(name: str, tracked: set[str]) -> str:
    """The directory of the nearest Cargo package that holds `name`: the
    directory env!("CARGO_MANIFEST_DIR") names when it is compiled."""
    for parent in Path(name).parents:
        if str(parent / "Cargo.toml") in tracked:
            return str(parent)
    return "."


def cargo_targets(tracked: set[str]) -> set[str]:
    """"kind name" for every example, test, bench and bin target a tracked
    package declares or cargo finds in its examples/, tests/, benches/ or
    src/bin/ (a file.rs, or a directory with main.rs)."""
    found = set()
    for manifest in sorted(t for t in tracked if posixpath.basename(t) == "Cargo.toml"):
        package = posixpath.dirname(manifest)
        try:
            data = tomllib.loads((ROOT / manifest).read_text())
        except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError):
            continue
        # The package layout's paths are written in parts so this function
        # does not itself cite them from the root.
        for kind, folder in (("example", "examples"), ("test", "tests"), ("bench", "benches"),
                             ("bin", posixpath.join("src", "bin"))):
            found |= {f"{kind} {entry['name']}" for entry in data.get(kind, []) if isinstance(entry, dict) and entry.get("name")}
            prefix = posixpath.join(package, folder) + "/"
            for rest in (t[len(prefix):] for t in tracked if t.startswith(prefix)):
                if rest.endswith(".rs") and "/" not in rest:
                    found.add(f"{kind} {rest[:-3]}")
                elif rest.count("/") == 1 and rest.endswith("/main.rs"):
                    found.add(f"{kind} {rest.split('/')[0]}")
        name = data.get("package", {}).get("name")
        if name and posixpath.join(package, "src", "main.rs") in tracked:
            found.add(f"bin {name}")
    return found


def manifest_paths(text: str, package: str) -> list[tuple[int, str, str]]:
    """(line, source, path) for each Rust path joined to the package
    directory with literal segments only."""
    _, prose = mask(text)
    names = [re.escape(m.group(1)) for m in MANIFEST_LET.finditer(prose)]
    base = re.compile(rf"(?<![\w.]){MANIFEST_BASE}" + (rf"|\b(?:{'|'.join(names)})\b" if names else ""))
    found = []
    for m in base.finditer(prose):
        segments, end = [], m.end()
        while joined := MANIFEST_JOIN.match(prose, end):
            segments.append(joined.group(1))
            end = joined.end()
        if segments:
            found.append((text.count("\n", 0, m.start()) + 1, " ".join(prose[m.start():end].split()),
                          posixpath.normpath(posixpath.join(package, *segments))))
    for m in MANIFEST_CONCAT.finditer(prose):
        found.append((text.count("\n", 0, m.start()) + 1, " ".join(m.group(0).split()),
                      posixpath.normpath(package + m.group(1))))
    return found


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


def python_script(path: Path, text: str) -> bool:
    return path.suffix == ".py" or (text.startswith("#!") and "python" in text.split("\n", 1)[0])


def dangling(path: Path, tracked: set[str], dirs: set[str], pattern: re.Pattern | None = None,
             targets: set[str] | None = None) -> list[str]:
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
    seen = set()
    for m in pattern.finditer(text):
        climb, ref = m.group(1), m.group(2).rstrip(".,:;)`'\"")
        if "*" in ref or "{" in ref or "<" in ref or "$" in ref or INSTALLED in f"/{ref}":
            continue
        if (name, ref.rstrip("/")) in EXTERNAL:
            continue
        if ref.endswith("/") and not markdown:
            continue
        target = ref.rstrip("/")
        seen.add(target)
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
    targets = cargo_targets(tracked) if targets is None else targets
    built = lambda target: (target in ("", ".") or target == BUILD_OUTPUT.rstrip("/") or target.startswith(BUILD_OUTPUT)
                            or INSTALLED in f"/{target}/" or (name, target) in EXTERNAL)
    commands = [text]
    references = path_expressions.references(name, text) if python_script(path, text) else None
    named = [m.group(1) for m in ROOTED.finditer(text)] if references is None else []
    if references is not None:
        # A script's strings are read for commands one by one, as a shell
        # would run them.
        commands = [r.name for r in references if r.kind == "command"]
        reported = set()
        for r in references:
            if r.kind == "target" and r.name not in targets:
                bad.append(f"{name}:{r.line}: {r.source} (no package has the {r.name.replace(' ', ' target ', 1)})")
            if r.kind != "path" or r.place is None:
                continue
            target = r.place.path
            if built(target) or (r.line, target) in reported:
                continue
            found = exists(target, tracked, dirs) if r.place.base != "cwd" else (
                here(target) or exists(posixpath.normpath(posixpath.join(base, target)), tracked, dirs))
            if not found:
                reported.add((r.line, target))
                bad.append(f"{name}:{r.line}: {r.source} (resolves to {target}, which git does not track)")
    if path.suffix == ".rs":
        for line, source, target in manifest_paths(text, package_of(name, tracked)):
            if not (built(target) or exists(target, tracked, dirs)):
                bad.append(f"{name}:{line}: {source} (resolves to {target}, which git does not track)")
    command_file = name.startswith(".github/") or path.suffix in COMMAND_SUFFIXES or path.name in COMMAND_NAMES
    for command in commands:
        if command_file or references is not None:
            named += [m.group(1) for m in RUN_WORD.finditer(command)]
            named += [m.group(1) for m in SHELL_ROOTED.finditer(command)]
            named += [posixpath.join(base, m.group(1)) for m in SHELL_HERE.finditer(command)]
        for m in CARGO_TARGET.finditer(command):
            if f"{m.group(1)} {m.group(2)}" not in targets:
                bad.append(f"{name}: {' '.join(m.group(0).split())} (no package has the {m.group(1)} target {m.group(2)})")
    for ref in dict.fromkeys(named):
        target = posixpath.normpath(ref)
        # A path under a known directory was read above.
        if target in seen or built(target):
            continue
        if not (here(target) or exists(posixpath.join(base, target), tracked, dirs)):
            bad.append(f"{name}: {ref} (a file the root has no copy of)")
    return list(dict.fromkeys(bad))


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
    cargo = cargo_targets(tracked)
    bad = [b for t in targets for b in dangling(t, tracked, dirs, pattern, cargo)]
    if bad:
        print("BAR-007 violated: references to paths git does not track:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-007 holds: {len(targets)} files scanned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
