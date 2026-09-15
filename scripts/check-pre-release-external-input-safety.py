#!/usr/bin/env python3
"""Run focused checks for pre-release external-input requirements."""

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
JOBS = "8"


def run(command: list[str], required: list[str]) -> None:
    environment = os.environ.copy()
    environment["CARGO_BUILD_JOBS"] = JOBS
    environment["CARGO_INCREMENTAL"] = "0"
    result = subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=600,
    )
    print(result.stdout, end="")
    if result.returncode:
        raise SystemExit(result.returncode)
    missing = [name for name in required if f"test {name} ... ok" not in result.stdout]
    if missing:
        raise SystemExit(f"required XIS-001 tests did not run: {', '.join(missing)}")


def check_xis_001() -> None:
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            "widgets::dialog::http::tests::",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        [
            "widgets::dialog::http::tests::xis_001_capture_validator_rejects_unsafe_observations",
            "widgets::dialog::http::tests::xis_001_request_process_receives_only_documented_environment",
            "widgets::dialog::http::tests::request_preserves_json_escaping_and_custom_headers",
            "widgets::dialog::http::tests::cancellation_joins_worker_and_closes_the_actual_connection",
            "widgets::dialog::http::tests::a_silent_endpoint_hits_the_owned_deadline",
            "widgets::dialog::http::tests::chunked_response_cannot_bypass_the_size_limit",
            "widgets::dialog::http::tests::unsafe_configuration_and_oversized_requests_fail_before_spawn",
        ],
    )
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--test",
            "api_widget_behavior",
            "dialog_http_acceptance::xis_001_disabled_dialog_configuration_never_spawns_curl",
            "--jobs",
            JOBS,
            "--",
            "--exact",
            "--test-threads=1",
        ],
        ["dialog_http_acceptance::xis_001_disabled_dialog_configuration_never_spawns_curl"],
    )


def main() -> int:
    if sys.argv[1:] != ["XIS-001"]:
        print(f"usage: {Path(sys.argv[0]).name} XIS-001", file=sys.stderr)
        return 2
    check_xis_001()
    print("XIS-001 dialog network safety passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
