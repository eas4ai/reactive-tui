# Widget catalog example

Status: Agreed 2026-09-15
Prefix: CAT

The repository needs one dependable showcase application for screenshots and
short recordings. It is an example, not a production application. It should
make the framework's widgets easy to find, exercise, and capture without
adding a second application architecture to the crate.

[CAT-001]
The repository MUST provide a `widget_catalog` Rust example with an overview
and pages for input, layout, data display, menus and dialogs, media, motion,
and system widgets. The pages MUST mount representative live instances of
every public widget family listed by the manual. Navigation MUST use a sidebar
at widths of at least 80 columns and a compact horizontal page strip below
that width. Arrow keys and the number keys shown by the catalog MUST select
pages. Resize events MUST change the navigation layout without restarting
the application.
Falsifier: The example is missing; a manual-listed widget family has no live
catalog instance; a displayed navigation key does not change pages; or the
wide and compact layouts do not follow the viewport width.
Mechanism: A source inventory check and DebugBackend behavior tests render all
pages, send the displayed navigation inputs, resize the viewport across the
breakpoint, and inspect the resulting frames.

[CAT-002]
The motion page MUST show a continuously rotating wireframe cube. Animation
MUST advance without keyboard input. Animation MUST request redraws at a
bounded cadence. Ctrl+Q, Ctrl+C, and Escape MUST all exit through the normal App
lifecycle and restore the host terminal.
Falsifier: Two animation samples separated by at least one frame interval are
identical; the example busy-spins; any displayed quit key fails to exit; or a
normal exit bypasses backend shutdown.
Mechanism: Deterministic cube-frame tests prove distinct bounded frames, while
a PTY smoke check launches the real example, observes two cube frames, sends
each advertised quit sequence in a separate run, and requires clean exit.

[CAT-003]
The media page MUST render the tracked project logo from
`manual/assets/logo.jpg` through the framework image widget. The example MUST
not require network access. The README and manual example inventory MUST name
the catalog command and its screenshot/video purpose.
Falsifier: The catalog downloads an asset, substitutes an unrelated image,
does not mount the image widget, or tracked documentation omits the runnable
command.
Mechanism: A tracked-input scan verifies the logo path and documentation, then
the locked toolchain compiles the example and runs its behavior tests.

The catalog does not claim to repair the known Kitty embedded-terminal crash.
The system page may present the terminal widget only through a bounded,
non-shell demonstration until that backlog item is resolved.
