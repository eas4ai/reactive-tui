# App performance ownership checkpoint

The approved migration is implemented. Each App constructs a PerformanceContext
and owns one pending-mode slot. Setters hold weak request ownership and wake that
App for work. The loop renders only when the final coalesced mode differs, so
cleanup/effect requests with no net change do not keep an idle App rendering.
App exit closes requests and its waker, including unwinding. Standalone global
functions remain separate. Existing context fields and component hook signatures
are preserved; external global App callers migrate to App::performance_context.

Contexts are provided after entering the root scope and inherit normal subtree
overrides. Performance hooks allocate consistent fallback/effect slots as
providers appear or disappear. Signals publish completed render cost, presentation
cadence, budget and actual selected mode. The first interval is nominal until two
presentations exist. Fixed modes keep their targets; Auto alone adapts. Separate
signals are not an atomic multi-field transaction. Idle Apps retain their last
completed sample; publication excludes only their own redraw notification and
preserves other Apps' subscriptions.

## Verification and failure demonstrations

- Final focused group: .cairn/reviews/api-residual/20260913T042230684641Z.
  Ten App integration cases, two additional exact unit cases and an independently
  compiled Rust consumer passed; required discovery and execution names are checked.
- Full library: api-019-performance-validation/20260913T042041468078Z/check-1.out:
  1013 passed, two ignored. Strict all-target Clippy, formatting and six guide
  examples passed at api-019-performance-validation/20260913T042230645805Z.
- Five safe mutants at api-019-performance-controls/20260913T041707925032Z:
  shared context, global request queue, unconditional redraw requests, owner
  self-notification and render-cost-as-cadence each failed actual behavior tests.
  Original bytes were restored after each; each corrected case passed.
- The first integration run exposed an invalid test attempt to render hooks after
  their App closed them. The corrected test asserts rejection; a separate live
  scope test covers provider disappearance. Strict Clippy subsequently rejected a
  redundant fixture Default; the corrected sentinel detects missing generated props.
  Both failed development outputs are retained, not counted as passes.
- Independent read-only review found stale idle snapshots, repeated-request
  feedback and incorrect cadence, all corrected and covered. A second review
  requested the same-mode guard at render entry and publication after Auto startup
  benchmarking; both are applied. No unresolved production finding was reported.

Native local/performance hook execution is added to the existing authorized
macOS/Windows workflow, with committed input blob identities, source stability,
exact test names and output hashes. That new native run has NOT yet occurred.
All inherited evidence must be refreshed after this commit. This is an API-019
checkpoint, not completion of API-019 or the commitment.

## Static review

Focused Ripwire edit checks returned zero. quality-delta returned 2 and test-gate
returned 4; these are nonpasses. The broad scan includes reference repositories
and reports many new/unresolved graph entries. Focused dead-code reports include
executed tests, dynamic Python group dispatch, generated components and the called
Owner::context. Small close-method similarities describe different owner state,
not interchangeable helpers. The set/update similarity existed before this change;
the existing set body moved behind a private notification parameter without adding
another update implementation. Checker growth is the explicit performance group;
its required names and negative controls justify those steps. No quality gate is
represented as passing.

## Production coding self-audit for this checkpoint

1. Scope and current paths were read before changes; explicit approval is recorded.
2. Changes address owned performance behavior and its proof; no dependency added.
3. Ownership is private and named; no new global App state or unsafe code.
4. Public provider literals remain valid; approved global migration is documented.
5. Existing result/error propagation remains; process checks preserve failures.
6. No secrets or external user data are involved; outputs and source identity are checked.
7. Pending requests coalesce and close; neither persistence nor schema changes apply.
8. Queue memory is constant, weak callbacks do not retain App, and idle feedback is tested.
9. API-019 remains the single in-progress todo; API-020 is pending.
10. Focused behavior, violating cases, library, lint, format and guide checks ran.
11. Native and inherited refresh remain explicitly pending; no full acceptance claim.
12. The developer approved the concrete compatibility migration before implementation.
13. This checkpoint is reviewable; final delivery waits for native and full closure evidence.
14. Guide and records explain who owns requests, what callers change, and observable limits.
