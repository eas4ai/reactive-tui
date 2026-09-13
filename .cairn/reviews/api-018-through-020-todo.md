# Remaining API commitment work

- Done: API-018 documentation, compatible Props defaults, public examples and supported-API matrix. Expanded formal acceptance and all 34 inherited requirements passed on the corrected source.
- Done: API-019 inventory and behavior checks for all residual audit concerns.
- In progress: API-020 image-capture timeout cleanup, current regression evidence, adversarial review and production self-audit.

Build and test concurrency is capped at 8. Existing approvals remain in force.

Current repair: restore complete descendant data through `as_element()` under
the approved API-016/API-019/API-020 decision. The unchanged five-test native
legacy consumer and focused 2,048-level small-stack snapshot test pass locally.
The full library and integration regression suite passed with eight jobs;
refreshed native evidence is pending. Windows
passed run 34775917767 on the confirmed current source; previous failed runs
had checked out a commit preceding its fixture repair. The macOS accessor
failure was deterministic, not a timeout. Ripwire quality-delta still reports
reference-source findings; test-gate lists affected callers rather than an
executable pass. Neither is counted as passing evidence.

Latest API-019 checkpoint: local scopes, mapping-test discovery, performance
ownership and Updater dispatch have focused passing checks. The approved
gesture route now binds hooks to mounted component ownership, applies thresholds
and keyed drop-zone options, preserves local/global coordinates, and unregisters
on removal. Click and gesture history is isolated between component owners.
Focused App and processor checks, Clippy, and the full library/integration suite
pass with the 8-job cap. Theme, Markdown/Syntect bounds,
editor selection, signals, legacy parsing, DebugBackend, raw mode, RenderTree,
nested events, transition metadata and CSS diagnostics are mapped into the
inventory mechanism. Combined checks and refreshed native records remain open.

The isolated libatspi state-set lifetime repair is complete, its Valgrind
falsifier passes, and refreshed API-011 evidence passes. The retained repair and
decision are recorded in the repository; it is no longer an API-019 blocker.
