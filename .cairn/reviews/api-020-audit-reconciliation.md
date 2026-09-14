# API-020 audit reconciliation

## API-020 final commitment review

Status: Complete

This immutable review artifact maps the complete Rust API audit to the accepted
requirements and their current mechanisms. The mutable commitment review records
the later post-evidence review and is deliberately outside this mechanism's input
footprint.

| Audit finding | Requirements | Reconciled behavior |
| --- | --- | --- |
| RAPI-01 signal allocation ownership | API-001 | Typed Rust and C lifecycles use matching allocations and checked ownership. |
| RAPI-02 component expansion and identity | API-002 | Nested and keyed App components update, retain identity and unmount once. |
| RAPI-03 hooks, effects and context | API-003, API-004, API-019 | Slots, effects, timers, context, references and App performance ownership have positive and violating cases. |
| RAPI-04 callbacks, routing and focus | API-005, API-006 | Painted bounds route input once; keyed and trapped focus restores correctly. |
| RAPI-05 Unicode editors | API-007, API-019 | Scalar positions and grapheme boundaries cover plain/syntax edits and selection. |
| RAPI-06 clipboard subprocesses | API-008 | Deadlines, errors, cancellation and reaping cover every claimed backend. |
| RAPI-07 styling, gradients and animation | API-009, API-010 | State variants, text tokens and paint properties change retained frames. |
| RAPI-08 widget and builder behavior | API-011 | The public inventory runs through App and the complete Orca workflow passes. |
| RAPI-09 dialog lifecycle and results | API-012 | Dialogs paint, route, return results, stack, cancel and clean up. |
| RAPI-10 keyframes and screens | API-013, API-019 | Interpolation, relative values, input, transitions and metadata limits are checked. |
| RAPI-11 images | API-014, API-020 | Decoding, placement, updates, removal, host routes and forced-timeout cleanup pass. |
| RAPI-12 features | API-015 | Default, minimal, optional, embedded and nightly SIMD configurations are checked. |
| RAPI-13 entry points | API-016, API-019 | Frame, patch and backend routes update and restore with owned input and parsing. |
| RAPI-14 native access | API-017 | C and TypeScript consumers exercise retained stateful native component APIs. |
| RAPI-15 documentation and residual coverage | API-018, API-019, API-020 | Public docs and examples pass, and every residual inventory row has executed coverage. |

The mapping names API-001 through API-020. API-019 additionally checks gestures,
Updater dispatch, theme/performance isolation, Markdown and syntax bounds, editor
selection, Unix input and SIGWINCH ownership, parsing, DebugBackend, raw mode,
RenderTree depth and width, nested events, test discovery, transition metadata,
CSS diagnostics and native platform records.

Historical defect controls are not acceptance evidence. The closure mechanism
rejects an incomplete mapping, unfinished work, mislabeled defect controls and
surviving process groups. It requires corrected normal and forced-timeout capture
runs. Approved host, platform, accessibility and binding limits remain documented
in `docs/supported-api.md`.

Known open findings: none
