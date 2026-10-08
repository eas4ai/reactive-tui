#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import tempfile
import unittest


def load_checker():
    path = Path(__file__).with_name("check-pre-release-public-api.py")
    spec = importlib.util.spec_from_file_location("dqc005_checker", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load checker from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CHECKER = load_checker()


def clean_observation() -> dict:
    return {
        "validator_tests": True,
        "rustdoc_surfaces": {"default": True, "ffi": True},
        "missing_docs": [],
        "public_item_count": 1,
        "test_helpers": [],
        "legacy_helpers": [],
        "retentions": {},
        "retention_errors": [],
    }


class ValidatorTests(unittest.TestCase):
    def test_clean_observation_passes(self):
        self.assertEqual(CHECKER.validate_observation(clean_observation()), [])

    def test_missing_documentation_is_rejected(self):
        observation = clean_observation()
        observation["missing_docs"] = ["default: missing documentation for a function"]
        self.assertIn(
            "rustdoc missing documentation: default: missing documentation for a function",
            CHECKER.validate_observation(observation),
        )

    def test_failed_rustdoc_surface_is_rejected(self):
        observation = clean_observation()
        observation["rustdoc_surfaces"]["ffi"] = False
        self.assertIn("rustdoc failed for ffi", CHECKER.validate_observation(observation))

    def test_production_test_helper_is_rejected(self):
        observation = clean_observation()
        observation["test_helpers"] = ["src/lib.rs:create_test_value"]
        self.assertIn(
            "production test helper: src/lib.rs:create_test_value",
            CHECKER.validate_observation(observation),
        )

    def test_unreferenced_legacy_helper_is_rejected(self):
        observation = clean_observation()
        observation["legacy_helpers"] = [{"name": "color_legacy", "references": 0}]
        self.assertIn(
            "unreferenced legacy helper: color_legacy",
            CHECKER.validate_observation(observation),
        )

    def test_stale_retention_is_rejected(self):
        observation = clean_observation()
        observation["retentions"] = {"removed_legacy": "DEC-999"}
        self.assertIn(
            "stale retention entry: removed_legacy",
            CHECKER.validate_observation(observation),
        )

    def test_retention_error_is_rejected(self):
        observation = clean_observation()
        observation["retention_errors"] = ["retention decision is missing: DEC-999"]
        self.assertIn(
            "retention decision is missing: DEC-999",
            CHECKER.validate_observation(observation),
        )


class SourceInventoryTests(unittest.TestCase):
    def test_test_only_helper_is_excluded_but_shipped_helper_is_found(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text(
                "#[cfg(test)] pub fn create_test_hidden() {}\n"
                "pub fn create_test_shipped() {}\n"
            )
            inventory = CHECKER.source_inventory([source], root)
            self.assertEqual(inventory["test_helpers"], ["lib.rs:create_test_shipped"])

    def test_legacy_reference_count_excludes_declaration(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text("pub fn color_legacy() {}\nfn caller() { color_legacy(); }\n")
            inventory = CHECKER.source_inventory([source], root)
            self.assertEqual(inventory["legacy_helpers"][0]["references"], 1)

    def test_bad_retention_schema_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "retentions.toml"
            path.write_text("version = 2\n")
            _, errors = CHECKER.load_retentions(path)
            self.assertEqual(errors, ["retention manifest schema is invalid"])

    def test_rustdoc_commands_deny_missing_docs(self):
        self.assertEqual(CHECKER.rustdoc_command([])[0:3], ["cargo", "+1.91.0", "doc"])
        self.assertEqual(len(CHECKER.SURFACES), 6)


if __name__ == "__main__":
    unittest.main()
