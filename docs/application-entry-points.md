# Application and terminal entry points

The supported App routes below retain their public constructors. Crossterm and
DirectTty adapt their terminal/input ownership to the shared SuprTUI painter.
Custom patch backends receive the complete candidate tree and actual changes
against the last successfully presented tree, including one real root insertion.

| Public route | Required behavior | Evidence |
| --- | --- | --- |
| `App::builder` with `SuprTuiBackend` | Complete styled grapheme frames; component input; updates, removal, resize, errors and terminal restoration | API-016 host consumer; inherited RND and API component checks |
| `App::builder` with `CrosstermBackend`, including `set_use_render_ops(false)` | Incremental output by default; disabling the option redraws complete frames with the same cells and geometry; component input and frame lifecycle | API-016 host consumer in both modes and manual output comparison |
| `App::builder` with `DirectTtyBackend` | Retained direct terminal route; complete updates, component input, resize and restoration | API-016 host consumer |
| `App::builder` with a custom patch backend | Real initial insertion; diff against the last presented tree; unchanged frames have no patches; backend receives the candidate tree; errors reach the caller and shutdown runs | Separately compiled API-016 legacy consumer |
| `DebugBackend` | Complete in-memory App frames with graphemes and acknowledged input geometry; explicit legacy patch calls retain their history; no host-terminal ownership | API-016 DebugBackend App/input and frame consumers; debug-backend library tests |
| C App builder | SuprTUI selection; callback transfers one owned Element per render; resize updates, quit, failed-build restoration; repeated terminal selection retains one session | Compiled API-016 C consumer and inherited ABI consumers |
| `core::Renderer`, `core::Terminal` and their retained C entry points | Explicit surface/frame and host-session ownership, resize, output errors and cleanup | API-014 surface/renderer tests and ABI/MNT terminal consumers |
| `platform::DirectTty` and platform-specific TTY types | Native input/output and terminal restoration; Unix clones share descriptor ownership; their retained App route is DirectTtyBackend | API-016 DirectTty host, batched-input and Unix clone workflows; API-019 must reconcile the remaining platform claims |
| `terminal::Terminal`, `PseudoTerminal`, `TerminalWidget` | Owned child PTY, input/output, resize and child cleanup | API-011 retained-terminal tests and current macOS/Windows/ConPTY records |
| `embedded::Session` and `TerminalView` (Unix, embedded-terminal feature) | Owned Ghostty session composed through App/SuprTUI | Inherited EMB-001 through EMB-006 |

API-016 consumers use shipped public interfaces. The Rust legacy consumer records
the patches and frames a caller actually receives. The host consumer drives a
real controlling PTY at 32x8 and 48x12, checks its interpreted cells, clicks a
painted button, sends keys, reverses and removes children, resizes, then checks
normal and error exits. The C consumer compiles against shipped headers, retains
its callback state until `rtui_app_run` returns, and never reuses the consumed App
or builder handle. The failed-build case must restore the terminal too. Terminal
selection acquires its session immediately. Selecting SuprTUI or Crossterm again
retains that session; selecting DebugBackend releases it. The manual consumers
also test one-patch updates, root removal, output optimization, visible debug
overlay, batched input, idempotent shutdown and Unix clone/drop restoration.

The host captures in this mechanism run on Unix. Cross-platform terminal and
widget claims also require their existing native records and the API-019 inventory;
the Linux PTY result alone does not certify every Windows/macOS platform operation.

Unix input uses the locally maintained Crossterm 0.29.0 source in
`src/backend/crossterm`. Its narrow readiness repair retains queued input
across resize/wake events and read-buffer boundaries. The independent
API-016 consumer queues 2,048 bytes before releasing its reader, requires
all bytes without another input write, and checks exhausted zero-timeout
polls and terminal restoration. Dependency provenance and changes are in
`src/backend/crossterm/REACTIVE_TUI_PATCH.md`.
