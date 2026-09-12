# Residual API behavior inventory

API-019 requires a contract and falsifier before implementing each residual concern.
These entries preserve the audit coverage table and findings from the API-018
public example review. They are open work, not accepted limitations or permission
to remove an API. The commitment review contains the earlier detailed probes.

| Concern | Required contract | Falsifier | State |
| --- | --- | --- | --- |
| Hover, drag, drag-and-drop, mouse position, clicks, long press, swipe and wheel hooks | Real routed component events update the retained hook state; thresholds, drop zones, coordinates and owner cleanup are respected | Advertised hook remains a default signal, ignores options or survives its owner | Pending API-019 |
| Public reference hooks | References retain identity/value through component renders without adding redraws; callback updates are reentrant and cleanup has explicit ownership; local references remain thread-confined | Rerender replaces stored values, callback reentry deadlocks, or a local reference crosses threads | Pending API-019 |
| ui::Updater | Registered updates reach the intended state/component and preserve defined ordering and ownership | Updater remains a marker or an update is discarded | Pending API-019 |
| Theme propagation | Parent themes reach descendants; overrides, changes and independent Apps remain isolated | Theme is ignored, stale, or leaks between Apps | Pending API-019 |
| Performance context | Metrics and requested modes belong to the correct App, with accurate mode reporting and context inheritance | Global publication/request state changes another App or always reports Auto | Pending API-019 |
| Markdown/Syntect integration | Parsed content and highlighted code reach styled output with documented options, spans and error behavior | Conversion compiles but loses promised rendered content or styles | Pending API-019 |
| Large Markdown/syntax input | Time, memory, depth and output are bounded or fail with an actionable error | An adversarial valid input hangs, overflows, or grows without the documented bound | Pending API-019 |
| Editor undo/selection | Undo/redo groups, selection replacement, movement and grapheme/scalar boundaries follow the documented model | Mixed Unicode editing cannot restore text/selection or corrupts a position | Pending API-019 |
| Unix input worker | Raw and parsed async input have one live descriptor owner and bounded queues; dropping the receiver or final session owner stops and joins owned workers, including while idle, before descriptor reuse | A worker survives consumer/session removal, reads a reused descriptor or queues without a bound | Pending API-019 |
| SIGWINCH ownership | Resize delivery does not take unsafe locks in the signal handler and restores prior ownership correctly | Signal callback locks a mutex, loses another owner's handler or leaks after cleanup | Pending API-019 |
| Legacy input parsing | Mouse buttons/modifiers, unknown keys and split UTF-8/escape sequences are retained correctly across reads | A supported input becomes Unknown, is discarded, or is corrupted at a read boundary | Pending API-019 |
| DebugBackend boundaries | Zero/oversized dimensions and mixed frame/patch input have defined bounded results | u16 truncation, invalid geometry or stale cells escape the contract | Pending API-019 |
| Raw-mode ownership | Independent retained terminal owners cannot restore modes while another owner still requires them | One owner's drop disrupts another live session or fails final restoration | Pending API-019 |
| Public RenderTree | Fragments, custom nodes, deep and wide trees preserve output with bounded traversal/diff costs | Advertised nodes disappear, recursion overflows, or work grows pathologically | Pending API-019 |
| Nested legacy events | Input reaches retained nested targets exactly once with correct focus and cleanup | Root-only dispatch loses a nested target or calls a stale handler | Pending API-019 |
| Legacy backend test reachability | Key, focus, paste, resize and mouse mapping tests are registered and executed by the Rust test harness | A named mapping test is missing from discovery or an intentional mismatch does not fail its executed test | Pending API-019 |
| Transition integration metadata | Reconcile the documented animation ID/custom-property bridge and hardware preference with the agreed terminal painter; implement promised behavior or obtain an explicit contract decision | Metadata is presented as working integration while the renderer never reads it | Pending API-019 |
| CSS property diagnostics | Reconcile the public compile-time property/value validation claim with accepted property types and actual terminal behavior | Unknown/mismatched properties silently succeed despite the promised diagnostic | Pending API-019 |
| Full claimed native platform surface | Every retained macOS/Windows/Unix path is tied to native evidence or an explicitly approved scope choice; reconcile the four recorded Windows warnings and six FFI warnings against current diagnostics | A platform claim relies only on Linux compilation or an untested placeholder, or a recorded diagnostic disappears without review | Pending API-019 |
| Image capture timeout ownership | The actual check runner reaps its private terminal/Xvfb descendants on timeout and cancellation, including separate sessions | The driver dies but an owned host or Xvfb process survives | Pending API-020 |

API-018 separately repairs the responsive macro's compile failure and discarded
breakpoint values under the existing API-009 style contract. That repair needs
its own boundary/resize/failure tests; this inventory does not defer it.
