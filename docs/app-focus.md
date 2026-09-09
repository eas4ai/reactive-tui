# App focus and traps

App's public focus methods and its keyboard/mouse router share one focus owner.
Rendered elements keep IDs by parent-scoped key, or by position when unkeyed.
A redraw does not change focus or repeat focus callbacks. Reordering keyed
siblings preserves the current target and updates the order used by the next Tab.

Focusable controls without a tab index use their rendered order. Positive tab
indices come first, in ascending order; equal indices use rendered order. Zero
follows the positive indices. Negative indices skip Tab navigation but remain
available for explicit focus. Tab moves forward; Shift+Tab and BackTab move
backward. Key release does not navigate. Ctrl/Alt/Meta-modified Tab remains
available to application input handlers.

`auto_focus` requests focus when the element mounts or when the flag changes
from false to true. Keeping the flag true across redraws does not steal focus.
Requests outside the active trap are ignored. Focus transitions notify the old
element's `on_blur` before the new element's `on_focus`, exactly once per change.
Removed nodes receive blur before their registrations are released.

`trap_focus` confines keyboard, spatial, mouse and explicit focus to that
container's focusable descendants. Opening a trap selects a descendant requesting
autofocus, or the first eligible control. A focusable container can receive focus
if it has no focusable descendants. An empty non-focusable trap keeps focus empty
and consumes Tab navigation; Enter cannot activate an outside element through
the router's root fallback.

Traps retain their opening order, membership and restoration target across
redraws. The most recently opened live trap is active. Closing an inner trap
reactivates the preceding trap. With `restore_focus`, closing or removing the
container restores its remembered target if that target is still eligible in
the current scope; otherwise focus falls back to the first eligible control.
Removing a parent with several nested traps releases them together and never
restores a removed target. Removing a subtree through EventRouter also releases
its traps.
An explicit imperative `create_focus_trap` call can reactivate an existing
container; ordinary declarative redraws only update its membership.

A focus trap alone is not a modal pointer-event backdrop: clicking an outside
control may invoke its callback, but cannot move keyboard focus outside the
trap. Dialog painting, backdrops and result delivery are governed separately
by API-012.

The acceptance runner is `scripts/check-api-focus.py`. It uses captured SuprTUI
frames, keyboard and mouse input, callback logs, and direct public focus APIs.
This document does not claim that the remaining widget/dialog recovery is done.
