# Local reference scopes

`use_local_ref` retains arbitrary non-Send values, such as `Rc<Cell<T>>`, across
renders without triggering redraws. `Hooks` remains Send + Sync; returned local
handles stay on their creating thread.

`App::run` supplies the scope automatically. For manual hook or generated-component
renders, wrap the full sequence once:

```rust
use reactive_tui::hooks::{use_local_ref, with_local_hooks};
use reactive_tui::reactive::Hooks;
use std::{cell::Cell, rc::Rc};

with_local_hooks(|| {
    let hooks = Hooks::new();
    let first = use_local_ref(&hooks, Rc::new(Cell::new(1)));
    first.current().set(7);
    drop(first);
    hooks.reset();
    assert_eq!(use_local_ref(&hooks, Rc::new(Cell::new(99))).current().get(), 7);
});
```

This is the approved API-019 migration: manual calls now require the wrapper.
The `use_local_ref` signature and supported value types remain the same. An owner
cannot use local slots on another thread, in a different scope, or after its scope
ends; those calls panic. Generated renders also check hook kind, value type and
count. Constructing an unrendered component does not bind it to a scope.

Nested explicit wrappers create independent scopes. `App::run` reuses an entered
scope, so manual rendering followed by App.run can share one wrapper. App cleanup
releases only its component owners and leaves unrelated owners in that scope alive.

Same-thread cleanup or final owner drop releases retained shares immediately.
Foreign-thread cleanup defers local destruction until the creating thread next
uses a local scope or exits it. Scope exit releases retained shares even while
unwinding; there is no promise that an idle creating thread will run deferred
cleanup before its scope ends. Cleanup racing with a sweep can wait for a later
sweep or scope exit. Once scope exit begins, destructors cannot reenter local hooks
in that closing scope. Escaped LocalRef handles retain their own shares.

The local-reference checks cover retention, invalid scopes, thread confinement,
cleanup, App reuse, component removal, errors and unwinding. Full API-019 acceptance
still depends on the remaining [residual audit concerns](residual-api-inventory.md).
