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

The consumer checks raw and parsed receiver removal while idle, raw receiver
removal followed by input, and reuse of a released terminal descriptor by a private
socket. It requires owned workers to stop. It never reads another application's
descriptor or sends input to the developer's terminal. The descriptor-reuse case
is a race: a passing run does not establish safe ownership. Its setup checks that
the replacement socket actually reused the descriptor.

Every case asserts correct behavior. The command exits nonzero while the defects
remain; a failed diagnostic is not an acceptance pass. Timestamped command records,
per-case outputs, exit statuses and reap results are retained under
`.cairn/reviews/api-019-input-lifecycle/`. The build driver's `exit` values normalize
success/failure to 0/1; `cases/results.json` retains actual child exit statuses.

These probes do not yet cover bounded queue growth, every session-removal order,
multiple readers, macOS, signal handlers, or `ThreadedEventLoop`. Their acceptance
integration and corrected-behavior checks remain API-019 work.
