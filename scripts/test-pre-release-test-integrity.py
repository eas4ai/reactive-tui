#!/usr/bin/env python3
import importlib.util
import io
import json
import os
from contextlib import redirect_stderr
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


def load_checker():
    path = Path(__file__).with_name("check-pre-release-test-integrity.py")
    spec = importlib.util.spec_from_file_location("dqc004_checker", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load checker from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CHECKER = load_checker()
# Synthetic observation paths, not files of this repository.
LOST_TEST = "/".join(["tests", "lost.rs"])
LOST_FFI_MODULE = "/".join(["src", "ffi", "lost.rs"])


def clean_observation() -> dict:
    return {
        "commands": {"validator": True, "default": True, "ffi": True},
        "compiled_targets": ["one_test"],
        "missing_targets": [],
        "disconnected_tests": [],
        "ffi_modules": ["src/ffi/mod.rs"],
        "ffi_exports": [{"path": "src/ffi/mod.rs", "line": 1, "name": "observable_export"}],
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
    def test_empty_export_inventory_is_rejected(self):
        self.assert_rejected("ffi_exports", [], "FFI export inventory is empty")

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
            [LOST_TEST],
            f"disconnected integration test source: {LOST_TEST}",
        )

    def test_unreachable_ffi_module_is_rejected(self):
        self.assert_rejected(
            "unreachable_ffi_modules",
            [LOST_FFI_MODULE],
            f"unreachable FFI module: {LOST_FFI_MODULE}",
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
    def test_production_cfg_evaluation_keeps_unknown_atoms_conservative(self):
        cases = {"test": False, "not(test)": True, "feature = ffi": None,
                 "any(test, feature = ffi)": None, "all(test, feature = ffi)": False,
                 "not(any(test, feature = ffi))": None,
                 "any(test, not(test))": True, "all()": True, "any()": False,
                 "all(any(test, not(test)), feature = ffi)": None}
        for expression, expected in cases.items():
            with self.subTest(expression=expression):
                self.assertIs(CHECKER.production_cfg_value(expression), expected)

    def test_export_inventory_rejects_comment_crossings_and_accepts_declaration_trivia(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "ffi.rs"
            source.write_text(
                'fn ordinary_call() {}\nfn wrapper() {\n'
                ' // extern "C" fn\n ordinary_call();\n}\n'
                'pub extern /* ABI comment */ "C" /* declaration */ fn real_export() {}\n'
                'pub extern /* "C" */ "Rust" fn rust_abi() {}\n'
                'pub extern // "Rust"\n "C" fn line_comment_export() {}\n'
                'const RAW: &str = r#"extern /* fake */ "C" fn raw_export() {}"#;\n'
            )
            exports = CHECKER.ffi_export_inventory({source}, root=root)
            self.assertEqual([item["name"] for item in exports],
                             ["real_export", "line_comment_export"])

    def test_export_inventory_retains_production_cfg_alternatives(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "ffi.rs"
            source.write_text(
                '#[cfg(not(test))] pub extern "C" fn non_test_export() {}\n'
                '#[cfg(any(test, feature = "ffi"))] pub extern "C" fn ffi_export() {}\n'
                '#[cfg(test)] pub extern "C" fn test_export() {}\n'
                '#[cfg(all(test, feature = "ffi"))] pub extern "C" fn test_feature_export() {}\n'
            )
            exports = CHECKER.ffi_export_inventory({source}, root=root)
            self.assertEqual([item["name"] for item in exports],
                             ["non_test_export", "ffi_export"])

    def test_export_inventory_retains_abi_literals_and_excludes_nonproduction_text(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "ffi.rs"
            source.write_text(
                '#[no_mangle]\npub extern "C" fn real_export() {}\n'
                '// pub extern "C" fn comment_export() {}\n'
                'const EXAMPLE: &str = r#"pub extern "C" fn string_export() {}"#;\n'
                '#[cfg(test)] mod tests { pub extern "C" fn test_callback() {} }\n'
            )
            self.assertEqual(CHECKER.ffi_export_inventory({source}, root=root),
                             [{"path": "ffi.rs", "line": 2, "name": "real_export"}])

    def test_feature_graphs_require_their_targets_and_do_not_hide_missing_tests(self):
        targets = [
            {"name": "base", "kind": ["test"], "src_path": str(CHECKER.ROOT / "tests/utility_paint_tests.rs")},
            {"name": "ffi", "kind": ["test"], "src_path": str(CHECKER.ROOT / "tests/ffi_tests.rs"),
             "required-features": ["ffi"]},
            {"name": "gpu", "kind": ["test"], "src_path": str(CHECKER.ROOT / "tests/wgpu_graphics.rs"),
             "required-features": ["wgpu-graphics"]},
        ]
        metadata = {"packages": [{"name": "reactive-tui", "targets": targets}]}
        passed = subprocess.CompletedProcess([], 0, "", "")
        failed_assertion = subprocess.CompletedProcess([], 101, "assertion `left != right` failed", "")

        for omitted in (None, "base", "ffi", "gpu"):
            def compile_result(command, **kwargs):
                features = command[command.index("--features") + 1] if "--features" in command else None
                if "--no-run" in command:
                    self.assertEqual(kwargs["target_dir"], CHECKER.TARGET / (features or "default"))
                names = {"base"} | ({"ffi"} if features == "ffi" else {"gpu"} if features == "wgpu-graphics" else set())
                output = "\n".join(json.dumps({"reason": "compiler-artifact", "target": {"kind": ["test"], "name": name}})
                                   for name in names if name != omitted)
                return subprocess.CompletedProcess(command, 0, output, "")

            with self.subTest(omitted=omitted), patch.object(CHECKER, "cargo_metadata", return_value=metadata), \
                 patch.object(CHECKER, "run", side_effect=compile_result), \
                 patch.object(CHECKER, "run_original", return_value=passed), \
                 patch.object(CHECKER, "run_mutant", return_value=failed_assertion):
                observation, results = CHECKER.make_observation()
            self.assertEqual(observation["missing_targets"], [] if omitted is None else [omitted])
            self.assertIn("graphics test compile", results)

    def test_command_environment_bounds_symbols_but_preserves_checks(self):
        parent = {"DQC004_PARENT_SETTING": "kept", "CARGO_TARGET_DIR": "parent-build",
                  "CARGO_PROFILE_TEST_DEBUG": "2",
                  "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "false"}
        with patch.dict(os.environ, parent, clear=True), patch.object(
            CHECKER.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)
        ) as execute:
            CHECKER.run(["simulated-command"], target_dir=CHECKER.TARGET / "default")
            self.assertEqual(dict(os.environ), parent)
        environment = execute.call_args.kwargs["env"]
        self.assertEqual(environment["DQC004_PARENT_SETTING"], "kept")
        self.assertEqual(environment["CARGO_TARGET_DIR"], str(CHECKER.TARGET / "default"))
        self.assertEqual(environment["CARGO_INCREMENTAL"], "0")
        for profile in ("DEV", "TEST"):
            self.assertEqual(environment[f"CARGO_PROFILE_{profile}_DEBUG"], "0")
            self.assertEqual(environment[f"CARGO_PROFILE_{profile}_DEBUG_ASSERTIONS"], "true")
            self.assertEqual(environment[f"CARGO_PROFILE_{profile}_OVERFLOW_CHECKS"], "true")
        self.assertEqual(parent["CARGO_PROFILE_TEST_DEBUG"], "2")

    def test_original_and_mutant_builds_use_separate_target_directories(self):
        passed = subprocess.CompletedProcess([], 0, "", "")
        with patch.object(CHECKER, "run", return_value=passed) as execute:
            CHECKER.run_original(CHECKER.SAMPLES[0])
            original = execute.call_args.kwargs["target_dir"]
            CHECKER.run_mutant(CHECKER.SAMPLES[0])
            mutant = execute.call_args.kwargs["target_dir"]
        self.assertEqual(original, CHECKER.TARGET / "default")
        self.assertEqual(mutant, CHECKER.TARGET / "mutants")
        self.assertNotEqual(original, mutant)

    def test_failure_report_retains_cargo_json_primary_diagnostics(self):
        message = "error[E0000]: observable compiler failure"
        output = "not json\nnull\n[]\n" + json.dumps({"reason": "compiler-message",
                  "message": {"rendered": message}}) + "\n"
        result = subprocess.CompletedProcess([], 101, output, "could not compile")
        capture = io.StringIO()
        with redirect_stderr(capture):
            CHECKER.report({}, ["command failed"], {"default test compile": result})
        self.assertIn(message, capture.getvalue())
        self.assertIn("could not compile", capture.getvalue())

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

    def test_unsafe_hook_scan_does_not_hide_production_cfg_alternatives(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text(
                '#[cfg(not(test))] mod production { static mut NON_TEST: usize = 0; }\n'
                '#[cfg(any(test, feature = "ffi"))] mod alternate { static mut ALTERNATE: usize = 0; }\n'
                '#[cfg(all(test, feature = "ffi"))] mod tests { static mut TEST_ONLY: usize = 0; }\n'
            )
            findings = CHECKER.unsafe_test_hooks([source], root)
            self.assertEqual([item["name"] for item in findings], ["NON_TEST", "ALTERNATE"])

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
