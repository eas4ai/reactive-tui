#!/usr/bin/env python3
"""Test evidence validation with synthetic records, never the live clipboard."""
import hashlib
import importlib.util
import io
import json
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "clipboard_platform_checker", Path(__file__).with_name("check-clipboard-platforms.py")
)
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.records = Path(self.directory.name)
        for backend, system in CHECKER.PLATFORMS.items():
            output = f"RTUI_CLIPBOARD_PLATFORM_OK backend={backend} cases=5\n1 passed\n".encode()
            process = b"2 passed\n"
            if backend == "windows":
                process += b"RTUI_WINDOWS_ADAPTER_OK\n1 passed\n"
            (self.records / (backend + ".out")).write_bytes(output)
            (self.records / (backend + ".process.out")).write_bytes(process)
            record = {"backend": backend, "system": system, "result": "pass",
                      "committed_inputs": True, "inputs_digest": "fixture-inputs",
                      "output": backend + ".out", "process_output": backend + ".process.out",
                      "output_digest": hashlib.sha256(output).hexdigest(),
                      "process_output_digest": hashlib.sha256(process).hexdigest()}
            (self.records / (backend + ".json")).write_text(json.dumps(record))

    def verify(self):
        with patch.object(CHECKER, "RECORDS", self.records), \
             patch.object(CHECKER, "committed_inputs", return_value=True), \
             patch.object(CHECKER, "input_digest", return_value="fixture-inputs"), \
             redirect_stdout(io.StringIO()):
            CHECKER.verify()

    def alter(self, backend, **changes):
        path = self.records / (backend + ".json")
        record = json.loads(path.read_text())
        record.update(changes)
        path.write_text(json.dumps(record))

    def test_default_evidence_is_retained_outside_deleted_docs_analysis(self):
        self.assertEqual(CHECKER.RECORDS, CHECKER.ROOT / ".cairn/reviews/clipboard-platforms")

    def test_all_synthetic_native_records_pass_validation(self):
        self.verify()

    def test_producer_verifier_mechanism_and_artifact_paths_agree(self):
        relative = str(CHECKER.RECORDS.relative_to(CHECKER.ROOT)).replace("\\", "/")
        declaration = (CHECKER.ROOT / ".cairn/mechanisms/api-clipboard").read_text()
        workflow = (CHECKER.ROOT / ".github/workflows/clipboard-platforms.yml").read_text()
        self.assertIn("  - " + relative + "\n", declaration)
        self.assertIn("path: " + relative + "/${{ matrix.backend }}.*", workflow)
        self.assertIn("include-hidden-files: true", workflow)

    def test_native_desktop_guard_precedes_build_or_clipboard_access(self):
        for backend, system in (("windows", "Windows"), ("macos", "Darwin")):
            args = SimpleNamespace(backend=backend, allow_dirty=False, dedicated_desktop=False)
            with self.subTest(backend=backend), \
                 patch.object(CHECKER.platform, "system", return_value=system), \
                 patch.object(CHECKER, "committed_inputs", return_value=True), \
                 patch.object(CHECKER, "build_probe") as build:
                with self.assertRaisesRegex(RuntimeError, "disposable test desktop"):
                    CHECKER.run_backend(args)
                build.assert_not_called()

    def test_missing_native_backend_is_rejected(self):
        (self.records / "macos.json").unlink()
        with self.assertRaisesRegex(RuntimeError, "missing real Darwin"):
            self.verify()

    def test_stale_inputs_are_rejected(self):
        self.alter("windows", inputs_digest="old-inputs")
        with self.assertRaisesRegex(RuntimeError, "stale or invalid"):
            self.verify()

    def test_wrong_native_system_is_rejected(self):
        self.alter("windows", system="Linux")
        with self.assertRaisesRegex(RuntimeError, "stale or invalid"):
            self.verify()

    def test_dirty_native_records_are_rejected(self):
        self.alter("windows", committed_inputs=False)
        with self.assertRaisesRegex(RuntimeError, "stale or invalid"):
            self.verify()

    def test_damaged_output_is_rejected(self):
        (self.records / "xsel.out").write_bytes(b"different output")
        with self.assertRaisesRegex(RuntimeError, "damaged platform output"):
            self.verify()

    def test_parent_directory_output_is_rejected(self):
        self.alter("macos", output="../outside.out")
        with self.assertRaisesRegex(RuntimeError, "local filename"):
            self.verify()

    def test_missing_execution_marker_is_rejected_even_with_matching_hash(self):
        output = b"1 passed\n"
        (self.records / "wayland.out").write_bytes(output)
        self.alter("wayland", output_digest=hashlib.sha256(output).hexdigest())
        with self.assertRaisesRegex(RuntimeError, "lacks executed behavior"):
            self.verify()

    def test_missing_windows_adapter_is_rejected_even_with_matching_hash(self):
        output = b"2 passed\n"
        (self.records / "windows.process.out").write_bytes(output)
        self.alter("windows", process_output_digest=hashlib.sha256(output).hexdigest())
        with self.assertRaisesRegex(RuntimeError, "Windows adapter checks"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
