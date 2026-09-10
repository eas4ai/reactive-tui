# Dialog lifecycle

Status: implementation and acceptance pending for API-012.

The dialog controls have App coverage in `docs/widget-acceptance.md`.
That evidence does not establish DialogEngine lifecycle behavior.

API-012 requires the engine to paint dialogs, route events, update active
dialogs, and deliver completion and cancellation results. Its acceptance
target is `api_dialog_lifecycle`. The target must verify stacking and limits,
close events, resource release, focus restoration, and asynchronous completion.
The engine currently discards close results and has no complete render,
event, or update interface. No engine readiness claim is made here.
