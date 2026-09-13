# Residual API diagnostics

The complete acceptance command is `cairn check API-019`, on committed inputs.
These focused diagnostics do not replace it or establish complete API-019 coverage.

## Unix input lifetime

On Linux, from the repository root:

```sh
python3 -B verification/api-residual/run-input-lifecycle.py
python3 -B verification/api-residual/check-controller-cleanup.py
```

The first command builds the actual library with the locked dependencies and a
12-job limit, then compiles an external Rust consumer. It overrides an inherited
Cargo target directory with this repository's `target`. Each case runs in a
disposable controlling PTY, with a ten-second process deadline. The child belongs
to the controller even after creating its private session: Linux's parent-death
signal kills it if the controller exits. Normal and deadline paths wait for it.
The second command verifies controller termination and reaps the adopted probe.
It requires the binary produced by the first command.

The consumer checks raw and parsed receiver removal while idle and under full-queue
backpressure, final session removal, clone lifetime, independent streams, owned
iterator removal, concurrent teardown, and reuse of a released terminal descriptor
by a private socket. It requires owned workers to stop. It never reads another application's
descriptor or sends input to the developer's terminal. The descriptor-reuse case
is a race: a passing run does not establish safe ownership. Its setup checks that
the replacement socket actually reused the descriptor.

Every case asserts correct behavior. The command exits nonzero while the defects
remain; a failed diagnostic is not an acceptance pass. Timestamped command records,
per-case outputs, exit statuses and reap results are retained under
`.cairn/reviews/api-019-input-lifecycle/`. The build driver's `exit` values normalize
success/failure to 0/1; `cases/results.json` retains actual child exit statuses.

All 14 cases are included in `scripts/check-api-residual.py --only unix-input`,
alongside exact discovery/execution of five receiver unit tests and the controller
cancellation check. Missing cases, timeout or missing reap results fail that group.
macOS, signal handlers and `ThreadedEventLoop` still require separate evidence.
The full API-019 coverage gate remains incomplete and cannot pass this group alone.

## Local reference ownership

`local-owner.rs` is an external consumer of the unchanged local-reference API.
Its ordinary build asserts retained value lifetime after returned-handle removal
and currently fails. Building with `--cfg send_local` must fail compilation because
LocalRef cannot cross threads. Exact compiler/run commands and the ownership
proposal are recorded in `.cairn/reviews/api-019-local-owner/`. Neither a new local
scope API nor the proposed App integration has been implemented.
