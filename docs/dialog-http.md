# Dialog HTTP contract

The recovery implementation connects `InputDialog` rendered through App to remote
validation and `AutocompleteDialog` to remote suggestions. The direct dialog-engine
route remains unfinished under API-012. This document does not declare catalog-wide
acceptance.

Configured HTTP endpoints require **curl 8.4 or newer** on the executable search
path. Local validation needs no external tool. A missing or older curl produces a
visible error. The version floor allows curl to stop oversized responses even
when their length is not known before transfer.
See the [curl transfer-limit documentation](https://curl.se/docs/manpage.html#--max-filesize).

## Validation

Set `ValidationConfig::async_validation_url` to an HTTP or HTTPS endpoint. The
dialog sends a JSON POST request with `Content-Type: application/json` and
`Accept: application/json`:

```json
{"value":"the current input value"}
```

The endpoint must return a 2xx response containing:

```json
{"valid":false,"message":"This value is already in use","warnings":[]}
```

`valid` is required and must be a boolean. `message` is optional; rejection without
a message gets a generic explanation. `warnings` is an optional array of strings
with at most 64 entries. Warnings are displayed and do not block submission.
Malformed JSON, an HTTP error, a transfer failure or an oversized response keeps
the dialog open with an error.

Local rules run first. Change validation follows the configured debounce delay;
blur validation runs when the field loses focus. Submitting while a matching
request is pending records the submission intent. A successful response then
rechecks local rules and invokes the current submission callback. A callback veto
keeps the dialog open and clears the accepted remote result, so another attempt
can request validation again. A successful change validation may be reused for
submission while the value and endpoint remain the same; current local rules still run.

An edit, changed endpoint, close or unmount cancels the old
request. Its result cannot confirm the new value. Recreating validator callbacks
on redraw keeps a matching request alive; completion uses the current callbacks.
Changed debounce settings reschedule pending change validation. Removal cancels owned work;
full dialog-engine completion/cancellation delivery is still an API-012 obligation.

## Suggestions

Set `AutocompleteConfig::suggestions_url` to use a remote source. Requests use the
same transport and limits as validation, with configured `headers` and this body:

```json
{"query":"the current input value"}
```

Return a 2xx response containing an array of strings, suggestion objects, or both:

```json
["plain value", {"value":"stored-id", "display":"Shown label", "description":"Details", "icon":"*", "metadata":{"kind":"item"}}]
```

Objects require a string `value`. The remaining fields are optional; `metadata`
maps strings to strings. Invalid responses and transport failures display an error.
`max_suggestions` limits displayed results, and an empty array displays
"No suggestions". An edit replaces the pending debounce or request; removed and
closed dialogs cancel owned work. Callback replacement alone does not restart HTTP.

Without a URL, filtering uses `static_suggestions` immediately. The default filter
performs a case-insensitive substring match; `filter_function` replaces it.
`min_chars` counts grapheme clusters, including combining text and joined emoji.
It applies to both sources. `debounce_delay` applies to HTTP requests.

Up/Down navigate results, including results outside the visible list. The wheel
scrolls the suggestion rows. Enter, Tab and suggestion clicks select the current
result; OK uses the same selection callback. A false `on_select` result keeps the
dialog and draft unchanged. With no selected result, Enter/OK confirms the typed
value. Escape dismisses suggestions first, then follows `escape_closable`.

The default renderer includes the icon and optional description, with matching
whole graphemes underlined and bold when `highlight_matches` is enabled.
`show_descriptions` controls description visibility. A `suggestion_renderer`
replaces this content and receives the full suggestion, including metadata.
Stable dialog identity preserves edits and selection across redraws; a changed
authored default value replaces the draft, and changed callbacks apply to the
next action.

## Transport ownership and limits

Each dialog owns at most one request worker. Cancellation stops and reaps curl,
then joins that worker. Process startup, version checking and transfer share a
five-second deadline. Serialized request configuration is limited to 1 MiB;
response bodies are limited to 64 KiB, including chunked transfers.

URL, headers and JSON payload travel through private stdin, not command arguments.
Curl configuration files and URL globbing are disabled. Only HTTP and HTTPS are
allowed, redirects are not followed, and normal TLS certificate verification
remains enabled. Standard proxy and certificate environment settings still apply.
Transport errors do not log request data or response bodies.

## Verification so far

Linux checks used curl 8.21.0 and actual loopback HTTP connections. App workflows
cover Unicode JSON submission, rejection and correction, HTTP/JSON errors at two
viewport sizes, replacement during a request, removal and submission vetoes.
Autocomplete App cases cover JSON queries and headers, structured results,
malformed responses, debounce coalescing, replacement and removal at both sizes.
Transport tests cover escaped JSON and headers, a silent endpoint, chunked size
limits, invalid configuration, a missing executable and cancellation that closes
the actual connection and joins its worker. The extracted process runner retains
the clipboard tests for deadlines, cancellation and child cleanup.

The HTTPS fixture additionally ran the explicit Rust probe twice against an
ephemeral local certificate: an explicitly trusted certificate succeeds, while an
untrusted certificate is rejected before any HTTP request reaches the handler.
The ordinary test run ignores this fixture probe; `scripts/check-dialog-http.py`
executes it and rejects a run that selected no test. The widget mechanism invokes
that fixture separately.

Native snapshot 34503199020 passes the HTTP transport and fourteen App cases on
Windows and macOS. macOS also passes both HTTPS trust cases. Windows HTTPS trust
setup still needs acceptance: importing the temporary certificate into the user
store timed out before either request ran. The fixture now uses the dedicated
machine store and removes only its freshly generated certificate by fingerprint
before testing rejection. Windows execution requires `--dedicated-desktop`;
application TLS verification remains unchanged. The corrected fixture needs a
native rerun.

These are editing checks, not current Cairn receipts. Orca/GNOME Terminal has
verified the App input and autocomplete workflows; engine lifecycle and final
catalog acceptance remain pending in [the widget inventory](widget-acceptance.md).
