# Share validated image decoding and preserve local URL behavior

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014
Would be wrong if: The shared decoder changes accepted pixel semantics or network loading is an existing promised behavior rather than the documented future extension.
History: Earlier API reversals concerned clipboard command deadlines and accessibility adapter provenance. They require explicit limits and retained dependency ownership here; this local decoder decision does not change platform claims or revise those choices and remains Judged.

## Decision

Decode file, base64, encoded-memory and raw RGB/RGBA sources into one checked RGBA image before rendering. Retain base64 image data URLs and the existing local-path Url behavior. HTTP and HTTPS return an explicit unsupported-source error without network access; ImageSource currently documents these as future support. Do not add a network client. Reject malformed data URLs, mismatched raw byte counts and zero dimensions. Bound encoded inputs to 64 MiB and decoded RGBA storage to 256 MiB, reporting a limit error rather than risking an unbounded allocation. External tools receive renderer-owned encoded PNG files for memory sources. Preserve caller-owned file paths and all public renderer signatures. Graphics placement, host evidence and the App adapter remain required separately.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

The widget image family now shares checked RGBA decoding for file, base64,
encoded-memory, data-URL and raw pixel sources. Native tests compare known pixels
and reject invalid data and extents. External renderers retain caller files and
own temporary PNG files for memory sources. The parallel platform image API now
shares this decoder through the later platform-image decision. See
docs/widget-acceptance.md for evidence and limits.
