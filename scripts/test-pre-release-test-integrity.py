#!/usr/bin/env python3
from pathlib import Path
import subprocess
import sys
import unittest


class MechanismScaffoldTests(unittest.TestCase):
    def test_scaffold_cannot_claim_success(self):
        checker = Path(__file__).with_name("check-pre-release-test-integrity.py")
        result = subprocess.run(
            [sys.executable, "-B", str(checker), "DQC-004"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("mechanism is not implemented", result.stderr)


if __name__ == "__main__":
    unittest.main()
