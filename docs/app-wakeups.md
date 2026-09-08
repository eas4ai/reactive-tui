# Shared App wakeups

App can sleep until input, a state change, a timer deadline or a stop request
arrives. The renderer and Ghostty interpreter remain unchanged by this API.

## Signal-driven roots

Signals read during rendering subscribe that App to value changes.
`ThreadSafeSignal` supports changes from another thread; local `Signal`
retains its existing thread restrictions. Equal writes do not notify.
The subscriptions are weak and refreshed on each render, so unused signals
stop requesting redraw and retained signals do not keep App alive.

Roots opt into idle waiting with `fn wake_driven(&self) -> bool { true }`.
Their background producers must use observed signals, the App scheduler, or
the attached wake handle. Existing roots default to periodic `update()` calls.
A wake-driven root still receives `update()` after events and notifications.

Run the signal-driven example:

```sh
cargo run --locked --example wake_counter
```

It advances through an App timer. Escape or Ctrl+C exits.

## Explicit background work

Before moving an App into `run()`, clone `app.waker()` and `app.scheduler()`.
The wake handle is thread-safe:

- `request_redraw()` marks the latest state dirty and wakes App.
- `wake()` asks App to inspect background state or changed deadlines.
- `request_stop()` wakes App and requests graceful exit.

Custom roots can implement `attach_waker()` to pass the handle to an owned
producer. `TerminalView` uses this to observe frame, exit and error publication.
It renders the final exit snapshot before leaving App. When its bounded
command queue fills, TerminalView retains one pending key and pauses host
input with `accepts_input()`. The worker wakes App after consuming commands,
allowing the key to be retried without loss. Timers and explicit stop requests
remain active during that pause.

Notifications occupy fixed pending flags, not a queue of frames. Many requests
can combine into one redraw. App keeps dirty state until the frame budget
allows presentation; new notifications during presentation survive for the
next iteration. Input and timer work remain serviceable between frames.

Handles become inert when App exits or is dropped. App clears its scheduled
work and timers on drop. A callback already running must return before App
can stop; arbitrary user callbacks cannot be preempted.

## Scheduler and deadlines

`app.scheduler()` returns the App-owned `Arc<Scheduler>`. Call
`schedule_update`, `schedule_timeout` or `schedule_interval` from a producer.
App runs queued work and due callbacks on its own thread. Earlier deadlines
wake its wait immediately. Callbacks run outside scheduler storage locks and
may enqueue work or schedule/cancel timers. Cancelling a running interval
allows that invocation to finish but prevents its next repetition.

`AppBuilder::scheduler` accepts an existing shared scheduler. One scheduler
can belong to only one live App. Queued tasks remain distinct; only their wake
notifications coalesce. The work queue is not given a new bounded-capacity
contract by this change. Zero-duration intervals remain eligible every turn
and intentionally prevent idle waiting. Delays outside the clock's range
panic as invalid scheduling requests.

Use this App scheduler for wake-aware work. Existing standalone runtime and
hook timer schedulers retain their existing lifecycle; this commitment does
not merge those systems or repair their separate effect-lifecycle behavior.

## Backend compatibility

`Backend::poll_event_with_wake` adds wake-aware waiting without requiring
existing backends to implement a new method. Its default checks legacy input
nonblockingly at intervals of at most 10 ms, sleeping on a condition variable
between checks. Legacy backends must honor their existing zero-timeout poll
contract. This fallback does not claim zero periodic input polling.

`SuprTuiBackend` overrides the method with Crossterm's EventStream and a
standard task waker. It waits for input, resize or App notification without
periodic frame polling. No Tokio executor is required by this wait adapter.
Do not concurrently read host input with a separate Crossterm reader.
Crossterm owns and signals cancellation to its internal input helper thread;
Reactive-TUI does not claim to join that upstream helper synchronously.

Existing polling roots and active animations retain timed updates. The App
animation manager and hook-animation start notifications keep animation work
moving; this does not change interpolation or completion semantics.

## Verification and limits

```sh
sh scripts/check-app-wakeups.sh
```

The checks exercise notifications around wait entry, signal subscriptions,
equal writes, idle waits, coalesced frames, queued work, earlier timers,
reentrant cancellation, input and error cleanup. A controlling-PTY probe
changes a signal from a background thread while App has no timer deadline,
then checks idle output, resize, quit and exact termios restoration. Inherited
renderer and embedded-session gates remain required.

The design borrows WezTerm's wake-and-invalidate pattern. No WezTerm crate or
source was copied into this implementation. It retains the existing host
input decoder and adds its event-stream feature plus the futures-core trait.
Direct developer-host compatibility and the reported counter-input failure
remain separate from these deterministic probes.
