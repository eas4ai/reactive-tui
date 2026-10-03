#!/usr/bin/env python3
"""charts-pictures: CHT-037, CHT-038 and CHT-039 (docs/spec/charts.md) through
tests/charts_pictures.rs, built with wgpu-graphics. A line, area, scatter, bar or
candlestick chart on a terminal that takes Kitty graphics or Sixel draws its plot
area as one pixel picture, with the hover marks in it; the tests run an App on a
backend that writes to memory, decode the pictures it sends and compare them with
the reference pictures under tests/snapshots/charts/pictures.

CHT-037 also asks that every reference picture the tests name is checked in and
tracked, and that the test file writes a reference only under REGENERATE=1
(BAR-004's rule, audited as charts-goldens audits its golden tests).

CHT-039 binds on this host: the nine cartesian charts of the catalog's Charts page
at 240 by 60 cells, in a release build on the hardware adapter, all send their
first picture within a second and keep the App's work per frame and its wait in
present under 16.6 ms at the 95th percentile while a bar is hovered every frame.
The same run on the macOS host and the Windows tablet is recorded, not binding.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import JOBS, ROOT, cargo_test_filtered, report, run
from charts_goldens import regeneration_problems
from host_builds import CONFIG, HOSTS, Unreachable, run_on, snapshot

FEATURE = ["wgpu-graphics"]
PACKAGE = "reactive-tui"
TEST = "tests/charts_pictures.rs"
PICTURES = ROOT / "tests/snapshots/charts/pictures"


def reference_names() -> list[str]:
    """The reference pictures the test file names: the `("name", element)`
    entries of the lists handed to the reference check."""
    text = (ROOT / TEST).read_text(errors="replace")
    return sorted(set(re.findall(r'^\s*\("([a-z0-9_]+)", .*\),\s*$', text, re.M))
                  | set(re.findall(r'vec!\[\("([a-z0-9_]+)", ', text)))


def altered_reference_problems() -> list[str]:
    """CHT-037: a reference picture that differs beyond GFX-002's tolerance
    fails the comparison. The candlestick's reference is replaced by a flat
    picture of its size in a copy of the snapshots, and its test must fail
    there."""
    original = PICTURES / "candlestick.png"
    if not original.is_file():
        return []
    try:
        from PIL import Image
    except ImportError:
        return ["Pillow is not installed, so the altered reference picture could not be made"]
    with tempfile.TemporaryDirectory(prefix="charts-pictures-altered-") as scratch:
        copy = Path(scratch) / "charts/pictures"
        copy.mkdir(parents=True)
        for png in PICTURES.glob("*.png"):
            (copy / png.name).write_bytes(png.read_bytes())
        with Image.open(original) as image:
            Image.new("RGBA", image.size, (120, 40, 200, 255)).save(copy / "candlestick.png")
        ok, why = cargo_test_filtered("charts_pictures", "cht_037_a_candlestick", features=FEATURE, package=PACKAGE,
                                      env={"REACTIVE_TUI_SNAPSHOTS": scratch})
    if ok:
        return ["the candlestick test passed against an altered reference picture"]
    if "differs" not in why and "sizes differ" not in why:
        return [f"the candlestick test failed against an altered reference for another reason: {why}"]
    return []


def reference_problems() -> list[str]:
    """Every reference picture named exists, is tracked by git, and is a PNG."""
    problems = []
    tracked = set(subprocess.run(["git", "ls-files", "tests/snapshots/charts/pictures"], cwd=ROOT,
                                 capture_output=True, text=True).stdout.split())
    names = reference_names()
    if not names:
        problems.append(f"{TEST} names no reference picture")
    for name in names:
        path = PICTURES / f"{name}.png"
        if not path.is_file():
            problems.append(f"missing reference picture {path.relative_to(ROOT)}")
            continue
        if str(path.relative_to(ROOT)) not in tracked:
            problems.append(f"reference picture {path.relative_to(ROOT)} is not tracked")
        if path.read_bytes()[:8] != b"\x89PNG\r\n\x1a\n":
            problems.append(f"reference picture {path.relative_to(ROOT)} is not a PNG")
    return problems


def test_summary(output: str) -> tuple[bool, str]:
    """Whether one run of the picture tests passed, with the stated violations."""
    ran = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", output)
    if not ran:
        lines = [l.strip() for l in output.splitlines() if l.strip()]
        return False, "did not run: " + "; ".join(lines[-3:])[:300]
    passed, failed = int(ran.group(1)), int(ran.group(2))
    if passed + failed == 0:
        return False, "no test ran"
    if failed == 0:
        return True, f"{passed} passed"
    lines = output.splitlines()
    messages = [lines[i + 1].strip() for i, l in enumerate(lines) if "panicked at" in l and i + 1 < len(lines)]
    return False, "; ".join(m for m in messages if m)[:300] or f"{failed} failed"


def speed_line(output: str) -> str:
    """The line the CHT-039 run prints with its numbers."""
    found = re.search(r"CHT-039 .*", output)
    return found.group(0) if found else "no CHT-039 line"


def main() -> int:
    results = {}
    ok, why = cargo_test_filtered("charts_pictures", "cht_037_", features=FEATURE, package=PACKAGE)
    problems = reference_problems() + regeneration_problems(TEST) + altered_reference_problems()
    results["CHT-037"] = (ok and not problems, "; ".join(problems[:6]) or why)
    results["CHT-038"] = cargo_test_filtered("charts_pictures", "cht_038_", features=FEATURE, package=PACKAGE)
    tests = f"test --locked -p {PACKAGE} --features wgpu-graphics --jobs {JOBS}"
    local = run(["cargo", *f"{tests} --release --test charts_pictures -- --ignored cht_039_ --nocapture".split()],
                timeout=3600, interleave=True)
    print(local.stdout[-3000:])
    # An interleaved run has its stderr in stdout.
    speed_ok, speed_why = test_summary(local.stdout)
    recorded = [f"linux: {speed_line(local.stdout)}; {speed_why}"]
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"no readable test host file at {CONFIG} ({error})")
        config = {}
    with tempfile.TemporaryDirectory(prefix="charts-pictures-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        for name in HOSTS:
            try:
                _, out = run_on(name, config[name], commit, bundle,
                                f"{tests} --release --test charts_pictures -- --ignored cht_039_ --nocapture")
                recorded.append(f"{name}: {speed_line(out)}")
            except (Unreachable, KeyError) as error:
                print(f"{name} not reached: {error}")
                recorded.append(f"{name}: not reached")
    results["CHT-039"] = (speed_ok, "; ".join(recorded))
    ok_all = True
    for req, (ok, why) in results.items():
        ok_all &= report(req, ok, why)
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
