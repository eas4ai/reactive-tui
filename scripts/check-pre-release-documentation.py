#!/usr/bin/env python3
"""Check tracked local documentation links."""
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
MANUAL_CHECKER = load_checker(Path(__file__).with_name("check-framework-manual.py"),
                              "rid_manual", "Framework manual checker")


def git(root: Path, *arguments: str, data: bytes | None = None) -> subprocess.CompletedProcess:
    result = subprocess.run(["git", *arguments], cwd=root, input=data,
                            capture_output=True, timeout=30, check=False)
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr.decode(errors="replace"))
    return result


def links(text: str) -> list[str]:
    text = re.sub(r"(?ms)^\s*(`{3,}|~{3,})[^\n]*\n.*?^\s*\1[^\n]*(?:\n|$)", "", text)
    targets = re.findall(r"!?\[[^]\n]*\]\(([^)\n]+)\)", text)
    targets.extend(re.findall(r"(?m)^\s*\[[^]\n]+\]:\s*(\S+)", text))
    targets.extend(re.findall(r'''(?:href|src)\s*=\s*["']([^"']+)["']''', text))
    return [target.strip().split(' "', 1)[0].strip("<>") for target in targets]


def inspect_links(root: Path, tracked: set[str]) -> list[str]:
    errors = []
    for name in sorted(tracked):
        page = root / name
        if page.suffix.lower() not in (".md", ".markdown") or not page.is_file():
            continue
        for raw in links(page.read_text(errors="replace")):
            url = urlsplit(raw)
            if url.scheme or url.netloc:
                continue
            target = (page.parent / unquote(url.path)).resolve() if url.path else page.resolve()
            if not target.is_relative_to(root):
                errors.append(f"{name}: link escapes repository: {raw}")
                continue
            relative = target.relative_to(root).as_posix()
            members = [p for p in tracked if p == relative or p.startswith(relative.rstrip("/") + "/")]
            if not target.exists() or not members:
                errors.append(f"{name}: missing or untracked link target: {raw}")
                continue
            if url.fragment and target.is_file() and target.suffix.lower() == ".md":
                content = target.read_text(errors="replace")
                anchors = {MANUAL_CHECKER.anchor(h) for h in MANUAL_CHECKER.headings(content)}
                anchors.update(re.findall(r'''(?:id|name)=["']([^"']+)["']''', content))
                if unquote(url.fragment) not in anchors:
                    errors.append(f"{name}: missing heading target: {raw}")
    return errors


def inspect_repository(root: Path) -> list[str]:
    root = root.resolve()
    result = git(root, "ls-files", "-z")
    if result.returncode:
        raise RuntimeError("Could not enumerate tracked repository files")
    tracked = set(result.stdout.decode().strip("\0").split("\0")) - {""}
    return inspect_links(root, tracked)


def main() -> int:
    if sys.argv[1:] != ["RID-002"]:
        print("usage: check-pre-release-documentation.py RID-002", file=sys.stderr)
        return 2
    tests = subprocess.run([sys.executable, "-B", "scripts/test-pre-release-documentation.py"],
                           cwd=ROOT, timeout=60, check=False)
    if tests.returncode:
        return tests.returncode
    try:
        errors = inspect_repository(ROOT)
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        errors = [str(error)]
    if errors:
        print("RID-002 link inspection failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("RID-002 link inspection passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
