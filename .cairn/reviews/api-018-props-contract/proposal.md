# Proposed Props contract — awaiting developer decision

Keep `#[derive(Props)]`, its builders, and the existing `validate() -> bool`
method. A caller explicitly invokes `validate()`; construction and App mounting
do not implicitly validate arbitrary application data.

- `#[prop(validate = non_blank)]` calls the named function with a shared
  reference to that field. All declared rules must return true. A struct without
  rules has no additional constraints and returns true.
- A bare `#[prop(validate)]` is a compile error with instructions to supply a
  rule. The library does not guess acceptable ages, sizes, names, or other
  application-specific values. Existing bare annotations need a named rule.
- `#[prop(optional)]` requires an explicitly written `Option<T>` field and uses
  `None` as its default. It does not rewrite the field's Rust type.
- Unsupported or malformed attributes fail compilation instead of being
  discarded. Documented defaults and builders keep their stated behavior.

`proposed-predicate.rs` is a concrete example: an empty name must be rejected,
an authored nonblank name accepted, and the optional subtitle may be absent.
It is a proposal, not a claim that the current macro implements this syntax.

The existing macro advertises a bare validation flag but generates a method
that always returns true. Its parser discards errors for unknown attributes.
Its optional-field documentation promises to wrap a plain field in `Option`,
but the implementation assigns `None` without changing that field's type.
The probes in this directory record the current behavior and the working
explicit-`Option` case. Defect observations are not acceptance passes.

Rust derive macros append items beside the original declaration. Attribute
macros can replace the annotated declaration. This is why keeping the derive
syntax requires the author to write the field's type explicitly.
See the [Rust Reference](https://doc.rust-lang.org/stable/reference/procedural-macros.html).

This choice needs approval because API-018 permits a narrower Props contract
only when explicitly approved. The existing API-011/API-018 approval concerns
screen readers; it does not authorize this Props change.

The alternative is to retain implicit type rewriting and define a larger macro
API that can transform field declarations, along with concrete validation rules
for the affected types. That changes how callers declare their props.
