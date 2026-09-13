# Remaining API commitment work

- Done: API-018 documentation, compatible Props defaults, public examples and supported-API matrix. Expanded formal acceptance and all 34 inherited requirements passed on the corrected source.
- In progress: API-019 inventory and behavior checks for all residual audit concerns; implement and verify any findings.
- Pending: API-020 image-capture timeout cleanup, complete current regression evidence, adversarial review and production self-audit.

Build and test concurrency is capped at 8. Existing approvals remain in force.

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
