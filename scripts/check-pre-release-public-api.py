#!/usr/bin/env python3
import importlib.util
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
SURFACES = {
    "default": [],
    "minimal": ["--no-default-features"],
    "ffi": ["--no-default-features", "--features", "ffi"],
    "docs.rs": ["--no-default-features", "--features", "async-capabilities,ffi"],
    "embedded-terminal": ["--no-default-features", "--features", "embedded-terminal"],
    "stable-combined": [
        "--features",
        "debug,debug_patches,async-capabilities,ffi,embedded-terminal",
    ],
}
PUBLIC_ITEM = re.compile(
    r"\bpub(?:\([^)]*\))?\s+(?:unsafe\s+)?(?:async\s+)?"
    r"(?:fn|struct|enum|trait|type|const|static|mod)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
TEST_HELPER = re.compile(
    r"\bpub(?:\([^)]*\))?\s+(?:unsafe\s+)?(?:async\s+)?fn\s+"
    r"((?:create|make|build)_test_[A-Za-z0-9_]*)"
)
LEGACY_HELPER = re.compile(
    r"\bpub(?:\([^)]*\))?\s+(?:unsafe\s+)?(?:async\s+)?fn\s+"
    r"([A-Za-z_][A-Za-z0-9_]*_legacy)\s*\("
)


def load_source_tools():
    path = Path(__file__).with_name("check-pre-release-library-diagnostics.py")
    spec = importlib.util.spec_from_file_location("dqc003_source_tools", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load source tools from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SOURCE_TOOLS = load_source_tools()


def outside_ranges(position: int, ranges: list[tuple[int, int]]) -> bool:
    return not any(start <= position < end for start, end in ranges)


def source_inventory(paths: list[Path], root: Path = ROOT) -> dict:
    public_items = []
    test_helpers = []
    legacy_helpers = []
    masked_sources = {}
    for path in paths:
        code = SOURCE_TOOLS.mask_non_code(path.read_text())
        ignored = SOURCE_TOOLS.test_only_ranges(code)
        masked_sources[path] = code
        for match in PUBLIC_ITEM.finditer(code):
            if outside_ranges(match.start(), ignored):
                public_items.append(f"{path.relative_to(root)}:{match.group(1)}")
        for match in TEST_HELPER.finditer(code):
            if outside_ranges(match.start(), ignored):
                test_helpers.append(f"{path.relative_to(root)}:{match.group(1)}")
        for match in LEGACY_HELPER.finditer(code):
            if outside_ranges(match.start(), ignored):
                legacy_helpers.append(
                    {"path": str(path.relative_to(root)), "name": match.group(1)}
                )
    for helper in legacy_helpers:
        helper["references"] = sum(
            len(re.findall(rf"\b{re.escape(helper['name'])}\b", code))
            for code in masked_sources.values()
        ) - 1
    return {
        "public_items": public_items,
        "test_helpers": sorted(test_helpers),
        "legacy_helpers": legacy_helpers,
    }


def load_retentions(path: Path) -> tuple[dict[str, str], list[str]]:
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        return {}, [f"retention manifest is unreadable: {error}"]
    if data.get("version") != 1 or not isinstance(data.get("retentions"), list):
        return {}, ["retention manifest schema is invalid"]
    retentions = {}
    errors = []
    for item in data["retentions"]:
        if not isinstance(item, dict) or not item.get("symbol") or not item.get("decision"):
            errors.append("retention entry is missing symbol or decision")
            continue
        symbol, decision = item["symbol"], item["decision"]
        decision_path = ROOT / "docs" / "decisions" / f"{decision}.md"
        if not decision_path.is_file():
            errors.append(f"retention decision is missing: {decision}")
        else:
            record = decision_path.read_text()
            if symbol not in record or "Status: Superseded" in record:
                errors.append(f"retention decision does not actively name {symbol}: {decision}")
        retentions[symbol] = decision
    return retentions, errors


def rustdoc_command(arguments: list[str]) -> list[str]:
    return ["cargo", "+1.91.0", "doc", "--locked", "--lib", "--no-deps", *arguments]


def run_rustdoc(arguments: list[str]) -> subprocess.CompletedProcess:
    env = {**os.environ, "RUSTDOCFLAGS": "-D missing_docs"}
    return subprocess.run(
        rustdoc_command(arguments),
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        timeout=900,
        check=False,
    )


def validate_observation(observation: dict) -> list[str]:
    errors = []
    if observation.get("validator_tests") is not True:
        errors.append("validator tests failed")
    for surface, passed in observation.get("rustdoc_surfaces", {}).items():
        if passed is not True:
            errors.append(f"rustdoc failed for {surface}")
    for diagnostic in observation.get("missing_docs", []):
        errors.append(f"rustdoc missing documentation: {diagnostic}")
    if observation.get("public_item_count", 0) < 1:
        errors.append("public API inventory is empty")
    for helper in observation.get("test_helpers", []):
        errors.append(f"production test helper: {helper}")
    retentions = observation.get("retentions", {})
    legacy_names = {helper["name"] for helper in observation.get("legacy_helpers", [])}
    for helper in observation.get("legacy_helpers", []):
        if helper.get("references", 0) == 0 and helper["name"] not in retentions:
            errors.append(f"unreferenced legacy helper: {helper['name']}")
    for symbol in retentions:
        if symbol not in legacy_names:
            errors.append(f"stale retention entry: {symbol}")
    errors.extend(observation.get("retention_errors", []))
    return errors


def make_observation() -> tuple[dict, dict[str, subprocess.CompletedProcess]]:
    unit = subprocess.run(
        [sys.executable, "-B", "scripts/test-pre-release-public-api.py"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    results = {name: run_rustdoc(args) for name, args in SURFACES.items()}
    inventory = source_inventory(SOURCE_TOOLS.library_source_files())
    retentions, retention_errors = load_retentions(ROOT / "public-api-retention.toml")
    missing_docs = []
    for name, result in results.items():
        if result.returncode:
            missing_docs.extend(
                f"{name}: {line.strip()}"
                for line in result.stderr.splitlines()
                if "missing documentation" in line
            )
    observation = {
        "validator_tests": unit.returncode == 0,
        "rustdoc_surfaces": {name: result.returncode == 0 for name, result in results.items()},
        "missing_docs": missing_docs,
        "public_item_count": len(inventory["public_items"]),
        "test_helpers": inventory["test_helpers"],
        "legacy_helpers": inventory["legacy_helpers"],
        "retentions": retentions,
        "retention_errors": retention_errors,
    }
    return observation, {"validator tests": unit, **results}


def report(observation: dict, errors: list[str], results: dict) -> None:
    if errors:
        print("DQC-005 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        for name, result in results.items():
            if result.returncode:
                print(f"--- {name} stderr ---", file=sys.stderr)
                print(result.stderr[-5000:], file=sys.stderr)
        return
    print("DQC-005 rustdoc surfaces: " + ", ".join(observation["rustdoc_surfaces"]))
    print(f"DQC-005 inventoried {observation['public_item_count']} shipped public declarations")
    print("DQC-005 production test helpers: none")
    if observation["legacy_helpers"]:
        for helper in observation["legacy_helpers"]:
            print(f"DQC-005 legacy helper {helper['name']}: {helper['references']} references")
    else:
        print("DQC-005 legacy helpers: none")
    print("DQC-005 retention decisions: " + (", ".join(observation["retentions"].values()) or "none"))
    print("DQC-005 passed: supported public API is documented and production helpers are intentional.")


def main() -> int:
    if sys.argv[1:] != ["DQC-005"]:
        print("usage: check-pre-release-public-api.py DQC-005", file=sys.stderr)
        return 2
    try:
        observation, results = make_observation()
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f"DQC-005 failed: {error}", file=sys.stderr)
        return 1
    errors = validate_observation(observation)
    report(observation, errors, results)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
