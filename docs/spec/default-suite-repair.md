# Default test-suite repair

Status: Agreed 2026-09-08
Prefix: DFT

The developer requested repair of the six failures in the refreshed default suite.

[DFT-001]
Numeric top, right, bottom and left utilities MUST change their inset without resetting z-index or the selected position mode.
Explicit position and z-index utilities MUST retain their existing last-wins behavior and position-layer defaults.
Falsifier: fixed z-50 top-0 becomes layer 10, any numeric side offset resets position/layer, or existing explicit-position ordering tests fail.
Mechanism: scripts/check-default-suite.py and tests/z_index_tests.rs.

[DFT-002]
For positive step counts, Steps(count, true) MUST jump at progress zero and exact interval boundaries, and finish at one.
Steps(count, false) MUST retain its jump-at-end behavior.
Falsifier: four-step jump-start returns zero at progress zero or 0.25 at progress 0.25, or end-step boundaries regress.
Mechanism: scripts/check-default-suite.py and animation step-easing tests.
Reference: https://www.w3.org/TR/css-easing-1/#step-easing-functions; normalized progress with no before flag.

[DFT-003]
The accordion, settings_accordion, faq_accordion and navigation_accordion documentation examples MUST compile and run as ordinary doctests.
Falsifier: an example lacks an import, is ignored or changed to non-executable text, or fails compilation/execution.
Mechanism: scripts/check-default-suite.py, Rust doctests.

[DFT-004]
The complete default-feature cargo test suite MUST pass without adding ignores or removing tests to conceal failures.
All inherited acceptance requirements MUST retain passing evidence.
Falsifier: cargo test --locked --no-fail-fast returns a failure or an inherited acceptance check regresses.
Mechanism: scripts/check-default-suite.py and inherited mechanisms.
