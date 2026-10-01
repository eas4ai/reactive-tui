# Framework quality bar

Prefix: BAR
Scope: every commitment

These requirements apply to every commitment. A commitment is not done while
its work violates one of them. They are drafted from the developer's stated
target on 2026-09-21: a first-class framework for large, fast terminals,
without sanded edges.

[BAR-001] Every commitment MUST leave `cargo build --locked --workspace`, `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo doc --locked --workspace --no-deps` and `cargo test --locked --workspace --no-fail-fast` passing on the Linux development host with default features.
Falsifier: Any of the five commands exits non-zero at the commitment's final commit.
Mechanism: workspace-gates
Rationale: Clippy fails today in crates/libghostty-vt (140 style findings) while CI lints the root crate only; charts-plot-layer repairs that crate so the gate holds from the first commitment on.
Status: Agreed 2026-09-22

[BAR-002] Every test added by a commitment MUST assert an observable outcome with an assert macro, a panic, or a `#[should_panic]` attribute; a test that only prints, only constructs values, or passes without hardware by skipping MUST be named with a `smoke_` prefix, and a hardware-gated skip MUST print `SKIP` with the reason. Tests present before this bar that violate it are repaired or renamed by the first commitment under the bar.
Falsifier: A test not named `smoke_` has no assert macro, panic or `#[should_panic]`, or a hardware-gated test returns early without printing `SKIP`.
Mechanism: assertion-audit
Status: Agreed 2026-09-22

[BAR-003] A widget delivered or reworked by a commitment MUST take every color from the active Theme through the layout's color resolver, MUST fill the rectangle its parent allots when no explicit size is given, MUST re-lay out after a terminal resize, and MUST expose every pointer action through the keyboard as well and describe its state to the screen reader.
Falsifier: A delivered widget contains a color literal, paints fewer columns than a parent wider than its default, paints stale geometry after a resize event, exposes an action only through the mouse, or has an accessibility node with no description of its state.
Mechanism: widget-bar
Status: Agreed 2026-09-22

[BAR-004] A widget delivered or reworked by a commitment MUST render on the debug backend to checked-in goldens of the text grid plus a color digest at two sizes, one at the widget's largest useful width and at least 400 columns for a widget whose layout scales with width.
Falsifier: A delivered widget has no golden, its wide golden is narrower than 400 columns although its layout scales with width, or the golden test regenerates instead of comparing unless REGENERATE=1 is set.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[BAR-005] A widget that animates MUST keep the App's work per frame, measured from the input wait returning to the frame being presented, under 16.6 ms while animating at 700 columns by 200 rows on the debug backend; a widget whose paint cost scales with its area MUST rasterize off the main thread.
Falsifier: The measured per-frame work exceeds 16.6 ms at that size during the widget's animation, or an area-scaling widget rasterizes on the main thread.
Mechanism: frame-budget
Rationale: Wall time divided by frames cannot be used: the loop paces presents to the FPS manager, so that ratio is at least one frame period on any host.
Status: Agreed 2026-09-22

[BAR-006] A widget delivered or reworked by a commitment MUST have a page in the widget catalog example and a section with a heading of its own in the manual, and every builder method the manual cites for it MUST exist in the code.
Falsifier: A delivered widget is absent from examples/widget_catalog, has no manual heading, or the manual cites a builder method that does not exist.
Mechanism: catalog-manual
Status: Agreed 2026-09-22

[BAR-007] A commitment MUST NOT leave a document link, script input or test path, in any form including relative `../` links, that does not exist in the tree at its final commit; a dated report (`docs/recon.md`, `docs/widget-study.md`) and a record of what was decided or printed on its day keep the paths they named and do not count.
Falsifier: A tracked file that is neither a dated report nor such a record references a repository path that `git ls-files` does not list.
Mechanism: dangling-paths
Rationale: Whole-tree, not range-based: deleting a target strands references in files that did not change, which is how the README and a dozen scripts broke before this bar existed.
Status: Agreed 2026-09-29

[BAR-008] Every commitment MUST leave `cargo deny --workspace --locked check advisories bans licenses sources` passing at its final commit, against the RustSec advisory database as cargo deny fetches it on the day of the check, and every advisory ignore and bans skip entry in deny.toml MUST state its reason.
Falsifier: The command exits non-zero at the commitment's final commit on a database fetched that day, or an advisory ignore, bans skip or skip-tree entry in deny.toml gives no reason.
Mechanism: dependency-checks
Rationale: GitHub CI is off, so nothing ran cargo deny: a duplicate sha2 and a discontinued async-std went unnoticed until someone ran it by hand.
Status: Agreed 2026-09-26

[BAR-009] Every commitment MUST leave `cargo build --locked -p reactive-tui --all-targets` passing with no warning on the macOS test host and on the Windows test host at its final commit.
Falsifier: At the commitment's final commit, the build fails or prints a warning on either host.
Mechanism: host-builds
Rationale: The Linux gates compile only the Linux side of each platform gate and with Rust 1.95, so 26 warnings grew unseen on macOS and Windows, two of them future errors (non-linux-build-warnings).
Status: Agreed 2026-09-27

[BAR-010] Every commitment MUST leave `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo doc --no-deps` and `cargo test`, each with `--locked -p reactive-tui --features wgpu-graphics`, passing on the Linux development host, and `cargo build --locked -p reactive-tui --all-targets --features wgpu-graphics` passing with no warning on the macOS and Windows test hosts, at its final commit.
Falsifier: At the commitment's final commit, any of the four commands exits non-zero on the Linux host, or the host build fails or prints a warning on either test host.
Mechanism: canvas-gates
Rationale: The gates build default features only, so nothing compiled src/graphics after 2026-09-19; the developer confirmed on 2026-09-27 that the canvas stays an optional feature with its own gate.
Status: Agreed 2026-09-27
