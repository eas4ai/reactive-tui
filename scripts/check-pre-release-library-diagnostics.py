#!/usr/bin/env python3
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


REQUIRED_PATHS = ("parser", "terminal", "reconciliation", "focus", "window")
ROOT = Path(__file__).resolve().parents[1]
RAW_OUTPUT = re.compile(r"\b(e?print(?:ln)?)\s*!")
ANSI_SEQUENCE = re.compile(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))")


def contains_internal_output(stream: str, output: str) -> bool:
    if not output:
        return False
    if stream == "stderr":
        return True
    visible = ANSI_SEQUENCE.sub("", output)
    visible = "".join(character for character in visible if character >= " " or character.isspace())
    return bool(visible.strip())


def validate_observation(observation: dict) -> list[str]:
    errors = []
    if observation.get("source_audit") is not True:
        errors.append("source audit failed")
    if not observation.get("audited_files"):
        errors.append("source audit reported no library files")
    for finding in observation.get("raw_macros", []):
        errors.append(
            f"{finding['path']}:{finding['line']} reaches {finding['macro']}!"
        )
    if observation.get("capture") is not True:
        errors.append("captured-output probe failed")
    for stream in ("stdout", "stderr"):
        if contains_internal_output(stream, observation.get(stream, "")):
            errors.append(f"captured internal {stream} output")
    paths = observation.get("exercised_paths", {})
    for path in REQUIRED_PATHS:
        if paths.get(path) is not True:
            errors.append(f"diagnostic probe skipped {path}")
    return errors


def capture_command() -> list[str]:
    return [
        "cargo",
        "+1.91.0",
        "test",
        "--locked",
        "--test",
        "dqc_003_captured_diagnostics",
        "--features",
        "debug,debug_patches",
        "--no-run",
        "--message-format=json",
    ]


def library_source_files() -> list[Path]:
    excluded = {"benches", "examples", "tests"}
    return sorted(
        path
        for path in (ROOT / "src").rglob("*.rs")
        if path.name != "tests.rs" and not excluded.intersection(path.relative_to(ROOT).parts)
    )


def mask_span(masked: list[str], source: str, start: int, end: int) -> None:
    for index in range(start, end):
        if source[index] != "\n":
            masked[index] = " "


def consume_line_comment(source: str, masked: list[str], start: int) -> int:
    end = source.find("\n", start)
    end = len(source) if end == -1 else end
    mask_span(masked, source, start, end)
    return end


def consume_block_comment(source: str, masked: list[str], start: int) -> int:
    index = start + 2
    depth = 1
    while index < len(source) and depth:
        if source.startswith("/*", index):
            depth += 1
            index += 2
        elif source.startswith("*/", index):
            depth -= 1
            index += 2
        else:
            index += 1
    mask_span(masked, source, start, index)
    return index


def consume_quoted_string(source: str, masked: list[str], start: int, width: int) -> int:
    index = start + width
    while index < len(source):
        if source[index] == "\\" and index + 1 < len(source):
            index += 2
        elif source[index] == '"':
            index += 1
            break
        else:
            index += 1
    mask_span(masked, source, start, index)
    return index


def consume_raw_string(
    source: str, masked: list[str], start: int, opener: re.Match[str]
) -> int:
    terminator = '"' + ("#" * len(opener.group(1)))
    content_start = start + len(opener.group(0))
    closing = source.find(terminator, content_start)
    end = len(source) if closing == -1 else closing + len(terminator)
    mask_span(masked, source, start, end)
    return end


def mask_non_code(source: str) -> str:
    masked = list(source)
    index = 0
    while index < len(source):
        raw = re.match(r'(?:br|r)(#*)"', source[index:])
        if raw:
            index = consume_raw_string(source, masked, index, raw)
        elif source.startswith("//", index):
            index = consume_line_comment(source, masked, index)
        elif source.startswith("/*", index):
            index = consume_block_comment(source, masked, index)
        elif source.startswith('b"', index):
            index = consume_quoted_string(source, masked, index, 2)
        elif source[index] == '"':
            index = consume_quoted_string(source, masked, index, 1)
        else:
            index += 1
    return "".join(masked)


def test_only_ranges(code: str) -> list[tuple[int, int]]:
    attribute = re.compile(r"#\s*\[\s*cfg\s*\([^\]]*\btest\b[^\]]*\)\s*\]\s*[^\{;]*\{")
    ranges = []
    for match in attribute.finditer(code):
        opening = code.rfind("{", match.start(), match.end())
        depth = 0
        for index in range(opening, len(code)):
            if code[index] == "{":
                depth += 1
            elif code[index] == "}":
                depth -= 1
                if depth == 0:
                    ranges.append((match.start(), index + 1))
                    break
    return ranges


def raw_output_macros(paths: list[Path]) -> list[dict[str, object]]:
    findings = []
    for path in paths:
        code = mask_non_code(path.read_text())
        ignored = test_only_ranges(code)
        for match in RAW_OUTPUT.finditer(code):
            if any(start <= match.start() < end for start, end in ignored):
                continue
            findings.append(
                {
                    "path": str(path),
                    "line": code.count("\n", 0, match.start()) + 1,
                    "macro": match.group(1),
                }
            )
    return findings


def probe_executable(output: str) -> Path | None:
    for line in output.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("target", {}).get("name") == "dqc_003_captured_diagnostics"
            and message.get("executable")
        ):
            return Path(message["executable"])
    return None


def probe_report(path: Path) -> dict[str, bool]:
    completed = set(path.read_text().splitlines()) if path.is_file() else set()
    return {name: name in completed for name in REQUIRED_PATHS}


def run_validator_tests() -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, "-B", "scripts/test-pre-release-library-diagnostics.py"],
        check=False,
    )


def build_probe() -> tuple[subprocess.CompletedProcess, Path | None]:
    build = subprocess.run(
        capture_command(), cwd=ROOT, text=True, capture_output=True, timeout=900, check=False
    )
    executable = probe_executable(build.stdout) if build.returncode == 0 else None
    return build, executable


def run_probe(
    executable: Path | None, report_path: Path
) -> subprocess.CompletedProcess | None:
    if executable is None:
        return None
    runtime_env = {**os.environ, "DQC003_REPORT": str(report_path), "TERM": "dumb"}
    for name in ("COLORTERM", "REACTIVE_TUI_FORCE_ENABLE"):
        runtime_env.pop(name, None)
    return subprocess.run(
        [executable],
        cwd=ROOT,
        env=runtime_env,
        text=True,
        capture_output=True,
        timeout=60,
        check=False,
    )


def make_observation(
    sources: list[Path],
    findings: list[dict[str, object]],
    captured: subprocess.CompletedProcess | None,
    report_path: Path,
) -> dict:
    return {
        "source_audit": True,
        "audited_files": [str(path.relative_to(ROOT)) for path in sources],
        "raw_macros": [
            {**finding, "path": str(Path(finding["path"]).relative_to(ROOT))}
            for finding in findings
        ],
        "capture": captured is not None and captured.returncode == 0,
        "stdout": "" if captured is None else captured.stdout,
        "stderr": "" if captured is None else captured.stderr,
        "exercised_paths": probe_report(report_path),
    }


def collect_errors(
    unit: subprocess.CompletedProcess,
    build: subprocess.CompletedProcess,
    executable: Path | None,
    observation: dict,
) -> list[str]:
    errors = []
    if unit.returncode != 0:
        errors.append("validator tests failed")
    if build.returncode != 0:
        errors.append("captured-output probe did not compile")
    elif executable is None:
        errors.append("cargo did not identify the captured-output probe executable")
    errors.extend(validate_observation(observation))
    return errors


def report_failure(
    errors: list[str],
    build: subprocess.CompletedProcess,
    captured: subprocess.CompletedProcess | None,
) -> None:
    print("DQC-003 failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    if build.returncode != 0 and build.stderr:
        print(build.stderr, end="", file=sys.stderr)
    if captured is not None and captured.stdout:
        print(f"captured stdout: {captured.stdout!r}", file=sys.stderr)
    if captured is not None and captured.stderr:
        print(f"captured stderr: {captured.stderr!r}", file=sys.stderr)


def report_success(observation: dict) -> None:
    print("DQC-003 audited library sources:")
    for path in observation["audited_files"]:
        print(f"- {path}")
    print("DQC-003 exercised diagnostic paths: " + ", ".join(REQUIRED_PATHS))
    print("DQC-003 passed: library diagnostics are routed without process output.")


def check_dqc_003() -> int:
    unit = run_validator_tests()
    sources = library_source_files()
    findings = raw_output_macros(sources)
    build, executable = build_probe()
    with tempfile.TemporaryDirectory() as directory:
        report_path = Path(directory) / "paths"
        captured = run_probe(executable, report_path)
        observation = make_observation(sources, findings, captured, report_path)
    errors = collect_errors(unit, build, executable, observation)
    if errors:
        report_failure(errors, build, captured)
        return 1
    report_success(observation)
    return 0


def main() -> int:
    if sys.argv[1:] != ["DQC-003"]:
        print("usage: check-pre-release-library-diagnostics.py DQC-003", file=sys.stderr)
        return 2
    return check_dqc_003()


if __name__ == "__main__":
    raise SystemExit(main())
