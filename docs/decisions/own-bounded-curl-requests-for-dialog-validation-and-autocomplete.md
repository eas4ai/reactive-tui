# Own bounded curl requests for dialog validation and autocomplete

Level: Judged
Decided by: Codex
Rests on: API-011,API-012,API-015
Would be wrong if: A request blocks App input, survives its dialog or replacement, exposes request secrets in command arguments or logs, or a failed or stale response can confirm a dialog.
History: Earlier API and clipboard reversals require proving cancellation and native process cleanup at the real ownership boundary. Existing dialog HTTP options have no working transport or response contract.

## Decision

Use curl 8.4 or newer as the explicit runtime prerequisite for configured dialog HTTP endpoints, with an observable error if unavailable. Reuse the bounded child-process ownership already proved for clipboard operations by extracting its private runner without changing clipboard behavior. Run requests on an owned worker, cancel and join on replacement or removal, and consume results through the dialog owner and scheduler. Send JSON POST bodies: {"value": string} for validation and {"query": string} for autocomplete. Accept only successful HTTP status codes; validation returns {"valid": boolean, "message": optional string, "warnings": optional string array}, and autocomplete returns an array of strings or suggestion objects. Bound each request to five seconds and each response to 64 KiB; bound serialized request configuration to 1 MiB. Disable curl config files, URL globbing, redirects and non-HTTP protocols; preserve certificate verification. Pass URL, headers and payload through private stdin, not process arguments. Debounce change requests, preserve submission vetoes, reject stale results, and show transport or parse errors. Verify loopback success, rejection, malformed responses, deadlines, replacement and removal with actual requests and child cleanup; retain explicit platform evidence limits.

## Realized by

Implementation: `src/widgets/dialog/http.rs`, `src/widgets/dialog/input/live.rs`, `src/widgets/dialog/autocomplete/live.rs`.

Behavior checks: `tests/api_widget_behavior/dialog_http.rs`, `scripts/check-dialog-http.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
