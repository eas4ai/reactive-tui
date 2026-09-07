# Shared App wakeups

Status: Agreed 2026-09-07
Prefix: WAK

The developer approved shared App wakeups as the next cleanup commitment.
Signals, scheduled work and embedded terminal output must reach App without
waiting for a periodic frame poll. Existing renderer and native interpreter
choices remain in place. The acceptance environment is the Unix PTY and
controlled backend tests; other hosts and platforms are not implied.

[WAK-001]
App MUST provide a thread-safe wake handle that interrupts its supported backend wait for redraw, scheduled work or stop.
Notifications MUST coalesce with bounded notification storage and MUST NOT be lost when sent before or during entry to the wait.
Falsifier: App sleeps through a pending notification, a notification burst creates an unbounded queue, or a stop request requires input.
Mechanism: scripts/check-app-wakeups.sh, controlled wait/race and lifecycle tests.

[WAK-002]
Signals read while App renders MUST request a redraw when their value changes, including ThreadSafeSignal writes from another thread.
Equal writes MUST NOT request redraw, and subscriptions MUST NOT keep a stopped App alive or wake an unrelated App.
Falsifier: a changed rendered signal stays stale without input, equal writes trigger redraw, or a retained signal wakes the wrong or stopped application.
Mechanism: scripts/check-app-wakeups.sh, signal-to-App tests.

[WAK-003]
The App scheduler MUST wake App when work is queued or its timer deadlines change, and App MUST service due timers while idle.
Timer callbacks MUST be able to schedule or cancel timers without holding the timer-store lock.
Falsifier: queued work or a newly earlier timer waits for input, a callback deadlocks on scheduling, or cancelled timers execute again.
Mechanism: scripts/check-app-wakeups.sh, scheduler and deadline tests.

[WAK-004]
A wake-driven root on a wake-capable backend MUST sleep when idle without periodic frame polling or redraw.
App MUST coalesce redraw requests within its frame budget while retaining the latest state and keeping input, resize, timers, animations and stop responsive.
Existing polling roots and backends MUST retain a documented compatibility path.
Falsifier: idle wake-driven App repeatedly polls, a burst loses its final state or bypasses frame pacing, or existing polling roots stop updating.
Mechanism: scripts/check-app-wakeups.sh, instrumented App and inherited gates.

[WAK-005]
Embedded terminal frame, exit and error publication MUST wake App through the shared mechanism.
All existing renderer and embedded-terminal requirements MUST retain current passing evidence.
Falsifier: delayed terminal output, child exit or a session error remains unobserved until host input, or an inherited requirement fails.
Mechanism: scripts/check-app-wakeups.sh and inherited real-PTY gates.
