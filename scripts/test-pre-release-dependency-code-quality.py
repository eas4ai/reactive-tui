#!/usr/bin/env python3
from pathlib import Path
import tempfile
import tomllib
import unittest

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-pre-release-dependency-code-quality.py"


def clean_observation():
    return {
        "cargo_audit": True,
        "cargo_deny_advisories": True,
        "cargo_deny_licenses": True,
        "cargo_deny_sources": True,
        "cargo_deny_bans": True,
        "atty_check": True,
        "atty_reachable": False,
        "exceptions": [],
    }


class ObservationValidationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(CHECKER, "dependency_checker", "dependency checker")

    def assert_missing_result_rejected(self, name):
        observation = clean_observation()
        del observation[name]
        self.assertIn(f"missing {name} result", self.checker.validate_observation(observation))

    def test_complete_clean_observation_is_accepted(self):
        self.assertEqual(self.checker.validate_observation(clean_observation()), [])

    def test_missing_policy_result_is_rejected(self):
        self.assert_missing_result_rejected("cargo_deny_licenses")

    def test_missing_atty_enumeration_is_rejected(self):
        self.assert_missing_result_rejected("atty_check")

    def test_failed_command_and_reachable_atty_are_rejected(self):
        observation = dict(clean_observation(), cargo_audit=False, atty_reachable=True)
        errors = self.checker.validate_observation(observation)
        self.assertIn("cargo_audit failed", errors)
        self.assertIn("atty remains reachable", errors)

    def test_unreviewed_exception_is_rejected(self):
        observation = dict(
            clean_observation(),
            exceptions=[
                {
                    "advisory": "RUSTSEC-2099-0001",
                    "package": "example",
                    "version": "1.0.0",
                    "decision": "",
                }
            ],
        )
        self.assertIn(
            "RUSTSEC-2099-0001 example 1.0.0 lacks a reviewed decision",
            self.checker.validate_observation(observation),
        )


class ExceptionDecisionTests(unittest.TestCase):
    def test_exception_decision_names_exact_exposure_and_mitigation(self):
        checker = load_checker(CHECKER, "dependency_checker", "dependency checker")
        exception = {
            "advisory": "RUSTSEC-2099-0001",
            "package": "example",
            "version": "1.0.0",
            "decision": "review-example",
        }
        self.assertFalse(checker.exception_decision_is_complete(exception, "Mitigation: upgrade later"))
        self.assertTrue(
            checker.exception_decision_is_complete(
                exception,
                "RUSTSEC-2099-0001 example 1.0.0\nExposure: parser is reachable\nMitigation: input is capped",
            )
        )


class PolicyValidationTests(unittest.TestCase):
    def test_graphics_exceptions_are_exact_and_keep_the_global_gates(self):
        with (ROOT / "deny.toml").open("rb") as handle:
            policy = tomllib.load(handle)
        self.assertIn("x86_64-pc-windows-gnu", policy["graph"]["targets"])
        self.assertNotIn("CC0-1.0", policy["licenses"]["allow"])
        self.assertEqual(
            policy["licenses"].get("exceptions"),
            [{"crate": "hexf-parse@=0.2.1", "allow": ["CC0-1.0"]}],
        )
        self.assertEqual(policy["bans"]["multiple-versions"], "deny")
        self.assertEqual(policy["bans"]["skip-tree"], [])
        self.assertEqual(policy["advisories"]["ignore"], [])
        decision = "retain-reviewed-wgpu-27-transitive-policy-exceptions"
        graphics_skips = {
            item["crate"] for item in policy["bans"]["skip"]
            if f"decision: {decision};" in item.get("reason", "")
        }
        self.assertEqual(graphics_skips, {
            "foldhash@=0.1.5", "hashbrown@=0.15.5", "hashbrown@=0.16.1",
            "rustc-hash@=1.1.0", "thiserror@=1.0.69", "thiserror-impl@=1.0.69",
        })

    def setUp(self):
        self.checker = load_checker(CHECKER, "dependency_checker", "dependency checker")
        self.valid = {
            "advisories": {"unsound": "all", "ignore": []},
            "licenses": {"allow": ["MIT"]},
            "bans": {"multiple-versions": "deny", "skip": []},
            "sources": {"unknown-registry": "deny", "unknown-git": "deny"},
        }

    def test_required_policy_sections_are_enforced(self):
        policy = dict(self.valid)
        del policy["sources"]
        self.assertIn("missing [sources] section", self.checker.validate_policy(policy))

    def test_unsound_and_unknown_sources_must_be_denied(self):
        policy = dict(self.valid)
        policy["advisories"] = {"unsound": "workspace", "ignore": []}
        policy["sources"] = {"unknown-registry": "warn", "unknown-git": "deny"}
        errors = self.checker.validate_policy(policy)
        self.assertIn("advisories.unsound must be all", errors)
        self.assertIn("sources.unknown-registry must be deny", errors)

    def test_duplicate_policy_requires_deny_with_explicit_skips(self):
        policy = dict(self.valid)
        policy["bans"] = {"multiple-versions": "warn"}
        errors = self.checker.validate_policy(policy)
        self.assertIn("bans.multiple-versions must be deny", errors)
        self.assertIn("bans.skip must be a list", errors)

    def test_security_commands_name_the_locked_production_graph(self):
        self.assertEqual(
            self.checker.deny_command("licenses"),
            ["cargo", "deny", "--locked", "--exclude-dev", "check", "licenses"],
        )
        self.assertEqual(
            self.checker.audit_command(),
            ["cargo", "audit", "-D", "unsound", "--file", "Cargo.lock", "--json"],
        )


class ArtifactReportTests(unittest.TestCase):
    def test_report_identifies_each_artifact_by_path_and_digest(self):
        checker = load_checker(CHECKER, "dependency_checker", "dependency checker")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lockfile = root / "Cargo.lock"
            policy = root / "deny.toml"
            lockfile.write_text("lock\n")
            policy.write_text("policy\n")

            self.assertEqual(
                checker.artifact_report((lockfile, policy)),
                [
                    f"{lockfile}: sha256:d8c9f2728aa278ebcd33ccedf3ad309a866870ad5fb93a03526b4b7655c9e911",
                    f"{policy}: sha256:c82fc52c78bf8154d4dd7d8766c422ab151052d72098d72ded237aafcd78e4e0",
                ],
            )


if __name__ == "__main__":
    unittest.main()
