#!/usr/bin/env python3
"""Run residual API behavior checks and reject missing audit coverage."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import runpy

ROOT = Path(__file__).resolve().parents[1]
CONCERNS = (
    "Hover, drag, drag-and-drop, mouse position, clicks, long press, swipe and wheel hooks",
    "Public reference hooks", "ui::Updater", "Theme propagation", "Performance context",
    "Markdown/Lumis integration", "Large Markdown/syntax input", "Editor undo/selection",
    "Unix input worker", "SIGWINCH ownership", "Legacy input parsing",
    "DebugBackend boundaries", "Raw-mode ownership", "Public RenderTree",
    "Nested legacy events", "Legacy backend test reachability", "Transition integration metadata",
    "CSS property diagnostics", "Full claimed native platform surface",
)
REF_TESTS = (
    "shared_reference_retains_value_and_identity_across_renders",
    "local_reference_retains_non_send_value_across_renders",
    "callback_reference_retains_current_value_across_renders",
    "multi_reference_retains_members_across_renders",
    "forwarded_reference_follows_the_current_parent",
    "callback_reference_allows_read_and_write_reentry",
    "shared_reference_slots_and_owners_are_independent",
    "shared_hooks_retain_state_without_returned_handles",
    "retained_callback_handles_use_the_latest_render_capture",
    "reference_hooks_preserve_existing_type_bounds",
)
LOCAL_TESTS = ('local_slots_retain_without_handles_and_isolate_owners', 'cleanup_and_last_clone_drop_release_local_shares', 'foreign_owner_drop_defers_until_creator_sweep_or_exit', 'scopes_reject_missing_wrong_thread_and_expired_owners', 'local_slots_validate_kind_type_and_generated_count', 'unwinding_releases_scope_but_escaped_handles_keep_ownership', 'cleanup_destructors_can_reenter_another_local_owner', 'smoke_thread_safe_hooks_and_generated_state_remain_send_sync', 'app_supplies_scope_for_generated_components_and_closes_on_error_and_unwind', 'app_reuses_manual_scope_and_leaves_other_owners_alive', 'generated_local_component_can_move_before_its_first_render', 'keyed_component_removal_releases_local_values_before_scope_exit', 'inner_scope_cleanup_reclaims_outer_owner_without_closing_inner')
PERFORMANCE_TESTS = (
    'sequential_apps_keep_snapshots_and_closed_setters_isolated',
    'concurrent_apps_route_worker_mode_requests_only_to_their_owner',
    'mode_bursts_keep_latest_request_and_metrics_do_not_redraw_idle_apps',
    'descendant_provider_overrides_app_performance_context',
    'errors_unwinds_and_unrun_drop_close_escaped_mode_setters',
    'performance_hooks_keep_slots_when_context_appears_and_owner_closes',
    'nested_app_run_restores_outer_performance_context',
    'standalone_globals_are_not_read_or_written_by_apps',
    'repeated_mode_effects_and_cleanup_requests_return_to_idle',
    'actual_fps_tracks_presentation_cadence_instead_of_render_throughput',
)
MAPPING_TESTS = tuple("backend::tests::" + name for name in (
    "map_paste_event", "map_focus_gained_lost", "map_resize_event",
    "map_key_event_basic", "map_mouse_event_basic",
))
INPUT_CASES = (
    "raw-receiver-idle", "parsed-receiver-idle", "reused-descriptor", "raw-receiver-active",
    "raw-receiver-full", "raw-session-full", "raw-session-idle", "parsed-receiver-full",
    "parsed-session-full", "parsed-session-idle", "clone-owner", "independent-streams",
    "owned-iterator", "concurrent-drop",
)
INPUT_TESTS = tuple("platform::input_receiver::tests::" + name for name in (
    "dropping_owned_iterator_joins_idle_reader", "receive_and_iteration_preserve_order_and_disconnect",
    "buffered_items_remain_after_session_shutdown", "draining_a_full_queue_wakes_the_producer",
    "cancellation_releases_a_full_queue_producer",
))
API019_LIB_TESTS = (
    "backend::suprtui::raw_mode_tests::api019_independent_library_owners_restore_only_after_the_last_release",
    "backend::tests::api019_debug_backend_accepts_empty_and_bounded_dimensions",
    "backend::tests::api019_debug_backend_rejects_oversized_resize_without_truncation_or_allocation",
    "component::runtime::tests::api019_component_removal_unregisters_owned_mouse_hooks",
    "markdown::tests::api019_fenced_rust_code_uses_lumis_and_disabled_mode_keeps_code_style",
    "markdown::tests::api019_markdown_and_syntax_checked_entry_points_reject_oversized_sources",
    "platform::parser::tests::api019_incomplete_sequence_buffer_is_bounded_and_reports_overflow",
    "platform::parser::tests::api019_sgr_mouse_preserves_buttons_modifiers_drag_and_wheel",
    "platform::parser::tests::api019_split_utf8_escape_and_mouse_sequences_are_retained",
    "platform::unix::tests::api019_sigwinch_dispatches_outside_signal_context_and_preserves_prior_handler",
    "render::tree::tests::api019_public_render_tree_keeps_fragment_custom_deep_and_wide_nodes",
    "screen::residual_metadata_tests::api019_compatibility_metadata_does_not_change_terminal_transition_progress",
    "theme::tests::api019_child_theme_inherits_and_overrides_without_cross_instance_state",
)
GESTURE_TESTS = (
    "api019_app_routes_mouse_hooks_options_and_local_coordinates",
    "tests::api019_drag_drop_options_and_owner_cleanup_control_routed_state",
    "tests::api019_component_click_and_gesture_history_are_isolated",
)
EDITOR_TESTS = (
    "cursor_uses_scalar_offsets_and_display_columns",
    "plain::selection_replaces_whole_graphemes_across_lines",
    "syntax::selection_replaces_whole_graphemes_across_lines",
    "word_movement_retains_unicode_grapheme_boundaries",
)
NESTED_EVENT_TESTS = (
    "capture_target_bubble_order_and_handled_result_are_preserved",
    "child_activation_runs_each_registration_once_without_activating_parent",
    "removed_callback_releases_its_capture_and_cannot_run_again",
)
CSS_TESTS = (
    "app::motion::tests::incompatible_css_units_keep_endpoints_and_unknown_styles_report_errors",
    "checked_conversion_requires_a_named_property_and_rejects_value_loss",
)
GROUPS = ("refs", "mapping-tests", "unix-input", "performance", "updaters", "inventory")


def require_registered(text, names):
    registered = {line.removesuffix(": test") for line in text.splitlines() if line.endswith(": test")}
    missing = set(names) - registered
    if missing:
        raise AssertionError("Rust test harness did not register: " + ", ".join(sorted(missing)))


def require_executed(text, names):
    executed = set(re.findall(r"^test ([\w:]+) \.\.\. ok$", text, re.M))
    missing = set(names) - executed
    if missing or re.search(r"test result: FAILED|[1-9][0-9]* failed;", text):
        raise AssertionError("Required passing Rust tests missing or failed: " + ", ".join(sorted(missing)))


def require_coverage(checks):
    missing = set(CONCERNS) - set(checks)
    if missing:
        raise AssertionError("Focused behavior checks still required for: " + "; ".join(sorted(missing)))


def require_input_cases(text):
    records = [json.loads(line.removeprefix("INPUT_LIFECYCLE "))
               for line in text.splitlines() if line.startswith("INPUT_LIFECYCLE ")]
    if len(records) != 1:
        raise AssertionError("Expected one complete input lifecycle result")
    rows = records[0]
    if sorted(row["case"] for row in rows) != sorted(INPUT_CASES):
        raise AssertionError("Missing, extra or duplicate input lifecycle cases")
    if any(row["exit"] != 0 or row["timeout"] or row["reaped"] is not True for row in rows):
        raise AssertionError("Input lifecycle cases failed, timed out or were not reaped")


def require_marker(text, marker):
    if marker not in text.splitlines():
        raise AssertionError("Consumer completion marker missing: " + marker)


class Check:
    def __init__(self):
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        self.output = ROOT / "target/evidence/api-residual" / stamp
        self.output.mkdir(parents=True)
        self.execute = runpy.run_path(str(ROOT / "scripts/check-widget-platforms.py"))["execute"]
        self.steps = []

    def run(self, name, command, timeout=900, verify=None):
        log = self.output / (name + ".out")
        item = {"name": name, "command": command, "log": str(log.relative_to(ROOT))}
        try:
            text = self.execute(command, log, timeout)
            if verify:
                verify(text)
        except Exception as error:
            item.update(result="fail", error=str(error))
            print("FAIL", name, error, flush=True)
        else:
            item["result"] = "pass"
            print("PASS", name, flush=True)
        if log.exists():
            item["sha256"] = hashlib.sha256(log.read_bytes()).hexdigest()
        self.steps.append(item)

    def refs(self):
        self.run("refs-discovery", ["cargo", "test", "--locked", "--test", "api_residual_refs", "--", "--list"],
                 verify=lambda text: require_registered(text, REF_TESTS))
        self.run("refs-behavior", ["cargo", "test", "--locked", "--test", "api_residual_refs", "--", "--test-threads=1"],
                 verify=lambda text: require_executed(text, REF_TESTS))

        self.run("local-discovery", ["cargo", "test", "--locked", "--test", "api_local_hooks", "--", "--list"],
                 verify=lambda text: require_registered(text, LOCAL_TESTS))
        self.run("local-behavior", ["cargo", "test", "--locked", "--test", "api_local_hooks", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, LOCAL_TESTS))

        self.run("local-consumer", ["python3", "-B", "verification/api-residual/run-local-owner.py"])

    def mapping_tests(self):
        self.run("mapping-discovery", ["cargo", "test", "--locked", "--lib", "backend::tests::map_", "--", "--list"],
                 verify=lambda text: require_registered(text, MAPPING_TESTS))
        self.run("mapping-behavior", ["cargo", "test", "--locked", "--lib", "backend::tests::map_", "--", "--test-threads=1"],
                 verify=lambda text: require_executed(text, MAPPING_TESTS))

    def unix_input(self):
        self.run("input-unit-discovery", ["cargo", "test", "--locked", "--lib", "platform::input_receiver::tests", "--", "--list"],
                 verify=lambda text: require_registered(text, INPUT_TESTS))
        self.run("input-unit-behavior", ["cargo", "test", "--locked", "--lib", "platform::input_receiver::tests"],
                 verify=lambda text: require_executed(text, INPUT_TESTS))
        self.run("input-lifecycle", ["python3", "-B", "verification/api-residual/run-input-lifecycle.py"],
                 verify=require_input_cases)
        self.run("input-controller-cancellation", ["python3", "-B", "verification/api-residual/check-controller-cleanup.py"], 30)

    def performance(self):
        self.run("performance-discovery", ["cargo", "test", "--locked", "--test", "api_performance_context", "--", "--list"],
                 verify=lambda text: require_registered(text, PERFORMANCE_TESTS))
        self.run("performance-behavior", ["cargo", "test", "--locked", "--test", "api_performance_context", "--", "--test-threads=1"],
                 verify=lambda text: require_executed(text, PERFORMANCE_TESTS))
        for name in ("hooks::fps::tests::optional_provider_changes_preserve_slots_and_recompute_quality",
                     "reactive::wake::tests::completed_metrics_notify_other_apps_and_preserve_owner_subscription"):
            self.run(name.rsplit("::", 1)[1], ["cargo", "test", "--locked", "--lib", name, "--", "--exact"],
                     verify=lambda text, name=name: require_executed(text, [name]))
        self.run("performance-consumer-library", ["cargo", "build", "--locked", "--lib"])
        binary = str(ROOT / "target/api019-performance-owner")
        self.run("performance-consumer-compile", ["rustc", "--edition=2021", "verification/api-residual/performance-owner.rs",
                 "--extern", "reactive_tui=" + str(ROOT / "target/debug/libreactive_tui.rlib"),
                   "-L", "dependency=" + str(ROOT / "target/debug/deps"), "-C", "link-arg=-Wl,--threads=8", "-o", binary])
        self.run("performance-consumer-behavior", [binary], 10,
                 verify=lambda text: require_marker(text, "PERFORMANCE_OWNER_OK"))

    def updaters(self):
        names = (
            "worker_request_wakes_idle_app_and_repaints_state",
            "requests_update_only_the_registered_app_before_render_and_close_after_exit",
            "errors_unwinds_and_unrun_drop_close_requests_and_release_updaters",
        )
        self.run("updater-discovery", ["cargo", "test", "--locked", "--test", "api_updater", "--", "--list"],
                 verify=lambda text: require_registered(text, names))
        self.run("updater-behavior", ["cargo", "test", "--locked", "--test", "api_updater"],
                 verify=lambda text: require_executed(text, names))
        unit_names = ["ui::updater::tests::" + name for name in (
            "requests_coalesce_order_and_reentry_waits_for_next_batch",
            "cancellation_and_owner_close_invalidate_escaped_handles",
        )]
        self.run("updater-ordering", ["cargo", "test", "--locked", "--lib", "ui::updater::tests"],
                 verify=lambda text: require_executed(text, unit_names))

    def inventory(self):
        self.run("api019-lib-discovery", ["cargo", "test", "--locked", "--lib", "api019_", "--", "--list"],
                 verify=lambda text: require_registered(text, API019_LIB_TESTS))
        self.run("api019-lib-behavior", ["cargo", "test", "--locked", "--lib", "api019_", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, API019_LIB_TESTS))

        self.run("gesture-app-discovery", ["cargo", "test", "--locked", "--test", "api_mouse_hook_routing", "--", "--list"],
                 verify=lambda text: require_registered(text, GESTURE_TESTS[:1]))
        self.run("gesture-app-behavior", ["cargo", "test", "--locked", "--test", "api_mouse_hook_routing", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, GESTURE_TESTS[:1]))
        self.run("gesture-options-discovery", ["cargo", "test", "--locked", "--test", "mouse_hooks_test", "api019_", "--", "--list"],
                 verify=lambda text: require_registered(text, GESTURE_TESTS[1:]))
        self.run("gesture-options-behavior", ["cargo", "test", "--locked", "--test", "mouse_hooks_test", "api019_", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, GESTURE_TESTS[1:]))

        self.run("editor-discovery", ["cargo", "test", "--locked", "--test", "api_editor_unicode", "--", "--list"],
                 verify=lambda text: require_registered(text, EDITOR_TESTS))
        self.run("editor-behavior", ["cargo", "test", "--locked", "--test", "api_editor_unicode", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, EDITOR_TESTS))

        self.run("nested-events-discovery", ["cargo", "test", "--locked", "--test", "api_event_routing", "--", "--list"],
                 verify=lambda text: require_registered(text, NESTED_EVENT_TESTS))
        self.run("nested-events-behavior", ["cargo", "test", "--locked", "--test", "api_event_routing", "--", "--test-threads=8"],
                 verify=lambda text: require_executed(text, NESTED_EVENT_TESTS))

        self.run("css-unit-discovery", ["cargo", "test", "--locked", "--lib", CSS_TESTS[0], "--", "--list"],
                 verify=lambda text: require_registered(text, CSS_TESTS[:1]))
        self.run("css-unit-behavior", ["cargo", "test", "--locked", "--lib", CSS_TESTS[0], "--", "--exact"],
                 verify=lambda text: require_executed(text, CSS_TESTS[:1]))
        self.run("css-public-discovery", ["cargo", "test", "--locked", "--test", "api_animation_screens", CSS_TESTS[1], "--", "--list"],
                 verify=lambda text: require_registered(text, CSS_TESTS[1:]))
        self.run("css-public-behavior", ["cargo", "test", "--locked", "--test", "api_animation_screens", CSS_TESTS[1], "--", "--exact"],
                 verify=lambda text: require_executed(text, CSS_TESTS[1:]))

        self.run("native-platform-records", ["python3", "-B", "scripts/check-clipboard-platforms.py", "--verify"])

        passed_groups = {
            step["name"] for step in self.steps
            if step.get("result") == "pass" and step["name"] in GROUPS
        }
        checks = {
            "Hover, drag, drag-and-drop, mouse position, clicks, long press, swipe and wheel hooks",
            "Theme propagation", "Markdown/Lumis integration", "Large Markdown/syntax input",
            "Editor undo/selection", "SIGWINCH ownership", "Legacy input parsing",
            "DebugBackend boundaries", "Raw-mode ownership", "Public RenderTree",
            "Nested legacy events", "Transition integration metadata", "CSS property diagnostics",
            "Full claimed native platform surface",
        }
        if "refs" in passed_groups:
            checks.add("Public reference hooks")
        if "mapping-tests" in passed_groups:
            checks.add("Legacy backend test reachability")
        if "unix-input" in passed_groups:
            checks.add("Unix input worker")
        if "performance" in passed_groups:
            checks.add("Performance context")
        if "updaters" in passed_groups:
            checks.add("ui::Updater")
        require_coverage(checks)

    def inspect(self, name):
        before = len(self.steps)
        try:
            getattr(self, name.replace("-", "_"))()
            if any(step["result"] != "pass" for step in self.steps[before:]):
                raise AssertionError("A required nested check failed; see retained outputs")
        except Exception as error:
            self.steps.append({"name": name, "result": "fail", "error": str(error)})
            print("FAIL", name, error, flush=True)
        else:
            self.steps.append({"name": name, "result": "pass"})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only", choices=GROUPS)
    args = parser.parse_args()
    os.chdir(ROOT)
    os.environ.update(CARGO_INCREMENTAL="0", CARGO_BUILD_JOBS="8", RUST_TEST_THREADS="8",
                      RAYON_NUM_THREADS="8", LP_NUM_THREADS="8", PYTHON_CPU_COUNT="8",
                      GOMAXPROCS="8", GOFLAGS="-p=8", CARGO_TARGET_DIR=str(ROOT / "target"))
    check = Check()
    check.run("checker-controls", ["python3", "-B", "verification/api-residual/checker-controls.py"], 30)
    for group in ((args.only,) if args.only else GROUPS):
        check.inspect(group)
    (check.output / "results.json").write_text(json.dumps(check.steps, indent=2) + "\n")
    print("Residual outputs:", check.output.relative_to(ROOT), flush=True)
    failures = [step["name"] for step in check.steps if step["result"] != "pass"]
    if failures:
        raise SystemExit("API-019 failed: " + ", ".join(failures))
    if args.only:
        print(f"PASS development section: {args.only}; full API-019 acceptance was not run")
    else:
        print("PASS API-019: all residual contracts and focused behavior checks")


if __name__ == "__main__":
    main()
