# Included AccessKit Linux adapter

Sources: accesskit_unix 0.23.0 and accesskit_atspi_common 0.20.0 from crates.io.
Exact archive SHA-256 values and upstream Git revisions are in provenance.json.
The licenses and upstream copyright notices are retained here.

These modules are private parts of reactive-tui. Including them in the library
sources ensures downstream consumers receive the state translation fixes; a
workspace Cargo patch table would not do that.

Mechanical changes: lib.rs becomes mod.rs; crate-local imports are qualified for
this private namespace; the Unix adapter always selects its async-io executor;
the unused simplified API stays disabled. These choices do not depend on the
parent library's optional Tokio feature. Unused facade reexports are omitted;
filtered_child_ids has an explicit receiver lifetime for Rust 2021 capture rules.

Behavior changes in translation/node.rs: expose Expandable and Expanded from
Node::is_expanded; expose Enabled and Sensitive only for enabled nodes, separately
from read-only state. The original state mapper omitted expansion and incorrectly
left disabled ordinary buttons enabled. Dedicated tests and the GNOME Terminal /
Orca integration check must pass before replacing these sources with upstream.

ScrollView role repair: expose the dedicated AT-SPI ScrollPane role; Pane remains
Panel. The real reader workflow verifies its label, focus and scrolled child input.

Focus repair in the same file: nodes supporting Action::Focus expose Component
even without pixel bounds. This makes grab_focus reachable; extents continue to
return the existing invalid rectangle when pixel coordinates are unknown.

Current-item repair: nonfalse AriaCurrent values expose Active and the exact
`current` object attribute. False stays inactive; absent values have no attribute.
The existing window/dialog active-state logic remains. This follows
https://w3c.github.io/core-aam/#ariaCurrent and Orca's current-item state reader.


Owned transport changes in unix/adapter.rs, context.rs, transport.rs and
atspi/bus.rs replace the process-wide singleton queue/worker. Each App has an
application context, a 4096-message outgoing queue and a cancellation signal.
Queue overflow and bus errors wake App and become returned errors; transport
errors are no longer converted into successful no-ops. Bus startup/publication
operations have three-second deadlines. Shutdown cancels pending operations and
joins the worker before releasing terminal resources. Normal cancellation does
not report an error. A fresh App can connect after an earlier App fails.

App's incoming action queue remains bounded to 64 requests; each event-loop turn
also drains at most 64, so a continuous producer cannot monopolize that turn.
The real fixture exercises simultaneous Apps and successive App lifetimes.
Private missing/stalled D-Bus endpoints exercise App errors and raw-mode/alternate-
screen restoration. Explicit screen_reader(true) reports unavailable connections;
automatic mode requires an interactive host with a desktop session-bus address.
