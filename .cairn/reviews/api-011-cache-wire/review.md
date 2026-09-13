# AT-SPI cache wire repair and remaining client lifetime failure

Current commitment: rust-api-remediation; inherited regression API-011.
API-019 and API-020 remain unfinished.

## Observed failure

Formal receipt API-011/20260913T142154903Z failed after the widget tests,
reader delivery and data-control observations. Orca aborted with heap-corruption
diagnostics before the final reader-survival assertion. Its original session,
reader and launch logs and coredump information are preserved here.

The reader also rejected our cache signals. GNOME specifies AddAccessible as
one ((so)(so)(so)iiassusau) argument and RemoveAccessible as one (so) argument:
https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Cache.html
The old emitter passed each structure directly as the D-Bus argument list,
flattening its outer structure. The repaired emitter wraps the structure in
one argument and uses the static Type bound required by tuple serialization.
Only these two private cache-signal call sites use the emitter.

## Verification and remaining limits

The reader runner now rejects the specific malformed AddAccessible and
RemoveAccessible diagnostics. The unchanged old binary fails that new check
(cache-before.out), even though its other observations succeed. The first
script edit had an indentation error (before.out), corrected before that
failure demonstration. The first build needed the static Type bound
(build.out); build-corrected.out records the successful corrected build.

The valid RemoveAccessible messages exposed a remote traversal race:
a node can disappear during a predicate. The existing bounded observation
loop now retries only the exact AT-SPI disappearance error. User actions and
deadlines are unchanged. check-observation-retry.py exercises the actual
functions extracted from the runner: a transient removal reaches the required
replacement, permanent disappearance fails within the same deadline, and
unrelated error messages/domains/codes propagate immediately. All passed.

Three corrected 60x16 data-control workflows passed, including genuine speech,
focus, removal and Orca survival. Cache unit tests, workspace formatting,
Python syntax and strict library Clippy passed. The broader reader run passed
both semantic negative controls and the 32x10 base workflow, then aborted in
the Python AT-SPI client during the 60x16 base workflow. No malformed-cache
warnings remained. Corrected crash logs and coredump metadata are preserved.
The cache repair therefore does NOT establish that the heap crashes are fixed.

## Independent memory diagnostic

A private, uninstalled Valgrind 3.26.0 package instrumented the client using the
installed libatspi 2.60.0-1 and GLib 2.88.0-1. Its complete log reports one
invalid read in atspi_state_set_contains, reached through
_atspi_accessible_test_cache and atspi_accessible_get_child_count. The state
object was freed through g_object_run_dispose while _atspi_dbus_call was active
under the same state query. The read then used memory 32 bytes into that
freed 40-byte allocation. This demonstrates a client-library use-after-free.
It does not prove that every earlier heap abort has that same root cause.

The diagnostic wrapper only, retained as valgrind-orca-diagnostic.py, extends
observation and process deadlines to accommodate instrumentation. The repository
acceptance deadlines remain unchanged. The diagnostic process exited zero,
but Valgrind reported an error: this is defect evidence, NOT an acceptance pass.
No desktop library was patched or installed. Relevant upstream implementation:
https://github.com/GNOME/at-spi2-core/blob/main/atspi/atspi-stateset.c

## Production self-audit

The repository slice preserves API signatures and lifecycle requirements,
corrects a demonstrated wire-contract violation, and retains both failure and
corrected-case evidence. Retry handling is narrowly tested and bounded.
Ripwire edit-check exited zero; quality-delta exited 2 and test-gate exited 4,
so neither broad static result is claimed as passing. The current source has
not passed complete API-011 or regression acceptance. Native records from
run 34761184178 passed on the preceding gesture-coordinate candidate and will
need refreshing after this source change. Delivery remains blocked on the
external client lifetime problem; an isolated libatspi repair requires an
explicit extension of this commitment under AGENTS.md's scope rule.
