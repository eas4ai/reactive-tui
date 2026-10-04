# Reactive TUI read-only code review

## Scope and constraints

Requested: map all Rust code; review correctness, complexity, duplication, unnecessary abstraction, unsupported claims, and weak evidence. Source is frozen outside the repository to avoid racing the active implementation agent. No production edits, tests, Clippy, builds, rust-analyzer, or Sudus state changes. Only `.review/` contains review artifacts.

“Contradicted claim” means source evidence conflicts with stated behavior. This review cannot infer an author's intent or certify that every possible defect has been found.

## Todo

- [done] Freeze inputs, inventory the entire Rust codebase, and assign review coverage.
- [done] Review production subsystems; inspect tests/documentation selectively as supporting source.
- [in progress] Validate findings across callers, distinguish confirmed defects from risks, and deduplicate.
- [pending] Publish prioritized findings, claims audit, coverage limits, and source drift report.

## Verification policy

Static source analysis only. No tests or Clippy will run. Reproduction examples are reasoned traces unless explicitly marked otherwise. Existing test assertions are evidence about coverage, not proof those tests pass.
