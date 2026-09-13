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
    "Markdown/Syntect integration", "Large Markdown/syntax input", "Editor undo/selection",
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
LOCAL_TESTS = ('local_slots_retain_without_handles_and_isolate_owners', 'cleanup_and_last_clone_drop_release_local_shares', 'foreign_owner_drop_defers_until_creator_sweep_or_exit', 'scopes_reject_missing_wrong_thread_and_expired_owners', 'local_slots_validate_kind_type_and_generated_count', 'unwinding_releases_scope_but_escaped_handles_keep_ownership', 'cleanup_destructors_can_reenter_another_local_owner', 'thread_safe_hooks_and_generated_state_remain_send_sync', 'app_supplies_scope_for_generated_components_and_closes_on_error_and_unwind', 'app_reuses_manual_scope_and_leaves_other_owners_alive', 'generated_local_component_can_move_before_its_first_render', 'keyed_component_removal_releases_local_values_before_scope_exit', 'inner_scope_cleanup_reclaims_outer_owner_without_closing_inner')
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
GROUPS = ("refs", "mapping-tests", "unix-input", "inventory")


def inventory_rows(text):
    rows = {}
    for line in text.splitlines():
        if not line.startswith("| "):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if cells[0] in ("Concern", "---"):
            continue
        if len(cells) != 4 or not all(cells) or cells[0] in rows:
            raise AssertionError("Residual inventory needs unique, complete contract/falsifier rows")
        rows[cells[0]] = cells[1:]
    required = set(CONCERNS) | {"Image capture timeout ownership"}
    if set(rows) != required:
        raise AssertionError(f"Residual inventory drift: missing={required - set(rows)}, extra={set(rows) - required}")
    return rows


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


class Check:
    def __init__(self):
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        self.output = ROOT / ".cairn/reviews/api-residual" / stamp
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
        self.run("local-behavior", ["cargo", "test", "--locked", "--test", "api_local_hooks", "--", "--test-threads=12"],
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

    def inventory(self):
        inventory_rows((ROOT / "docs/residual-api-inventory.md").read_text())
        # Presence alone cannot certify a concern. Add its complete behavior check
        # here only after building its positive and safe violating cases.
        # The Unix receiver checks are integrated, but the wider legacy event-loop
        # and native-platform concerns still need completion before coverage closes.
        require_coverage({})

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
    os.environ.update(CARGO_INCREMENTAL="0", CARGO_BUILD_JOBS="12", RUST_TEST_THREADS="12",
                      RAYON_NUM_THREADS="12", LP_NUM_THREADS="12", PYTHON_CPU_COUNT="12",
                      GOMAXPROCS="12", GOFLAGS="-p=12", CARGO_TARGET_DIR=str(ROOT / "target"))
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
