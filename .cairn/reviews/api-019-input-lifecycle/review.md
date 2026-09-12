# API-019: Unix input receiver lifetime and compatibility

Status: defects reproduced; proposed compatibility change awaits the developer.
Production baseline: `ef4ce3affd9916bfc2dd7332bb7f97ccdf3ed4d8`.

## Existing contract and observed behavior

The commitment review at line 4155 already requires bounded retained input work,
receiver/session cleanup while idle and under input, and no access to reused
descriptors. The condensed inventory now repeats receiver removal explicitly and
names both raw and parsed input. This preserves that earlier contract.

`UnixTty::spawn_input_thread` returns the concrete standard-library
`Receiver<Vec<u8>>`. Its detached worker copies the owner's descriptor number,
uses an unbounded channel, and notices receiver removal only when sending input
fails. Its idle timeout continues the loop. Dropping the terminal owner can close
the descriptor while the worker still reads it or restores its flags.

`DirectTty::start_async_events` returns `Receiver<TerminalEvent>`. It adds another
unbounded channel and detached parser worker. The parser waits on the raw receiver;
a failed output send leaves only the inner event loop, not the worker loop.

The external consumer compiles with explicit annotations for both existing return
types. Each case gets its own controlling PTY. It counts new Linux thread IDs,
checks a successful raw-input exchange, and asserts that owned workers disappear.
The descriptor attack uses a private socket and verifies exact descriptor reuse.
It does not access another process's descriptors.

| Diagnostic run | Raw receiver dropped idle | Parsed receiver dropped idle | Descriptor reused | Raw drop followed by input |
| --- | --- | --- | --- | --- |
| `20260912T211735870056Z` | Fail: reader remains | Fail: both readers remain | Pass in this run | Pass: reader exits |
| `20260912T212437894022Z` | Fail: reader remains | Fail: both readers remain | Fail: socket bytes consumed | Pass: reader exits |
| `20260912T212803395506Z` | Fail: reader remains | Fail: both readers remain | Fail: socket bytes consumed | Pass: reader exits |

Every library build and external-consumer compilation passed. Every complete
lifecycle run exited nonzero. Failed Rust assertions exited 101; no case reached
its outer ten-second deadline, and every child was reaped. The leaked threads end
when their isolated test process exits. Both failing reuse cases received the exact
private marker `PRIVATE_REPLACEMENT_INPUT` through the old TTY receiver. This is a
race; the first passing run is not proof of safe descriptor ownership.

The first two runs used a temporary build driver. The third used the checked-in
`verification/api-residual/run-input-lifecycle.py`, which reuses the existing
bounded process executor. Its command records normalize command failure to 1;
the per-case JSON retains the actual child statuses.

## Concrete proposal, not yet implemented

Change these two Unix-only return types to an owned receiver:

```rust,ignore
// Proposed signatures, not declarations in the current library:
UnixTty::spawn_input_thread(&self) -> Result<InputReceiver<Vec<u8>>>
DirectTty::start_async_events(&self) -> Result<InputReceiver<TerminalEvent>>
```

`platform::InputReceiver<T>` would own cancellation and worker joins. Dropping it
or the final terminal/session owner would stop the associated raw/parser work,
including when idle, before releasing descriptors. Queues would have a fixed bound
and cancellation would interrupt queue waits as well as input waits. Retain real
bytes/events, stable receiving methods (`recv`, `try_recv`, `recv_timeout`, `iter`,
`try_iter`, and owned/borrowed iteration) and the corresponding standard error
types. Do not add synthetic wake events or an arbitrary idle expiry.

The wrapper is not interchangeable with the old concrete receiver type. An
annotated caller would migrate as follows:

```rust,ignore
// Before:
let input: std::sync::mpsc::Receiver<Vec<u8>> = tty.spawn_input_thread()?;
// Proposed after:
let input: reactive_tui::platform::InputReceiver<Vec<u8>> = tty.spawn_input_thread()?;
// Inferred receiver use keeps its ordinary form:
let input = tty.spawn_input_thread()?;
let bytes = input.recv()?;
```

Function parameters, stored receiver types, and explicitly named standard iterator
types would also need migration. Exact compatibility with every generic/trait use
has not been established. Examples above illustrate the decision, not a completed
or compiling wrapper implementation. The C ABI and Windows-only receiver APIs are
outside this proposed signature change.

Pinned Rust 1.95.0's `Sender` exposes `send`; `SyncSender` exposes `send` and
`try_send`. Neither provides an idle receiver-disconnection notification/query.
Changing to `sync_channel` can bound memory while retaining the old receiver type,
but it cannot provide a hook when the consumer drops that type. Sending dummy
bytes/events to detect disconnection would change the stream. Installed Rust
source and documentation were inspected; their paths, hashes and method anchors
are retained in `verification/rust-channel-docs.json`, alongside the compiler
version. The current official [SyncSender documentation](https://doc.rust-lang.org/std/sync/mpsc/struct.SyncSender.html)
also describes send-based disconnection detection; it is not the pinned-version
evidence.

The alternative is to retain the exact old signatures, explicitly approve a
session-owned legacy lifetime, and add an owned-receiver route. An idle legacy
worker could then remain after receiver removal until terminal/session teardown.
Descriptor ownership, bounded queues and teardown still require repair. This
alternative weakens the recorded receiver-drop guarantee and adds two lifetime
contracts to maintain. The owned receiver is the recommendation.

This requires a developer decision because the commitment preserves working Rust
APIs and forbids silent migration or contract narrowing. The earlier Props approval
does not authorize this change. No input signature, production worker, architecture
decision or scope exception has been changed here.

## Diagnostic ownership and verification

The controller uses Linux `PR_SET_PDEATHSIG` before starting a new session, then
checks its parent identity to cover termination during setup. Normal and timeout
paths kill only its private process group as needed and wait for the direct child.
The cancellation control becomes a subreaper, terminates the controller, and waits
for the adopted probe to die by SIGKILL. Its finally block cleans up owned processes
on an assertion failure too.

The checked-in cancellation control passed. A temporary copy of the controller
with only `prctl(1, signal.SIGKILL, 0, 0, 0)` changed to
`prctl(1, 0, 0, 0, 0)` failed the required SIGKILL assertion. Its eventual SIGHUP
does not count as the required parent-death behavior. The control reaped that child
as well; the temporary mutant was removed and production source was untouched.
Current positive/negative outputs and exact commands are in `verification/`;
the earlier control captures are preserved separately.

Rustfmt passed and all three Python sources parsed. The five residual checker
controls passed, including rejection of incomplete inventories and nested failures.
Ripwire's three edit checks exited 0 with no incompatible callers. Its common-name
matching labels the new `main` and `run` symbols as contract changes against unrelated
baseline names; no existing symbol was edited. Quality-delta exited 2, and test-gate
exited 4: neither is a passing gate. The quality output is dominated by unchanged
reference trees. The two diagnostic findings are the `private_session` callback
classified as dead (it actually runs via `Popen.preexec_fn`) and shared polling
structure in the separate startup/shutdown assertions. Keeping those two opposite
assertions separate makes the lifecycle checks explicit. The test gate identifies
four documentation symbols and no modeled tests; actual execution is recorded
above. Full outputs and a focused quality summary remain in `verification/`.

## Limits and production-standard review

Reviewed this diagnostic change against all 14 production rules. The added files
are external probes, bounded controllers, and evidence; they preserve the production
API and record observed failures plainly. Unsafe Rust is limited to adopting the
private descriptor explicitly passed by the controller. No new library dependency,
global application state, user data, or unrelated process is involved. Build/test
concurrency remains capped at 12. Caller discovery tried the graph first (transport
closed), then Tilth and Ripwire; external users are not inferable from local counts.

These probes are Linux diagnostics, not native-platform acceptance. Bounded queue
growth, session-drop permutations, multiple readers, macOS, signals, the separate
`ThreadedEventLoop`, and corrected receiver behavior remain unverified. The new
probes must be integrated into API-019's complete mechanism after repair. The formal
API-019 baseline `20260912T211515640Z` remains failed for reference defects, absent
backend test discovery and incomplete coverage. No correctness requirement is
closed by this diagnostic commit, and API-020's image-capture cancellation defect
is separate and still open. This is a reviewable decision checkpoint, not delivery
of a completed production library or the final commitment self-audit.
