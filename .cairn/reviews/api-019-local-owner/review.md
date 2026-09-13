# Local reference ownership: proposed compatibility choice

Status: proposal only; no local-reference or App implementation change.

## Observed contract conflict

`use_local_ref(&Hooks, T)` currently constructs a new `Rc<RefCell<T>>` on every
call and does not retain it in Hooks (`src/hooks/refs.rs`). The original retention
test still fails. The shared, callback and multi-reference repairs now pass their
tests, including the existing generic bounds, without changing LocalRef.

The independent consumer in `verification/api-residual/local-owner.rs` compiled
against the actual library. It verifies `Hooks: Send + Sync`, accepts an arbitrary
`Rc` local value, and moves/drops Hooks on another thread. Its lifetime assertion
fails: the value is already destroyed when the returned LocalRef is dropped,
before the Hooks owner disappears. Compiling its `send_local` variant fails with
E0277, correctly rejecting transfer of `Rc<RefCell<Rc<i32>>>` to another thread.
Exact commands and outputs are in `20260913T002934478844Z/`. The lifetime failure
is not an acceptance pass; the confinement compile failure is the required control.

Retained local values cannot be placed in the existing thread-safe slot storage
(`src/reactive/hooks.rs`, `Box<dyn Any + Send + Sync>`). A concrete conflict is:

1. Thread A creates Hooks and a local Rc value, then drops all returned handles.
2. Persistence requires the library to keep that value for A's next render.
3. A moves the sole Hooks owner to B and remains alive but idle.
4. B drops Hooks. B cannot safely destroy A's arbitrary non-Send local value.

Putting the value in Hooks violates its Send contract. A weak thread-local entry
loses persistence; a strong entry without an explicit owner survives indefinitely
while A is idle. Queueing cleanup does not execute it on an idle thread. Existing
component TLS is only a temporary context stack, not an executor that can force A
to perform cleanup. An unsafe Send assertion does not resolve this conflict.

The existing owned-hook decision requires standalone cleanup on cleanup/final
drop. A local owner therefore introduces a real, visible lifetime distinction;
it must be selected explicitly. The approved Unix receiver migration does not
authorize a local-hook precondition or deferred local cleanup.

## Recommended API and usage

Add the scoped entry point `reactive_tui::hooks::with_local_hooks`:

```rust,ignore
// Proposed API, not yet implemented:
pub fn with_local_hooks<R>(body: impl FnOnce() -> R) -> R;
```

Standalone hook callers wrap the complete sequence of related renders once:

```rust,ignore
with_local_hooks(|| {
    let hooks = Hooks::new();
    let first = use_local_ref(&hooks, Rc::new(Cell::new(1)));
    first.current().set(7);
    drop(first);

    hooks.reset();
    let second = use_local_ref(&hooks, Rc::new(Cell::new(99)));
    assert_eq!(second.current().get(), 7);
});
```

The scope owns a thread-confined local slot arena for the synchronous closure.
Hooks retains only thread-safe slot metadata, an owner identity and a lifecycle
token. Arena entries are separated by Hooks identity and positional slot, with
existing kind/type/count checks. Local values remain arbitrary non-Send types.
TLS contains only temporary bindings to the explicitly owned arena, and no TLS
borrow is held across user code or value destruction.

Keep the `use_local_ref` signature. Calls require an entered scope and the same
live owner/thread on subsequent renders. Missing scope, another arena/thread, or
reuse after its scope ends produces an explicit programmer error. Dropping the
scope releases its retained shares on the creating thread, including on unwinding.
Escaped LocalRef handles keep owning their values normally and remain non-Send.

Same-thread Hooks cleanup removes that owner's arena entries. Foreign-thread
cleanup invalidates metadata; the creating thread reclaims the local entries at
its next scope sweep or, unconditionally, when the scope ends. This is deferred
cleanup, not a promise of immediate destruction on foreign-thread Hooks drop.

## Automatic App support and compatibility cost

`App::run` consumes App and runs a synchronous loop (`src/app.rs:110,137`). It can
execute its existing body as a move closure owning `self`: reuse an entered local
scope, or create one if absent. App destruction then occurs inside that scope on
return, error and unwinding. Merely creating a guard before the existing body is
insufficient if that guard drops before the function's `self` parameter.

App construction does not render (`AppBuilder::build`, `src/app.rs:793`). The root
and keyed children already enter resource scopes for rendering and removal
(`src/component/runtime.rs:88,115,229`). Cleanup must remove only that Hooks owner's
local entries, not a borrowed arena shared with another App or the enclosing
caller. No Rc arena field is added to App, Hooks, Component, or generated state.

Ordinary local-hook calls during App.run get automatic support. Manual hook calls,
manual component construction/rendering that uses local hooks, and standalone
Hooks sequences require the explicit wrapper. An already entered scope can span
manual rendering followed by App.run. Existing component-resource owner binding
restrictions still apply. Local state cannot migrate between threads or into a
new, unrelated scope; moving an unrendered App/component remains possible wherever
its existing types allow it. Existing Send callbacks still cannot capture LocalRef.

The alternative is to restrict hook values to Send, retain them in synchronized
storage, and preserve non-Send handles separately. Hooks can then reclaim values
on any thread without a local scope. Existing Rc-based local-hook callers must
change their value types, and the advertised arbitrary non-Send value support
would narrow. The explicit scope is recommended because it preserves that use.

## Work required after approval

Record the selected contract and ownership decision before building it. Verify
persistence after every returned handle is dropped, independent slots/owners,
kind/type/count failures, missing/wrong/expired scopes, same-thread and foreign
cleanup with drop counters, and unconditional scope exit without later library
calls. Retain static Hooks/Component thread-safety and LocalRef confinement checks.
Exercise real generated components and App.run, including nested/manual scopes,
removal, errors and unwinding. Existing failing local behavior must become a real
correctness pass; it cannot be relabeled or excluded from complete acceptance.
