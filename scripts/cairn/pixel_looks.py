#!/usr/bin/env python3
"""pixel-looks: PIX-001 to PIX-006 (docs/spec/pixel-looks.md) through
tests/pixel_looks.rs, built with wgpu-graphics. A widget's or element's look is
one pixel picture over its rectangle where the terminal takes Kitty graphics
or Sixel, drawn around its text, kept and sent only when it changes, hidden
under what covers it, and back to cells byte for byte without pixels; the
tests run an App on a backend that writes to memory, decode the pictures it
sends and compare them with the reference pictures under
tests/snapshots/pixel-looks.

PIX-001 also asks, where the private display can be had (Xvfb and kitty), for
a screenshot of the catalog's Input page in which a label cell's background
equals the picture pixel beside it; without them that part is unverified and
the requirement has no verdict.

PIX-003 to PIX-005 also ask that every reference picture the tests name is
checked in and tracked, that the test file writes a reference only under
REGENERATE=1 (BAR-004's rule), and that an altered reference fails the
comparison.

PIX-006 binds on this host: the catalog's Input page and a page of 192 looks
at 240 by 60 cells, in a release build on the hardware adapter, with Kitty
graphics through shared memory and with Sixel, all send their first pictures
within a second and keep the App's work per frame and its wait in present
under 16.6 ms at the 95th percentile while a key is typed or a slider dragged
every frame, repainting the changed look alone. The same run on the macOS
host and the Windows tablet is recorded, not binding.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import JOBS, ROOT, cargo_test_filtered, report, run
from charts_goldens import regeneration_problems
from host_builds import CONFIG, HOSTS, Unreachable, run_on, snapshot

FEATURE = ["wgpu-graphics"]
PACKAGE = "reactive-tui"
TEST = "tests/pixel_looks.rs"
PICTURES = ROOT / "tests/snapshots/pixel-looks"
REFERENCES = {
    "PIX-003": "pix_003_",
    "PIX-004": "pix_004_",
    "PIX-005": "pix_005_",
}


def reference_names() -> list[str]:
    """The reference pictures the test file names: the first argument of each
    `case("name", ...)`."""
    text = (ROOT / TEST).read_text(errors="replace")
    return sorted(set(re.findall(r'case\(\s*"([a-z0-9_]+)",', text)))


def reference_problems() -> list[str]:
    """Every reference picture named exists, is tracked by git, and is a PNG."""
    problems = []
    tracked = set(subprocess.run(["git", "ls-files", "tests/snapshots/pixel-looks"], cwd=ROOT,
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


def altered_reference_problems() -> list[str]:
    """A reference picture that differs beyond GFX-002's tolerance fails the
    comparison: the primary button's reference is replaced by a flat picture
    of its size in a copy of the snapshots, and its test must fail there."""
    original = PICTURES / "primary_default.png"
    if not original.is_file():
        return []
    try:
        from PIL import Image
    except ImportError:
        return ["Pillow is not installed, so the altered reference picture could not be made"]
    with tempfile.TemporaryDirectory(prefix="pixel-looks-altered-") as scratch:
        copy = Path(scratch) / "pixel-looks"
        shutil.copytree(PICTURES, copy)
        with Image.open(original) as image:
            Image.new("RGBA", image.size, (120, 40, 200, 255)).save(copy / "primary_default.png")
        ok, why = cargo_test_filtered("pixel_looks", "pix_003_rounded_containers", features=FEATURE, package=PACKAGE,
                                      env={"REACTIVE_TUI_SNAPSHOTS": scratch})
    if ok:
        return ["the rounded-container test passed against an altered reference picture"]
    if "differs" not in why and "sizes differ" not in why:
        return [f"the rounded-container test failed against an altered reference for another reason: {why}"]
    return []


def test_summary(output: str) -> tuple[bool, str]:
    """Whether one run of the tests passed, with the stated violations."""
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


def speed_lines(output: str) -> str:
    """The lines the PIX-006 run prints with its numbers."""
    found = re.findall(r"PIX-006 .*", output)
    return " / ".join(found) if found else "no PIX-006 line"


class NoDisplay(Exception):
    """No private display can be had: the screenshot part is unverified."""


def screenshot_problem() -> str | None:
    """PIX-001's screenshot: the catalog's Input page in kitty on a private
    X display; the background pixel of a button's label cell equals the
    picture pixel beside it within 2 of 255. Needs Xvfb, kitty and
    ImageMagick's import; raises NoDisplay without them."""
    for tool in ("Xvfb", "kitty", "import"):
        if shutil.which(tool) is None:
            raise NoDisplay(f"{tool} is not installed")
    probe = ROOT / "scripts/pixel-looks-screenshot.py"
    if not probe.is_file():
        raise NoDisplay(f"{probe.relative_to(ROOT)} does not exist yet")
    result = run([sys.executable, "-B", str(probe)], timeout=600)
    text = result.stdout + result.stderr
    if result.returncode == 2:
        raise NoDisplay(text.strip()[-300:])
    if result.returncode != 0:
        return text.strip().splitlines()[-1][:300] if text.strip() else "the screenshot check failed"
    return None


def main() -> int:
    results = {}
    ok, why = cargo_test_filtered("pixel_looks", "pix_001_", features=FEATURE, package=PACKAGE)
    try:
        shot = screenshot_problem()
    except NoDisplay as error:
        print(f"PIX-001 unverified: the screenshot part needs a private display: {error}")
        shot = None
        display = False
    else:
        display = True
    if ok and not display:
        # The tests pass but the screenshot could not be taken: no verdict.
        print("PIX-001 unverified: the tests pass but the screenshot part did not run")
        return 1
    results["PIX-001"] = (ok and shot is None, shot or why)
    results["PIX-002"] = cargo_test_filtered("pixel_looks", "pix_002_", features=FEATURE, package=PACKAGE)
    problems = reference_problems() + regeneration_problems(TEST) + altered_reference_problems()
    for req, substring in REFERENCES.items():
        ok, why = cargo_test_filtered("pixel_looks", substring, features=FEATURE, package=PACKAGE)
        results[req] = (ok and not problems, "; ".join(problems[:6]) or why)
    tests = f"test --locked -p {PACKAGE} --features wgpu-graphics --jobs {JOBS}"
    local = run(["cargo", *f"{tests} --release --test pixel_looks -- --ignored pix_006_ --nocapture".split()],
                timeout=3600, interleave=True)
    print(local.stdout[-3000:])
    speed_ok, speed_why = test_summary(local.stdout)
    recorded = [f"linux: {speed_lines(local.stdout)}; {speed_why}"]
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"no readable test host file at {CONFIG} ({error})")
        config = {}
    if speed_ok:
        with tempfile.TemporaryDirectory(prefix="pixel-looks-") as scratch:
            commit, bundle = snapshot(Path(scratch))
            for name in HOSTS:
                try:
                    _, out = run_on(name, config[name], commit, bundle,
                                    f"{tests} --release --test pixel_looks -- --ignored pix_006_ --nocapture")
                    recorded.append(f"{name}: {speed_lines(out)}")
                except (Unreachable, KeyError) as error:
                    print(f"{name} not reached: {error}")
                    recorded.append(f"{name}: not reached")
    results["PIX-006"] = (speed_ok, "; ".join(recorded))
    ok_all = True
    for req, (ok, why) in results.items():
        ok_all &= report(req, ok, why)
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
