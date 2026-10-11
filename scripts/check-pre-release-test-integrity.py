#!/usr/bin/env python3
from dataclasses import asdict, dataclass
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve() / "dqc004"


def load_source_tools():
    path = Path(__file__).with_name("check-pre-release-library-diagnostics.py")
    spec = importlib.util.spec_from_file_location("dqc003_source_tools", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load source tools from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SOURCE_TOOLS = load_source_tools()
MODULE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTRIBUTE = re.compile(r"#\s*\[\s*path\s*=\s*\"([^\"]+)\"\s*\]\s*$")
EXPORTED_C_FN = re.compile(r"(?:#\s*\[\s*no_mangle\s*\]\s*)?(?:pub\s+)?(?:unsafe\s+)?extern\s+\"C\"\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)")
STATIC_MUT = re.compile(r"\bstatic\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)")
RAW_HOOK = re.compile(r"\bpub\s+(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*(?:test|mock|hook|global)[A-Za-z0-9_]*)\s*\([^)]*\*(?:mut|const)\b", re.IGNORECASE | re.DOTALL)
TEST_FN = re.compile(r"#\s*\[\s*test\s*\]\s*(?:#\s*\[[^]]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)\s*\{")
ASSERTION = re.compile(r"\bassert(?:_eq|_ne|_matches)?\s*!|\bdebug_assert(?:_eq|_ne)?\s*!")


@dataclass(frozen=True)
class MutationSample:
    target: str
    source: str
    function: str


SAMPLES = (
    MutationSample(
        "utility_paint_tests",
        "tests/utility_paint_tests.rs",
        "parse_px_accepts_px_suffix_and_raw",
    ),
    MutationSample(
        "production_readiness_test",
        "tests/production_readiness_test.rs",
        "test_effect_cleanup",
    ),
)


def cargo_command(features: str | None = None) -> list[str]:
    command = [
        "cargo",
        "+1.91.0",
        "test",
        "--locked",
        "--tests",
        "--no-run",
        "--message-format=json",
    ]
    if features:
        command.extend(["--features", features])
    return command


def run(command: list[str], *, cwd: Path = ROOT, timeout: int = 900, target_dir: Path | None = None):
    environment = {**os.environ, "CARGO_INCREMENTAL": "0"}
    if target_dir is not None:
        environment["CARGO_TARGET_DIR"] = str(target_dir)
    # Full target inventories need assertions, not multi-gigabyte debug symbols.
    for profile in ("DEV", "TEST"):
        environment[f"CARGO_PROFILE_{profile}_DEBUG"] = "0"
        environment[f"CARGO_PROFILE_{profile}_DEBUG_ASSERTIONS"] = "true"
        environment[f"CARGO_PROFILE_{profile}_OVERFLOW_CHECKS"] = "true"
    return subprocess.run(
        command,
        cwd=cwd,
        env=environment,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )


def cargo_metadata() -> dict:
    result = run(["cargo", "+1.91.0", "metadata", "--locked", "--no-deps", "--format-version=1"])
    if result.returncode != 0:
        raise RuntimeError(result.stderr or "cargo metadata failed")
    return json.loads(result.stdout)


def root_package(metadata: dict) -> dict:
    root_id = (metadata.get("resolve") or {}).get("root")
    packages = metadata.get("packages", [])
    for package in packages:
        if package.get("id") == root_id or package.get("name") == "reactive-tui":
            return package
    raise RuntimeError("reactive-tui package missing from cargo metadata")


def integration_targets(package: dict) -> dict[str, dict]:
    return {
        target["name"]: target
        for target in package.get("targets", [])
        if "test" in target.get("kind", [])
    }


def source_targets_below_tests(package: dict) -> dict[str, dict]:
    tests_root = (ROOT / "tests").resolve()
    targets = {}
    for target in package.get("targets", []):
        path = Path(target["src_path"]).resolve()
        if path.is_relative_to(tests_root):
            targets[target["name"]] = target
    return targets


def compiled_artifacts(output: str) -> set[str]:
    names = set()
    for line in output.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target", {})
        if message.get("reason") == "compiler-artifact" and "test" in target.get("kind", []):
            names.add(target.get("name", ""))
    return names - {""}


def module_children(path: Path, *, crate_root: bool = False) -> list[Path]:
    source = path.read_text()
    code = SOURCE_TOOLS.mask_non_code(source)
    children = []
    for match in MODULE.finditer(code):
        name = match.group(1)
        prefix = source[max(0, match.start() - 300) : match.start()]
        path_attribute = PATH_ATTRIBUTE.search(prefix)
        explicit = None if path_attribute is None else path_attribute.group(1)
        module_root = (
            path.parent if crate_root or path.name == "mod.rs" else path.parent / path.stem
        )
        if explicit:
            candidates = (path.parent / explicit, module_root / explicit)
            candidate = next((item for item in candidates if item.is_file()), candidates[0])
        else:
            direct = module_root / f"{name}.rs"
            nested = module_root / name / "mod.rs"
            candidate = direct if direct.is_file() else nested
        if candidate.is_file():
            children.append(candidate.resolve())
    return children


def reachable_modules(roots: list[Path]) -> set[Path]:
    reachable: set[Path] = set()
    pending = [(path.resolve(), True) for path in roots if path.is_file()]
    while pending:
        path, crate_root = pending.pop()
        if path in reachable:
            continue
        reachable.add(path)
        pending.extend(
            (child, False)
            for child in module_children(path, crate_root=crate_root)
            if child not in reachable
        )
    return reachable


def rust_source_inventory(root: Path) -> set[Path]:
    return {path.resolve() for path in root.rglob("*.rs")}


def ffi_declaration_code(source: str) -> str:
    code = SOURCE_TOOLS.mask_non_code(source)
    masked = list(code)
    # Restore only an actual ABI token after a live extern keyword. Comments
    # may separate the tokens, but their quoted examples are never ABI tokens.
    for match in re.finditer(r"\bextern\b", code):
        index = match.end()
        while index < len(source):
            if source.startswith("//", index):
                index = SOURCE_TOOLS.consume_line_comment(source, masked, index)
            elif source.startswith("/*", index):
                index = SOURCE_TOOLS.consume_block_comment(source, masked, index)
            elif source[index].isspace():
                index += 1
            else:
                break
        if source.startswith('"C"', index):
            masked[index:index + 3] = '"C"'
    return "".join(masked)


def production_cfg_value(expression: str) -> bool | None:
    expression = expression.strip()
    if expression == "test":
        return False
    combination = re.fullmatch(r"(all|any|not)\s*\((.*)\)", expression, re.DOTALL)
    if combination is None:
        return None  # Other cfg atoms vary by production platform/features.
    operator, arguments = combination.groups()
    parts = []
    depth = 0
    start = 0
    for index, character in enumerate(arguments):
        if character == "(":
            depth += 1
        elif character == ")":
            depth -= 1
        elif character == "," and depth == 0:
            parts.append(arguments[start:index])
            start = index + 1
    if arguments[start:].strip():
        parts.append(arguments[start:])
    values = [production_cfg_value(part) for part in parts]
    if operator == "not":
        return None if len(values) != 1 or values[0] is None else not values[0]
    if operator == "all":
        return False if False in values else (None if None in values else True)
    return True if True in values else (None if None in values else False)


def ffi_test_only_ranges(code: str) -> list[tuple[int, int]]:
    gates = {match.start(): production_cfg_value(match.group(1))
             for match in re.finditer(r"#\s*\[\s*cfg\s*\(([^\]]*)\)\s*\]", code)}
    return [(start, end) for start, end in SOURCE_TOOLS.test_only_ranges(code)
            if gates.get(start) is False]


def ffi_export_inventory(paths: set[Path], root: Path = ROOT) -> list[dict[str, object]]:
    exports = []
    for path in sorted(paths):
        source = path.read_text()
        code = ffi_declaration_code(source)
        ignored = ffi_test_only_ranges(code)
        for match in EXPORTED_C_FN.finditer(code):
            start, end = match.span(1)
            if any(left <= start < right for left, right in ignored):
                continue
            exports.append(
                {
                    "path": str(path.relative_to(root)),
                    "line": code.count("\n", 0, start) + 1,
                    "name": match.group(1),
                }
            )
    return exports


def unsafe_test_hooks(paths: list[Path], root: Path = ROOT) -> list[dict[str, object]]:
    findings = []
    for path in paths:
        code = SOURCE_TOOLS.mask_non_code(path.read_text())
        ignored = ffi_test_only_ranges(code)
        for kind, pattern in (("mutable global", STATIC_MUT), ("raw-pointer hook", RAW_HOOK)):
            for match in pattern.finditer(code):
                if any(start <= match.start() < end for start, end in ignored):
                    continue
                findings.append(
                    {
                        "path": str(path.relative_to(root)),
                        "line": code.count("\n", 0, match.start()) + 1,
                        "kind": kind,
                        "name": match.group(1),
                    }
                )
    return findings


def function_range(source: str, function: str) -> tuple[int, int] | None:
    code = SOURCE_TOOLS.mask_non_code(source)
    for match in TEST_FN.finditer(code):
        if match.group(1) != function:
            continue
        opening = code.rfind("{", match.start(), match.end())
        depth = 0
        for index in range(opening, len(code)):
            if code[index] == "{":
                depth += 1
            elif code[index] == "}":
                depth -= 1
                if depth == 0:
                    return opening, index + 1
    return None


def function_body(source: str, function: str) -> str | None:
    span = function_range(source, function)
    return None if span is None else source[slice(*span)]


def assertion_count(path: Path, function: str) -> int:
    body = function_body(path.read_text(), function)
    return 0 if body is None else len(ASSERTION.findall(SOURCE_TOOLS.mask_non_code(body)))


def run_original(sample: MutationSample) -> subprocess.CompletedProcess:
    return run(
        [
            "cargo",
            "+1.91.0",
            "test",
            "--locked",
            "--test",
            sample.target,
            sample.function,
            "--",
            "--exact",
        ],
        target_dir=TARGET / "default",
    )


def run_mutant(sample: MutationSample) -> subprocess.CompletedProcess:
    source_path = ROOT / sample.source
    source = source_path.read_text()
    span = function_range(source, sample.function)
    if span is None:
        return subprocess.CompletedProcess([], 2, "", "mutation target test is missing")
    start, end = span
    match = re.search(r"\bassert_eq\s*!", SOURCE_TOOLS.mask_non_code(source[start:end]))
    if match is None:
        return subprocess.CompletedProcess([], 2, "", "mutation target has no equality assertion")
    absolute = start + match.start()
    mutated = source[:absolute] + "assert_ne" + source[absolute + len("assert_eq") :]
    with tempfile.TemporaryDirectory(prefix="dqc004-") as directory:
        mutant_root = Path(directory)
        (mutant_root / "tests").mkdir()
        (mutant_root / "tests" / "sample.rs").write_text(mutated)
        manifest = (
            "[package]\nname = \"dqc004-mutant\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n"
            "[dependencies]\nreactive-tui = { path = \""
            + str(ROOT).replace("\\", "\\\\").replace('"', '\\"')
            + "\" }\n"
        )
        (mutant_root / "Cargo.toml").write_text(manifest)
        return run(
            ["cargo", "+1.91.0", "test", "--offline", "--test", "sample", sample.function, "--", "--exact"],
            cwd=mutant_root,
            target_dir=TARGET / "mutants",
        )


def assertion_failure(result: subprocess.CompletedProcess) -> bool:
    output = result.stdout + result.stderr
    return result.returncode != 0 and "assertion `left" in output and "failed" in output


def validate_observation(observation: dict) -> list[str]:
    errors = []
    for name, passed in observation.get("commands", {}).items():
        if passed is not True:
            errors.append(f"command failed: {name}")
    for target in observation.get("missing_targets", []):
        errors.append(f"Cargo target was not compiled: {target}")
    for path in observation.get("disconnected_tests", []):
        errors.append(f"disconnected integration test source: {path}")
    for path in observation.get("unreachable_ffi_modules", []):
        errors.append(f"unreachable FFI module: {path}")
    for hook in observation.get("unsafe_hooks", []):
        errors.append(f"{hook['path']}:{hook['line']} exposes {hook['kind']} {hook['name']}")
    if not observation.get("compiled_targets"):
        errors.append("compiled target inventory is empty")
    if not observation.get("ffi_modules"):
        errors.append("FFI module inventory is empty")
    if not observation.get("ffi_exports"):
        errors.append("FFI export inventory is empty")
    for sample in observation.get("mutations", []):
        label = f"{sample['target']}::{sample['function']}"
        if sample.get("assertions", 0) < 1:
            errors.append(f"sample has no observable assertion: {label}")
        if sample.get("original_passed") is not True:
            errors.append(f"original sample failed: {label}")
        if sample.get("mutant_failed") is not True:
            errors.append(f"assertion mutant survived: {label}")
        elif sample.get("assertion_failed") is not True:
            errors.append(f"mutant failed outside its assertion: {label}")
    return errors


def make_observation() -> tuple[dict, dict[str, subprocess.CompletedProcess]]:
    metadata = cargo_metadata()
    package = root_package(metadata)
    targets = integration_targets(package)
    source_targets = source_targets_below_tests(package)
    feature_graphs = {"default test compile": None, "FFI test compile": "ffi",
                      "graphics test compile": "wgpu-graphics"}
    # The facade emits an unhashed rlib alongside its cdylib. Keep graph and
    # mutant builds separate so a different dependency graph cannot replace it.
    builds = {name: run(cargo_command(features), target_dir=TARGET / (features or "default"))
              for name, features in feature_graphs.items()}
    compiled = {name: compiled_artifacts(result.stdout) for name, result in builds.items()}
    all_names = set().union(*compiled.values())
    required_features = {name: set(target.get("required-features", target.get("required_features", [])))
                         for name, target in targets.items()}
    missing = set(targets) - all_names
    for graph, features in feature_graphs.items():
        enabled = {features} if features else set()
        expected = {name for name, required in required_features.items() if required <= enabled}
        missing.update(expected - compiled[graph])
    roots = [Path(target["src_path"]) for target in source_targets.values()]
    reached_tests = reachable_modules(roots)
    all_tests = rust_source_inventory(ROOT / "tests")
    reached_ffi = reachable_modules([ROOT / "src" / "ffi" / "mod.rs"])
    all_ffi = rust_source_inventory(ROOT / "src" / "ffi")
    unit = run([sys.executable, "-B", "scripts/test-pre-release-test-integrity.py"])
    commands = {"validator tests": unit.returncode == 0,
                **{name: result.returncode == 0 for name, result in builds.items()}}
    mutations = []
    results = {"validator tests": unit, **builds}
    for sample in SAMPLES:
        original = run_original(sample)
        mutant = run_mutant(sample)
        results[f"original {sample.function}"] = original
        results[f"mutant {sample.function}"] = mutant
        mutations.append(
            {
                **asdict(sample),
                "assertions": assertion_count(ROOT / sample.source, sample.function),
                "original_passed": original.returncode == 0,
                "mutant_failed": mutant.returncode != 0,
                "assertion_failed": assertion_failure(mutant),
            }
        )
    observation = {
        "commands": commands,
        "compiled_targets": sorted(all_names),
        "missing_targets": sorted(missing),
        "test_sources": sorted(str(path.relative_to(ROOT)) for path in all_tests),
        "disconnected_tests": sorted(str(path.relative_to(ROOT)) for path in all_tests - reached_tests),
        "ffi_modules": sorted(str(path.relative_to(ROOT)) for path in reached_ffi),
        "ffi_exports": ffi_export_inventory(reached_ffi),
        "unreachable_ffi_modules": sorted(str(path.relative_to(ROOT)) for path in all_ffi - reached_ffi),
        "unsafe_hooks": unsafe_test_hooks(SOURCE_TOOLS.library_source_files()),
        "mutations": mutations,
    }
    return observation, results


def rendered_diagnostics(output: str):
    for line in output.splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(item, dict) or item.get("reason") != "compiler-message":
            continue
        message = item.get("message")
        if isinstance(message, dict) and isinstance(message.get("rendered"), str):
            yield message["rendered"]


def report(observation: dict, errors: list[str], results: dict[str, subprocess.CompletedProcess]) -> None:
    if errors:
        print("DQC-004 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        for name, result in results.items():
            if result.returncode != 0 and not name.startswith("mutant "):
                for diagnostic in rendered_diagnostics(result.stdout):
                    print(diagnostic, file=sys.stderr)
                print(f"--- {name} stderr ---", file=sys.stderr)
                print(result.stderr[-4000:], file=sys.stderr)
        return
    print("DQC-004 compiled test targets:")
    for target in observation["compiled_targets"]:
        print(f"- {target}")
    print("DQC-004 reachable FFI modules and exports:")
    for path in observation["ffi_modules"]:
        print(f"- {path}")
    for export in observation["ffi_exports"]:
        print(f"- {export['path']}:{export['line']} {export['name']}")
    print("DQC-004 audited hooks: none unsafe in shipped code")
    print("DQC-004 mutation results:")
    for sample in observation["mutations"]:
        print(f"- {sample['target']}::{sample['function']}: original passed; assertion mutant failed")
    print("DQC-004 passed: test targets, FFI reachability, hooks, and observable assertions are sound.")


def main() -> int:
    if sys.argv[1:] != ["DQC-004"]:
        print("usage: check-pre-release-test-integrity.py DQC-004", file=sys.stderr)
        return 2
    try:
        observation, results = make_observation()
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f"DQC-004 failed: {error}", file=sys.stderr)
        return 1
    errors = validate_observation(observation)
    report(observation, errors, results)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
