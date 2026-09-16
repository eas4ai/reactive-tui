commit: f119ef8d16e1a870eda2564308a8dd2e0d5c86c4
examined:
  - CAT-001 through CAT-003 and the widget-catalog commitment.
  - Catalog source, live widget inventory, navigation, cube clock, and quit lifecycle.
  - Behavior and validator tests, PTY output, and all three Cairn receipts.
  - README, manual, local logo, and mechanism dependency declarations.
findings:
  - open: CAT-001 InputDialog and AutocompleteDialog are labels, not live instances. Replace them with local interactive dialogs and make each overlay separately reachable for capture.
  - open: CAT-001 The compact strip gives every entry full width and has no measured-frame assertion. Verify all numbered shortcuts remain visible after resize; fix overflow if observed.
  - open: CAT-001 The Table demo has no columns or rows, and Modal is hidden with no opening control. Give them representative, reachable states.

## Scope examined

Compared CAT-001 through CAT-003 with the catalog source, tests, checker,
mechanism declarations, README, and manual. No framework API or dependency
changed. Reviewed the actual PTY output and terminal restoration assertions.
The nine behavior tests pass, as do the ten validator falsifier tests and all
three committed Cairn mechanisms. Those passes do not resolve the findings
above: the source inventory accepts family names inside a coverage string.

## Mechanism attacks

Validator fixtures reject missing pages, the wrong/network logo, missing test
evidence, identical animation samples, failed quit sequences, and omitted
documentation. Three isolation fixtures prove unfinished requirements cannot
fail an unrelated requirement. The corrected shell fixture passes.

The real PTY check hashes only visible cube strokes and vertices, not changing
labels or ANSI output. It observes two distinct frames separated by at least
160 ms without input, then checks Ctrl+Q, Ctrl+C, and Escape in separate runs.
Each normal exit restores termios, alternate screen, cursor, and child lifetime.
Deterministic animation tests check an 80 ms deadline and skip missed frames.

## Limits and required repair

The current tests inspect the element tree, not a measured DebugBackend frame.
They therefore do not prove compact navigation geometry or live dialog-family
coverage. Add those assertions before the final review. The logo path is local
and tracked, but verify its real decode and mounted image props as well.
The Kitty embedded-shell crash remains explicitly outside this commitment.
No executable code changed while this review was performed.
