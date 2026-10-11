#!/usr/bin/env python3
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
REQUIRED_RESULTS = (
    "cargo_audit",
    "cargo_deny_advisories",
    "cargo_deny_licenses",
    "cargo_deny_sources",
    "cargo_deny_bans",
    "atty_check",
    "atty_reachable",
    "exceptions",
)


def validate_observation(observation: dict) -> list[str]:
    errors = []
    for name in REQUIRED_RESULTS:
        if name not in observation:
            errors.append(f"missing {name} result")
    for name in REQUIRED_RESULTS[:6]:
        if name in observation and observation[name] is not True:
            errors.append(f"{name} failed")
    if observation.get("atty_reachable") is True:
        errors.append("atty remains reachable")
    for exception in observation.get("exceptions", []):
        if not exception.get("decision"):
            errors.append(
                f"{exception['advisory']} {exception['package']} "
                f"{exception['version']} lacks a reviewed decision"
            )
    return errors


def exception_decision_is_complete(exception: dict, text: str) -> bool:
    required = (
        exception["advisory"],
        exception["package"],
        exception["version"],
        "Exposure:",
        "Mitigation:",
    )
    return all(value in text for value in required)


def validate_policy(policy: dict) -> list[str]:
    errors = []
    for section in ("advisories", "licenses", "bans", "sources"):
        if section not in policy:
            errors.append(f"missing [{section}] section")

    advisories = policy.get("advisories", {})
    if advisories.get("unsound") != "all":
        errors.append("advisories.unsound must be all")
    if not isinstance(advisories.get("ignore"), list):
        errors.append("advisories.ignore must be a list")

    licenses = policy.get("licenses", {})
    if not isinstance(licenses.get("allow"), list) or not licenses.get("allow"):
        errors.append("licenses.allow must be a nonempty list")

    bans = policy.get("bans", {})
    if bans.get("multiple-versions") != "deny":
        errors.append("bans.multiple-versions must be deny")
    if not isinstance(bans.get("skip"), list):
        errors.append("bans.skip must be a list")

    sources = policy.get("sources", {})
    for field in ("unknown-registry", "unknown-git"):
        if sources.get(field) != "deny":
            errors.append(f"sources.{field} must be deny")
    return errors


def run(
    name: str, command: list[str], *, echo_stdout: bool = True
) -> subprocess.CompletedProcess[str]:
    print(f"== {name} ==", flush=True)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        env={**os.environ, "CARGO_TERM_COLOR": "never"},
        text=True,
        capture_output=True,
        timeout=900,
        check=False,
    )
    if completed.stdout and echo_stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    return completed


def deny_command(check: str) -> list[str]:
    return ["cargo", "deny", "--locked", "--exclude-dev", "check", check]


def audit_command() -> list[str]:
    return ["cargo", "audit", "-D", "unsound", "--file", "Cargo.lock", "--json"]


def artifact_report(paths: tuple[Path, ...]) -> list[str]:
    return [
        f"{path}: sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"
        for path in paths
    ]


def advisory_exceptions(policy: dict, audit: dict) -> list[dict[str, str]]:
    packages = {}
    advisory_groups = [
        audit.get("vulnerabilities", {}).get("list", []),
        *audit.get("warnings", {}).values(),
    ]
    for group in advisory_groups:
        for finding in group:
            advisory = finding.get("advisory", {}).get("id", "unknown")
            package = finding.get("package", {})
            packages[advisory] = (package.get("name", "unknown"), package.get("version", "unknown"))

    exceptions = []
    for item in policy.get("advisories", {}).get("ignore", []):
        if isinstance(item, str):
            advisory = item
            reason = ""
        else:
            advisory = item.get("id", item.get("crate", "unknown"))
            reason = item.get("reason", "")
        decision_match = re.search(r"\bdecision: ([a-z0-9-]+)\b", reason)
        decision = decision_match.group(1) if decision_match else ""
        package, version = packages.get(advisory, ("unknown", "unknown"))
        exception = {
            "advisory": advisory,
            "package": package,
            "version": version,
            "decision": decision,
        }
        decision_path = ROOT / "docs" / "decisions" / f"{decision}.md"
        if not decision or not decision_path.is_file():
            exception["decision"] = ""
        elif not exception_decision_is_complete(exception, decision_path.read_text()):
            exception["decision"] = ""
        exceptions.append(exception)
    return exceptions


def check_dqc_001() -> int:
    unit = run(
        "validator tests",
        [sys.executable, "-B", "scripts/test-pre-release-dependency-code-quality.py"],
    )
    with (ROOT / "deny.toml").open("rb") as handle:
        policy = tomllib.load(handle)
    errors = validate_policy(policy)

    audit = run(
        "cargo audit",
        audit_command(),
        echo_stdout=False,
    )
    try:
        audit_report = json.loads(audit.stdout)
    except json.JSONDecodeError:
        audit_report = {}
        errors.append("cargo audit did not return complete JSON")

    deny_results = {}
    for check in ("advisories", "licenses", "sources", "bans"):
        deny_results[check] = run(
            f"cargo deny {check}", deny_command(check)
        )
    atty = run(
        "atty reachability",
        ["cargo", "tree", "--target", "all", "--edges", "normal,build"],
        echo_stdout=False,
    )

    observation = {
        "cargo_audit": (
            audit.returncode == 0
            and audit_report.get("vulnerabilities", {}).get("count") == 0
            and not audit_report.get("warnings", {}).get("unsound")
        ),
        **{
            f"cargo_deny_{name}": completed.returncode == 0
            for name, completed in deny_results.items()
        },
        "atty_check": atty.returncode == 0,
        "atty_reachable": bool(re.search(r"(?m)^\s*atty v", atty.stdout)),
        "exceptions": advisory_exceptions(policy, audit_report),
    }
    if unit.returncode != 0:
        errors.append("validator tests failed")
    errors.extend(validate_observation(observation))
    if errors:
        print("DQC-001 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("DQC-001 audited artifacts:")
    for artifact in artifact_report((ROOT / "Cargo.lock", ROOT / "deny.toml")):
        print(f"- {artifact}")
    print("DQC-001 passed: locked audit, policy checks, and atty reachability are clean.")
    return 0


def main() -> int:
    if sys.argv[1:] != ["DQC-001"]:
        print("usage: check-pre-release-dependency-code-quality.py DQC-001", file=sys.stderr)
        return 2
    return check_dqc_001()


if __name__ == "__main__":
    raise SystemExit(main())
