#!/usr/bin/env python3
"""Verify declaration relocation, not the underlying acceptance behaviors."""
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
MECHANISMS = (
    "maintenance-format", "maintenance-lint", "maintenance-ffi",
    "default-suite-repair", "registry-cache-isolation", "registry-concurrency",
    "app-wakeups", "embedded-terminal", "suprtui-renderer",
)


class InheritedPathsTests(unittest.TestCase):
    def run_fixture(self, omit_input=False):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "scripts").mkdir()
            (root / ".cairn/mechanisms").mkdir(parents=True)
            script = root / "scripts/check-inherited-abi.py"
            script.write_text((ROOT / "scripts/check-inherited-abi.py").read_text())
            requirements = [f"{prefix}-{number:03}" for prefix, count in
                            (("MNT", 4), ("DFT", 4), ("CCH", 2), ("REG", 3),
                             ("WAK", 5), ("EMB", 6), ("RND", 6))
                            for number in range(1, count + 1)]
            combined = "inputs:\n  - scripts\n"
            for index, name in enumerate(MECHANISMS):
                declaration = ("command: python3 -c \"print('ran " + name + "')\"\n" +
                               "inputs:\n  - scripts\nrequirements:\n")
                if index == 0:
                    declaration += "".join("  - " + item + "\n" for item in requirements)
                (root / ".cairn/mechanisms" / name).write_text(declaration)
                if not omit_input or index != 0:
                    combined += "  - .cairn/mechanisms/" + name + "\n"
            combined += "requirements:\n  - ABI-004\n"
            (root / ".cairn/mechanisms/binding-inherited").write_text(combined)
            return subprocess.run(["python3", "-B", str(script)], cwd=root,
                                  text=True, capture_output=True, timeout=30, check=False)

    def test_extensionless_declarations_execute_every_selected_command(self):
        result = self.run_fixture()
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in MECHANISMS:
            self.assertIn("ran " + name, result.stdout)

    def test_missing_declared_dependency_still_fails(self):
        result = self.run_fixture(omit_input=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Combined declaration omits inputs", result.stderr)


if __name__ == "__main__":
    unittest.main()
