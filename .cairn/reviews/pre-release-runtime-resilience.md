# Review: pre-release-runtime-resilience

commit: 275c8d20b9bea775475c4159ea247e418866d6ac
findings:
  - unresolved: RTR-003 clears its running flag before the synchronously aborted task is observed as dropped, so the sync-stop case can pass without proving task termination.

## Scope examined

Re-read RTR-001 through RTR-004, the four mechanism declarations, their
validators and public-API fixtures, the latest committed receipts, and the
runtime code changed for accessibility selection, threaded and Tokio event
loops, adaptive frame timing, and automatic grid placement. Compared the
corrected behavior with Fable findings M1, M3, M4, M7, and L1.

RTR-001 distinguishes automatic from explicit accessibility selection and
exercises missing and stalled session-bus endpoints through an interactive PTY.
RTR-002 covers producer saturation, consumer progress, idle-reader shutdown,
Drop, direct-post capacity errors, and thread-count recovery. RTR-003 calls all
four public start and stop routes inside current-thread and multi-thread Tokio
runtimes. RTR-004 crosses zero, ordinary, reversed, and extreme FPS bounds and
varies rows and item counts for zero-column grids.

## Failure demonstrations and corrected cases

The RTR-001 validator rejects a failed automatic App, a missing first frame, an
explicit request that silently succeeds, and an explicit error without the
connection cause. Its corrected missing-bus and stalled-bus cases paint, accept
input, exit, and restore the terminal in automatic mode while explicit mode
returns the connection error.

The RTR-002 validator rejects timeout, process failure, and missing completion.
The committed baseline timed out under queue saturation. The corrected PTY
cases drain more events than the 512-event capacity, stop an idle reader, and
restore the isolated process thread count after Drop. The direct-post test
requires an immediate capacity error and recovery after consumption.

The RTR-003 validator rejects timeout, process failure, and either missing
runtime marker. After an invalid import fixture was corrected and retained as
failed history, the valid baseline reproduced the nested block_on panic in both
runtime flavors. The corrected case passes sync and async lifecycle calls in
both runtimes. The review found that its sync-stop termination observation is
premature, as described below.

The RTR-004 validator rejects timeout, process failure, and either missing API
marker. Its baseline independently reproduced a zero FPS target and the
zero-column divide by zero. The corrected cross-product keeps targets and frame
durations nonzero, including u32::MAX inputs, and every zero-column grid returns
without placed children.

## Finding to resolve

TokioEventLoop::begin_shutdown stores false in is_running before returning the
JoinHandle. The sync stop path then aborts that handle, and the test waits only
for is_running to become false. It can therefore pass before Tokio drops the
task future. TokioRunningGuard already owns the same flag and clears it when the
future completes or is cancelled. Removing the early store makes the existing
sync-stop test wait for actual future destruction and closes the proof gap
without changing the public API.

## Other limits

The threaded event-loop PTY and thread-count checks run on Linux. The Windows
implementation uses a bounded console-read timeout and the same condition
variable queue, but a local Windows cross-check could not reach project code
because the x86_64-w64-mingw32-gcc linker is not installed. Cross-platform CI is
owned by the later pre-release CI commitment, so this review does not claim a
Windows execution result.

The full default-feature library suite passed with 1,043 tests and 8 mechanism
fixtures intentionally ignored. Focused checks and no-dependency library
Clippy passed with eight build jobs. Ripwire reported no gating quality
regression for the final RTR-003 or RTR-004 source changes.
