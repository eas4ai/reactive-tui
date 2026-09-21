# Require named caller-invoked Props validation and explicit optional field types

Level: Consequential
Decided by: Developer
Rests on: API-018, API-020
Would be wrong if: A declared rule is ignored, optional fields require type rewriting, or documentation promises validation before the caller invokes validate().
History: The API audit required enforcement or explicit approval of a narrower Props contract. The developer approved escalation api-018-api-020 on 2026-09-12 after reviewing compiler probes.

## Decision

Keep derive(Props), fluent builders and the inherent validate() -> bool method. Validation runs only when the caller invokes it. Each #[prop(validate = rule)] names a predicate called with a shared reference to that field; validate succeeds only when all declared predicates return true. Fields without rules add no constraints. Reject bare, malformed, duplicate or unsupported prop options with an actionable compiler error. #[prop(optional)] requires an explicit Option<T> field and defaults to None; derives cannot rewrite the input struct. Preserve documented string defaults and default constructors, and migrate existing bare validation annotations to named rules. This approval does not add automatic construction or App validation. The reviewed proposal and failing/corrected-shape compiler probes are retained under .cairn/reviews/api-018-props-contract.

## Realized by

- 98fce395f0024a97adbe54d311a71b794b787fd6 Enforce approved named Props rules and explicit optional fields

Behavior and diagnostics: `tests/api_props_contract.rs`, `tests/props_derive_test.rs`,
macro option unit tests and `src/component/props.rs` rustdoc cases.
Review: `.cairn/reviews/rust-api-remediation.md`, API-018 approved Props implementation.
