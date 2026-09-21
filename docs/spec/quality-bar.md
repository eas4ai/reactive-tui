# Framework quality bar

Prefix: BAR
Scope: every commitment

These requirements apply to every commitment. A commitment is not done while
its work violates one of them. They are drafted from the developer's stated
target on 2026-09-21: a first-class framework for large, fast terminals,
without sanded edges.

[BAR-001] Every commitment MUST leave `cargo build --locked --workspace`, `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo doc --locked --workspace --no-deps` and `cargo test --locked --workspace --no-fail-fast` passing on Linux.
Falsifier: Any of the five commands exits non-zero at the commitment's final commit.
Mechanism: workspace-gates
Rationale: Today clippy fails in crates/libghostty-vt while CI lints the root crate only; the first commitment to touch that crate repairs it.
Status: Agreed 2026-09-21

[BAR-002] Every test added by a commitment MUST assert an observable outcome; a test that only prints, only checks `is_ok()` on a no-op, or passes without hardware by skipping silently MUST be named with a `smoke_` prefix and excluded from the correctness gate by name.
Falsifier: A test added in the commitment's range has no assertion on state or output and is not named `smoke_`, or a hardware-dependent test passes on a machine without that hardware without printing a skip reason.
Mechanism: assertion-audit
Status: Agreed 2026-09-21

[BAR-003] A widget delivered or reworked by a commitment MUST take its colors from the active Theme, fill the rectangle its parent allots when no explicit size is given, re-lay out on resize, and remain usable by keyboard, mouse and screen reader.
Falsifier: A delivered widget contains a hard-coded color, ignores a parent rectangle larger than its default size, paints stale geometry after a resize, or exposes an action reachable only by mouse.
Mechanism: widget-bar
Status: Agreed 2026-09-21

[BAR-004] A widget delivered or reworked by a commitment MUST render on the debug backend to goldens at two sizes, one of them at least 400 columns wide, and the goldens MUST be checked in.
Falsifier: A delivered widget has no golden, or its wide golden is narrower than 400 columns, or the golden test regenerates instead of comparing.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[BAR-005] A widget that animates MUST keep the App main loop under 16.6 ms per frame while animating at 700 columns by 200 rows on the debug backend, with rasterization on a worker thread.
Falsifier: The measured main-loop frame time exceeds 16.6 ms at that size during the widget's animation, or rasterization runs on the main thread.
Mechanism: frame-budget
Status: Agreed 2026-09-21

[BAR-006] A widget delivered or reworked by a commitment MUST have a page in the widget catalog example and a section in the manual whose builder methods match the code.
Falsifier: A delivered widget is absent from examples/widget_catalog or manual/, or a documented builder method does not exist in the code.
Mechanism: catalog-manual
Status: Agreed 2026-09-21

[BAR-007] A commitment MUST NOT add a document link, script input or test path that does not exist in the tree at its final commit.
Falsifier: A file added or changed in the commitment's range references a path that `git ls-files` does not list.
Mechanism: dangling-paths
Status: Agreed 2026-09-21
