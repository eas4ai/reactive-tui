#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest


def load_checker():
    path = Path(__file__).with_name("check-pre-release-test-integrity.py")
    spec = importlib.util.spec_from_file_location("dqc004_checker", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load checker from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CHECKER = load_checker()


def clean_observation() -> dict:
    return {
        "commands": {"validator": True, "default": True, "ffi": True},
        "compiled_targets": ["one_test"],
        "missing_targets": [],
        "disconnected_tests": [],
        "ffi_modules": ["src/ffi/mod.rs"],
        "unreachable_ffi_modules": [],
        "unsafe_hooks": [],
        "mutations": [
            {
                "target": "one_test",
                "function": "checks_result",
                "assertions": 1,
                "original_passed": True,
                "mutant_failed": True,
                "assertion_failed": True,
            }
        ],
    }


class ValidatorTests(unittest.TestCase):
    def assert_rejected(self, key, value, message):
        observation = clean_observation()
        observation[key] = value
        self.assertIn(message, CHECKER.validate_observation(observation))

    def test_clean_observation_passes(self):
        self.assertEqual(CHECKER.validate_observation(clean_observation()), [])

    def test_failed_command_is_rejected(self):
        self.assert_rejected("commands", {"default": False}, "command failed: default")

    def test_missing_target_is_rejected(self):
        self.assert_rejected(
            "missing_targets", ["named_test"], "Cargo target was not compiled: named_test"
        )

    def test_disconnected_source_is_rejected(self):
        self.assert_rejected(
            "disconnected_tests",
            ["tests/lost.rs"],
            "disconnected integration test source: tests/lost.rs",
        )

    def test_unreachable_ffi_module_is_rejected(self):
        self.assert_rejected(
            "unreachable_ffi_modules",
            ["src/ffi/lost.rs"],
            "unreachable FFI module: src/ffi/lost.rs",
        )

    def test_unsafe_global_hook_is_rejected(self):
        self.assert_rejected(
            "unsafe_hooks",
            [{"path": "src/lib.rs", "line": 7, "kind": "mutable global", "name": "HOOK"}],
            "src/lib.rs:7 exposes mutable global HOOK",
        )

    def test_assertionless_sample_is_rejected(self):
        observation = clean_observation()
        observation["mutations"][0]["assertions"] = 0
        self.assertIn(
            "sample has no observable assertion: one_test::checks_result",
            CHECKER.validate_observation(observation),
        )

    def test_passing_mutant_is_rejected(self):
        observation = clean_observation()
        observation["mutations"][0]["mutant_failed"] = False
        self.assertIn(
            "assertion mutant survived: one_test::checks_result",
            CHECKER.validate_observation(observation),
        )

    def test_non_assertion_mutant_failure_is_rejected(self):
        observation = clean_observation()
        observation["mutations"][0]["assertion_failed"] = False
        self.assertIn(
            "mutant failed outside its assertion: one_test::checks_result",
            CHECKER.validate_observation(observation),
        )


class SourceAnalysisTests(unittest.TestCase):
    def test_root_package_accepts_no_deps_metadata(self):
        package = {"id": "reactive-tui 0.1.0", "name": "reactive-tui"}
        self.assertIs(CHECKER.root_package({"packages": [package], "resolve": None}), package)

    def test_source_targets_include_explicit_examples_below_tests(self):
        package = {
            "targets": [
                {
                    "name": "probe",
                    "src_path": str(CHECKER.ROOT / "tests" / "probe.rs"),
                    "kind": ["example"],
                },
                {
                    "name": "library",
                    "src_path": str(CHECKER.ROOT / "src" / "lib.rs"),
                    "kind": ["lib"],
                },
            ]
        }
        self.assertEqual(set(CHECKER.source_targets_below_tests(package)), {"probe"})

    def test_rust_source_inventory_is_recursive_and_excludes_other_languages(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "nested"
            nested.mkdir()
            rust = nested / "module.rs"
            rust.write_text("")
            (nested / "helper.py").write_text("")
            self.assertEqual(CHECKER.rust_source_inventory(root), {rust})

    def test_reachability_follows_real_mod_declarations_only(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            support = root / "support"
            support.mkdir()
            entry = root / "entry.rs"
            child = root / "child.rs"
            helper = support / "helper.rs"
            (root / "ignored.rs").write_text("")
            entry.write_text(
                'mod child;\n#[path = "support/helper.rs"] mod helper;\n'
                '// mod ignored;\nconst TEXT: &str = "mod ignored;";\n'
            )
            child.write_text("")
            helper.write_text("")
            self.assertEqual(CHECKER.reachable_modules([entry]), {entry, child, helper})

    def test_nested_module_children_resolve_below_the_parent_module(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            entry = root / "entry.rs"
            parent = root / "parent.rs"
            nested = root / "parent" / "nested.rs"
            nested.parent.mkdir()
            entry.write_text("mod parent;\n")
            parent.write_text("mod nested;\n")
            nested.write_text("")
            self.assertEqual(CHECKER.reachable_modules([entry]), {entry, parent, nested})

    def test_unsafe_hook_scan_ignores_test_only_code(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text(
                "#[cfg(test)] mod tests { static mut SAFE_IN_TESTS: usize = 0; }\n"
                "static mut SHIPPED: usize = 0;\n"
            )
            findings = CHECKER.unsafe_test_hooks([source], root)
            self.assertEqual([finding["name"] for finding in findings], ["SHIPPED"])

    def test_assertion_count_is_scoped_to_named_test(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "sample.rs"
            source.write_text(
                "#[test] fn empty() {}\n"
                "#[test] fn observed() { assert_eq!(2 + 2, 4); }\n"
            )
            self.assertEqual(CHECKER.assertion_count(source, "empty"), 0)
            self.assertEqual(CHECKER.assertion_count(source, "observed"), 1)

    def test_cargo_compile_command_is_locked_and_no_run(self):
        command = CHECKER.cargo_command("ffi")
        self.assertIn("--locked", command)
        self.assertIn("--no-run", command)
        self.assertEqual(command[-2:], ["--features", "ffi"])

    def test_assertion_failure_requires_assertion_diagnostic(self):
        assertion = subprocess.CompletedProcess([], 101, "assertion `left != right` failed", "")
        compile_error = subprocess.CompletedProcess([], 101, "", "could not compile")
        self.assertTrue(CHECKER.assertion_failure(assertion))
        self.assertFalse(CHECKER.assertion_failure(compile_error))


if __name__ == "__main__":
    unittest.main()
