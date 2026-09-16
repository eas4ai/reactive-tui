# Recovery roadmap

Status: Agreed 2026-09-07
Current: widget-catalog

1. `suprtui-renderer` — the first screen: styled frames, updates, Unicode,
   resize, output errors, input, and terminal restoration (RND-001 through RND-006).

2. `embedded-terminal` — a real shell PTY interpreted by libghostty-rs and
   displayed through App/SuprTUI, with keyboard input, resize, and cleanup.

The first renderer commitment is complete; its requirements remain inherited.
Image protocols, clipboard, and remaining legacy defects were deferred until
the Rust API remediation commitment below.

3. `app-wakeups` — shared wake notifications connect signals, scheduled work
   and terminal output to App, with idle waiting and bounded redraw bursts.

4. `registry-concurrency` — remove the concurrent registry metrics/cleanup
   stall and verify lifecycle reentry and concurrent progress.

5. `registry-cache-isolation` — isolate named lookup between registries, make
   clones observe shared registrations, and refresh the full-suite assessment.

6. `default-suite-repair` — repair numeric inset layering, step-easing test
   semantics and accordion doctests; make the default full suite pass.

7. `ffi-lint-format-repair` — repair FFI compile/link integration, strict
   default-feature Clippy failures and workspace formatting debt.

8. `binding-abi-compatibility` — align C headers and TypeScript bindings with
   Rust exports and verify consumer interoperability.

9. `rust-api-remediation` — remediate all 15 Rust API audit findings and named
   residual concerns, restore end-to-end native behavior and bindings, and retain
   all inherited acceptance contracts (API-001 through API-020).

10. `documentation-retention` — retain only Cairn-managed specifications,
    commitments, and decisions in the tracked documentation tree (DOC-001).

11. `framework-manual` — publish a source-grounded manual with a linked
    overview and focused pages for every public framework system (MAN-001 and
    MAN-002).

12. `crates-io-release-preparation` — completed the provisional bounded,
    registry-resolvable 0.1.0 package foundation (CRT-001 through CRT-003).
    Final packages are rebuilt from the remediated tree in commitment 20.

The Fable audit findings are mapped exactly once in
`docs/spec/pre-release-audit-map.md`. Remediation work comes before final
packaging.

13. `pre-release-ffi-safety` — repair C ABI lifetime, nullability, panic,
    ownership, concurrency, buffer, and declaration defects (FFS-001 through
    FFS-005).

14. `pre-release-terminal-lifecycle` — restore the host terminal after faults
    and signals, filter control data, and bound terminal helpers (TRL-001
    through TRL-004).

15. `pre-release-reactive-concurrency` — give hooks stable cancellable owners,
    release internal locks before caller code, and bound fallback timers
    (RAC-001 through RAC-003).

16. `pre-release-runtime-resilience` — make accessibility, public event loops,
    adaptive display settings, and grid inputs degrade or fail without hangs
    and panics (RTR-001 through RTR-004).

17. `pre-release-external-input-safety` — make dialog network access explicit,
    bound complete image memory, protect renderer arguments, and close explorer
    identity races (XIS-001 through XIS-003).

18. `pre-release-dependency-code-quality` — repair and enforce the dependency
    graph, route diagnostics through logging, remove dead surfaces, strengthen
    tests, and document public items (DQC-001 through DQC-005).

19. `example-cleanup` — remove the six repository examples the developer
    found broken in Kitty, retain the working gradient example, and remove
    stale invitations to run the deleted programs (EXC-001).

20. `widget-catalog` — build one responsive catalog for the public widget
    families, the project logo, and a spinning wireframe cube, with real
    navigation, animation, resize, and quit checks (CAT-001 through CAT-003).

21. `pre-release-ci-documentation` — run supported-platform CI, restore tracked
    documentation inputs, rerun every mechanism, and clean stale repository
    rules (RID-001 through RID-003).

22. `final-release-packaging` — rebuild final registry packages and release
    metadata, then require fresh cross-platform evidence and an adversarial
    audit before a release decision (FRP-001 through FRP-004).

## Queued next iteration

`wgpu-graphics` — optional offscreen graphics presented inside the terminal,
a viewport-sized shaded spinning cube, CPU fallback, bounded lifecycle, and
render/readback measurements (GPU-001 through GPU-005). Draft contract:
[specification](wgpu-graphics.md) and
[commitment](../commitments/wgpu-graphics.md).

This queue entry does not activate implementation or change `Current:`.
The catalog's open visual findings and the existing release sequence remain.
