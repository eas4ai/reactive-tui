# Performance ownership: proposed compatibility choice

Status: diagnosis and proposal only; no performance implementation change.

## Observed failure

The external consumer `verification/api-residual/performance-owner.rs` runs two
real App/SuprTUI instances with a private output sink and stops each from its
first render. The first selects PowerSave; the second selects Gaming. The first
App's retained use_fps signal changes from target_fps 30 to 144 when the second
runs. Both report mode Auto. Calling the escaped first-App mode setter puts
Balanced into the shared process request queue even though that App has exited.
The independence assertion fails. Exact commands and raw output are retained in
`20260913T021322293642Z/`. An earlier fixture compile error is preserved separately;
it is not library evidence. The failing runtime assertion is diagnostic, not a pass.

`src/app.rs::publish_performance_context` reads or writes one global context and
always publishes Auto. Both render paths drain one global requested-mode slot.
`src/hooks/perf_context.rs` exports set/get global context and request/take mode
functions with no App identity. `src/hooks/fps.rs` reads an inherited context first,
then this global context. Existing public PerformanceContext has shared metrics
signals and a set_mode closure, so it can already serve as the explicit handle.

## Proposed API and behavior

Add `App::performance_context(&self) -> Arc<PerformanceContext>`. Each App owns its
context and pending-mode state from construction until cleanup. Component hooks
receive that context through the existing scoped inheritance; ordinary use_fps,
use_performance, use_frame_timing and use_performance_mode call forms remain.
Parent-provided overrides retain precedence and stable hook slots must not change
when an optional context appears or disappears. Report the actual selected mode
and timing budget. Each set_mode closure targets only its own App, requests a
wakeup, and cannot schedule work after that App closes.

External callers obtain the handle before App.run consumes the App. They read its
signals or call its set_mode closure from any supported thread. No new required
field is added to the public PerformanceContext struct, preserving existing
explicit providers and struct literals.

Keep the named global setter/getter and request/take functions as an explicit
standalone compatibility facility. Standalone hooks can still use that facility.
Apps will neither publish into it nor drain its queue. Existing external code
that observes or controls an App through those global functions must migrate to
that App's handle or its component hooks. This caller-visible change needs a
separate decision; local-reference scope approval does not authorize it.

## Why a choice is required

A no-argument global getter and a mode request without an owner cannot identify
which of two Apps a caller intends. Retaining last-publisher/first-consumer routing
keeps the demonstrated cross-App behavior. Thread-local globals also fail for
nested Apps or callers on worker threads. Silently choosing the current/last App
would hide the ambiguity rather than repair ownership.

The alternative is to preserve process-global routing and explicitly limit the
performance family to one App per process. That narrows the committed multi-App
isolation contract. Explicit App handles are recommended because they preserve
multi-App operation and ordinary component-hook usage.

## Required proof after approval

Use independent App modes and retained handles across sequential, concurrent and
nested runs; verify no metrics/request crossover, wake delivery, correct selected
mode, provider inheritance/overrides, stable hook slots, and closed-owner behavior.
Exercise standalone global APIs independently and document external migration.
Demonstrate failures with a shared-context/queue mutant and corrected cases.
