# Pre-release external input safety

Status: Agreed 2026-09-14
Prefix: XIS

This specification remediates Fable audit findings M2, L8, L9, and L11. It
covers network requests, image inputs, external image renderers, and file
operations that can change between inspection and use.

[XIS-001]
Dialog remote validation and autocomplete MUST make network access explicit,
bounded, and documented. The request process MUST receive only the environment
needed for supported proxy and certificate behavior.
Falsifier: Typing into a dialog can send an undocumented request, follow an
unsupported protocol, exceed its time or response limit, or expose unrelated
parent environment values to the request process.
Mechanism: A fake HTTP server and fake curl executable record request timing,
arguments, stdin, environment, response limits, cancellation, and process
cleanup for enabled and disabled configurations.

[XIS-002]
Image decoding MUST enforce one total memory budget across decoded storage and
required copies. External renderers MUST treat every image path as data, even
when a relative path begins with a hyphen.
Falsifier: A permitted encoded image causes peak image storage to exceed the
documented limit, or chafa or viu parses an image path as an option.
Mechanism: Adversarial image fixtures measure allocations at the boundary, and
fake renderer executables verify the exact argument separator and path.

[XIS-003]
File-explorer remove and copy operations MUST act on the same in-root entry
that was validated, or fail before changing another entry.
Falsifier: Replacing an entry between metadata inspection and opening makes an
operation remove or copy a different entry inside the configured root.
Mechanism: A race-focused filesystem test repeatedly replaces the selected
entry and accepts only the original identity, a safe retry, or a clear error.
