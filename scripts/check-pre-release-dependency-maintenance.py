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
GRAPH_NAMES = ("default", "minimal", "ffi", "markdown")


def validate_observation(observation: dict) -> list[str]:
    errors = []
    if observation.get("audit") is not True:
        errors.append("cargo audit failed")

    graphs = observation.get("graphs", {})
    for name in GRAPH_NAMES:
        if graphs.get(name) is not True:
            errors.append(f"{name} dependency graph failed")

    for finding in observation.get("unmaintained", []):
        if not finding.get("decision"):
            errors.append(
                f"{finding['advisory']} {finding['package']} {finding['version']} "
                "lacks a reviewed maintenance decision"
            )

    versions = observation.get("versions", {})
    reviews = observation.get("duplicate_reviews", {})
    for package in ("taffy", "vte"):
        package_versions = versions.get(package, [])
        if not package_versions:
            errors.append(f"dependency graphs did not report {package}")
        if len(package_versions) > 1 and not reviews.get(package):
            errors.append(
                f"duplicate {package} versions lack a reviewed decision: "
                + ", ".join(package_versions)
            )

    if observation.get("crossterm_packages") != ["reactive-tui-crossterm"]:
        errors.append("maintained crossterm fork identity is ambiguous")
    if observation.get("onig_default") is not False:
        errors.append("default dependency graph contains onig_sys")
    return errors


def package_versions(tree: str, package: str) -> list[str]:
    pattern = rf"(?m)^{re.escape(package)} v([^\s]+)(?:\s|$)"
    return sorted(set(re.findall(pattern, tree)), key=semver_key)


def semver_key(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.split("."))


def graph_commands() -> dict[str, list[str]]:
    base = [
        "cargo",
        "+1.91.0",
        "tree",
        "--locked",
        "--target",
        "all",
        "-p",
        "reactive-tui",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ]
    production = ["--edges", "normal,build"]
    return {
        "default": [*base, *production],
        "minimal": [*base, *production, "--no-default-features"],
        "ffi": [*base, *production, "--no-default-features", "--features", "ffi"],
        "markdown": [*base, "--no-default-features", "-e", "features"],
    }


def duplicate_decision_is_complete(package: str, versions: list[str], text: str) -> bool:
    required = [*(f"{package} {version}" for version in versions), "Reason:"]
    return all(value in text for value in required)


def run(
    name: str, command: list[str], *, echo_output: bool = False
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
    if completed.returncode != 0 or echo_output:
        if completed.stdout:
            print(completed.stdout, end="")
        if completed.stderr:
            print(completed.stderr, end="", file=sys.stderr)
    return completed


def packages(tree: str) -> set[tuple[str, str]]:
    return set(re.findall(r"(?m)^([A-Za-z0-9_-]+) v([^\s]+)(?:\s|$)", tree))


def decision_text(slug: str) -> str:
    if not slug:
        return ""
    path = ROOT / "docs" / "decisions" / f"{slug}.md"
    if not path.is_file():
        return ""
    text = path.read_text()
    if "Superseded by:" in text or "Level:" not in text:
        return ""
    return text


def unmaintained_findings(
    policy: dict, audit: dict, production_packages: set[tuple[str, str]]
) -> list[dict[str, str]]:
    reviews = {
        (item.get("advisory"), item.get("package"), item.get("version")): item.get(
            "decision", ""
        )
        for item in policy.get("unmaintained", {}).get("exceptions", [])
    }
    findings = []
    for finding in audit.get("warnings", {}).get("unmaintained", []):
        advisory = finding.get("advisory", {}).get("id", "unknown")
        package = finding.get("package", {})
        name = package.get("name", "unknown")
        version = package.get("version", "unknown")
        if (name, version) not in production_packages:
            continue
        slug = reviews.get((advisory, name, version), "")
        text = decision_text(slug)
        required = (advisory, name, version, "Exposure:", "Mitigation:")
        findings.append(
            {
                "advisory": advisory,
                "package": name,
                "version": version,
                "decision": slug if all(value in text for value in required) else "",
            }
        )
    return findings


def duplicate_review(policy: dict, package: str, versions: list[str]) -> str:
    if len(versions) < 2:
        return ""
    for exception in policy.get("duplicates", {}).get("exceptions", []):
        if exception.get("package") != package or exception.get("versions") != versions:
            continue
        slug = exception.get("decision", "")
        if duplicate_decision_is_complete(package, versions, decision_text(slug)):
            return slug
    return ""


def check_dqc_002() -> int:
    unit = run(
        "validator tests",
        [sys.executable, "-B", "scripts/test-pre-release-dependency-maintenance.py"],
        echo_output=True,
    )
    with (ROOT / "dependency-maintenance.toml").open("rb") as handle:
        policy = tomllib.load(handle)

    graph_results = {
        name: run(f"{name} dependency graph", command)
        for name, command in graph_commands().items()
    }
    audit = run("cargo audit", ["cargo", "audit", "--file", "Cargo.lock", "--json"])
    try:
        audit_report = json.loads(audit.stdout)
    except json.JSONDecodeError:
        audit_report = {}

    production_packages = set().union(
        *(packages(graph_results[name].stdout) for name in ("default", "minimal", "ffi"))
    )
    default_packages = packages(graph_results["default"].stdout)
    versions = {
        package: sorted(
            {version for name, version in production_packages if name == package},
            key=semver_key,
        )
        for package in ("taffy", "vte")
    }
    crossterm_packages = sorted(
        {
            name
            for name, _ in production_packages
            if name in {"crossterm", "reactive-tui-crossterm"}
        }
    )
    observation = {
        "audit": (
            audit.returncode == 0
            and isinstance(audit_report.get("warnings"), dict)
            and "unmaintained" in audit_report["warnings"]
        ),
        "graphs": {
            name: completed.returncode == 0 and bool(packages(completed.stdout))
            for name, completed in graph_results.items()
        },
        "unmaintained": unmaintained_findings(policy, audit_report, production_packages),
        "versions": versions,
        "duplicate_reviews": {
            package: duplicate_review(policy, package, package_versions)
            for package, package_versions in versions.items()
        },
        "crossterm_packages": crossterm_packages,
        "onig_default": any(name == "onig_sys" for name, _ in default_packages),
    }
    errors = []
    if unit.returncode != 0:
        errors.append("validator tests failed")
    errors.extend(validate_observation(observation))
    if errors:
        print("DQC-002 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    lock_digest = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    print(f"DQC-002 audited Cargo.lock: sha256:{lock_digest}")
    for name, completed in graph_results.items():
        digest = hashlib.sha256(completed.stdout.encode()).hexdigest()
        print(f"- {name}: sha256:{digest}; packages={len(packages(completed.stdout))}")
    print(f"- taffy: {', '.join(versions['taffy'])}")
    print(f"- vte: {', '.join(versions['vte'])}")
    print("DQC-002 passed: maintenance decisions, identities, and feature graphs are clean.")
    return 0


def main() -> int:
    if sys.argv[1:] != ["DQC-002"]:
        print("usage: check-pre-release-dependency-maintenance.py DQC-002", file=sys.stderr)
        return 2
    return check_dqc_002()


if __name__ == "__main__":
    raise SystemExit(main())
