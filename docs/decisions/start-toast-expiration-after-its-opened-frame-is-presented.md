# Start toast expiration after its opened frame is presented

Level: Judged
Decided by: Codex
Rests on: ABI-004,API-004,API-012
Would be wrong if: A slow first frame consumes the toast lifetime, a toast never expires after opening, or retained output can restart a timer after unmount.
History: Earlier API reversals exposed assumptions about timing and native presentation. This repair follows an observed default-suite failure and a controlled slow-render reproducer; it preserves the existing toast duration and animation APIs and requires actual captured frames plus cleanup tests.

## Decision

Start the App-owned toast deadline only after the modal has measured its body and successfully presented the fully opened frame. Carry a private presentation callback through the existing modal and layout callback path. Preserve duration changes, persistent toasts, manual closing and unmount cleanup. Exercise short and zero durations with animation enabled and disabled while another component delays the initial frame. Keep the separate compiler stall recorded as unexplained; this change does not claim to repair it.

## Realized by

(none yet: recorded, not built)

- a44c078ad57cac491d0488ebd8439d154e893d22 Start toast expiration after its opened frame is presented
