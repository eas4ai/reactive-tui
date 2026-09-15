#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-pre-release-dependency-code-quality.py"


def load_checker():
    if not CHECKER.is_file():
        raise AssertionError(f"dependency checker is missing: {CHECKER}")
    spec = importlib.util.spec_from_file_location("dependency_checker", CHECKER)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class ObservationValidationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker()
        self.valid = {
            "cargo_audit": True,
            "cargo_deny_advisories": True,
            "cargo_deny_licenses": True,
            "cargo_deny_sources": True,
            "cargo_deny_bans": True,
            "atty_reachable": False,
            "exceptions": [],
        }

    def test_complete_clean_observation_is_accepted(self):
        self.assertEqual(self.checker.validate_observation(self.valid), [])

    def test_missing_policy_result_is_rejected(self):
        observation = dict(self.valid)
        del observation["cargo_deny_licenses"]
        self.assertIn("missing cargo_deny_licenses result", self.checker.validate_observation(observation))

    def test_failed_command_and_reachable_atty_are_rejected(self):
        observation = dict(self.valid, cargo_audit=False, atty_reachable=True)
        errors = self.checker.validate_observation(observation)
        self.assertIn("cargo_audit failed", errors)
        self.assertIn("atty remains reachable", errors)

    def test_unreviewed_exception_is_rejected(self):
        observation = dict(
            self.valid,
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


class PolicyValidationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker()
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


if __name__ == "__main__":
    unittest.main()
