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
        raise SystemExit(f"required tests did not run: {', '.join(missing)}")


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
        ["python3", "-B", "scripts/check-dialog-http.py"],
        ["widgets::dialog::http::tests::tls_certificate_probe"],
    )
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--test",
            "api_widget_behavior",
            "dialog_http_acceptance::",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        [
            "dialog_http_acceptance::xis_001_disabled_dialog_configuration_never_spawns_curl",
            "dialog_http_acceptance::dialog_engine_http_validation_delivers_an_awaitable_result",
            "dialog_http_acceptance::dialog_engine_http_close_cancels_a_live_request_and_does_not_submit",
            "dialog_http_acceptance::input_dialog_remote_validation_posts_json_and_completes_pending_submission",
            "dialog_http_acceptance::autocomplete_http_sends_query_headers_and_renders_suggestion_objects",
            "dialog_http_acceptance::autocomplete_http_debounces_edits_into_one_current_query",
            "dialog_http_acceptance::autocomplete_removal_cancels_a_live_http_request",
        ],
    )


def check_xis_002() -> None:
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            "widgets::display::image::external_renderer::tests::xis_002_",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        [
            "widgets::display::image::external_renderer::tests::xis_002_argument_validator_rejects_unsafe_observations",
            "widgets::display::image::external_renderer::tests::xis_002_external_paths_are_positional_data",
        ],
    )
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            "widgets::display::image::decoded::tests::xis_002_decode_rejects_combined_storage_over_budget",
            "--jobs",
            JOBS,
            "--",
            "--exact",
            "--test-threads=1",
        ],
        [
            "widgets::display::image::decoded::tests::xis_002_decode_rejects_combined_storage_over_budget"
        ],
    )
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            "widgets::display::image::decoded::tests::api_image_decode_",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        [
            "widgets::display::image::decoded::tests::api_image_decode_sources_preserve_pixels_and_encoded_dimensions",
            "widgets::display::image::decoded::tests::api_image_decode_all_advertised_encoded_formats",
            "widgets::display::image::decoded::tests::api_image_decode_rejects_malformed_and_oversized_sources",
            "widgets::display::image::decoded::tests::api_image_decode_opaque_renderers_composite_alpha",
        ],
    )


def check_xis_003() -> None:
    run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            "widgets::display::file_explorer::live::worker::tests::xis_003_",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        [
            "widgets::display::file_explorer::live::worker::tests::xis_003_validator_rejects_replaced_entry_outcomes",
            "widgets::display::file_explorer::live::worker::tests::xis_003_copy_rejects_an_entry_replaced_after_inspection",
            "widgets::display::file_explorer::live::worker::tests::xis_003_remove_rejects_an_entry_replaced_after_inspection",
        ],
    )


def main() -> int:
    if sys.argv[1:] not in (["XIS-001"], ["XIS-002"], ["XIS-003"]):
        print(
            f"usage: {Path(sys.argv[0]).name} XIS-001|XIS-002|XIS-003",
            file=sys.stderr,
        )
        return 2
    requirement = sys.argv[1]
    if requirement == "XIS-001":
        check_xis_001()
        print("XIS-001 dialog network safety passed")
    elif requirement == "XIS-002":
        check_xis_002()
        print("XIS-002 image input safety passed")
    else:
        check_xis_003()
        print("XIS-003 file-explorer identity safety passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
