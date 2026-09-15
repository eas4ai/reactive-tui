# Review: pre-release-external-input-safety

commitment: pre-release-external-input-safety
commit: e7161136e939bc44cf940ac44fafc94baf2eda7c
findings:
  - resolved: XIS-003 now runs 32 independent replacement rounds for copy and 32 for remove.

## Scope examined

Re-read XIS-001 through XIS-003, the three mechanism declarations, their
validators and adversarial fixtures, the implementation changes, the three
decisions, the manual limits, and the latest committed receipts. Compared the
current tree with commitment start `417ab07d` and the Fable findings assigned
to this commitment.

XIS-001 records curl arguments, stdin, environment, request timing, response
limits, cancellation, and cleanup. It checks disabled dialog configurations
and enabled validation and autocomplete triggers. XIS-002 checks renderer
argument separation and a compressed RGB image whose complete decoder and
conversion storage exceeds the static budget. XIS-003 swaps the selected
identity after metadata inspection with a deterministic hook and accepts only
the original identity or a clear error that preserves the replacement.

## Failure demonstrations and corrected cases

The XIS-001 validator rejects unrelated inherited environment, missing
supported proxy or certificate values, changed curl arguments, non-stdin
request data, a missing request, and a disabled configuration that spawns
curl. The corrected receipt includes the focused unit suite, trusted and
untrusted TLS probes, and 17 dialog HTTP acceptance cases.

The XIS-002 validator rejects missing or late `--` separators and missing
renderer invocations. Its oversized valid RGB fixture passed before complete
storage was counted and now fails before simultaneous decoder and RGBA storage
can exceed 256 MiB. Smaller RGB, RGBA, grayscale, and animated cases pass.

The XIS-003 validator rejects copied replacement bytes and deletion of a
replacement while the original remains. The committed baseline reproduced
both failures. The corrected copy and remove cases now return a changed-entry
error after every forced swap. Each operation now runs 32 independent rounds,
which satisfies the mechanism's repeated-replacement wording and also checks
that test-hook and staging state do not leak between operations.

## Resolved finding

The initial XIS-003 fixture performed one deterministic replacement for copy
and one for remove. Commit `6a22f0cc` repeats both cases 32 times with fresh
filesystem identities. Receipt `20260915T140628649Z` records the corrected
mechanism passing all 64 replacement rounds and its validator.

## Other limits

Regular-file and symbolic-link removal ultimately uses a capability-scoped
pathname operation after identity revalidation; the recorded decision names a
later replacement in that final syscall window as its wrong condition. The
current falsifier targets replacement between metadata inspection and opening.

The complete file-explorer worker suite passed 14 tests and no-dependency
library Clippy passed with eight jobs. A Windows cross-check could not reach
project code because `onig_sys` requires the unavailable
`x86_64-w64-mingw32-gcc` compiler. Ripwire found unchanged function contracts;
its quality delta reported expected short-horizon churn because Cairn's
failing declaration and repair both touch `copy_entry`.
