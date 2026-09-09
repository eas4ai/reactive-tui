# Allow cold Windows clipboard startup within a fifteen-second bound

Level: Consequential
Decided by: Codex
Supersedes: allow-five-seconds-for-windows-clipboard-commands
Cause: the stated condition occurred
Rests on: API-008
Would be wrong if: A normal cold Windows clipboard operation still exceeds fifteen seconds, cancellation leaves its child alive, or a stalled command escapes the deadline.
History: The earlier two-to-five-second increase was verified after a PowerShell lifecycle fixture had warmed startup. Fresh runners now time out at five seconds; this second revision is Consequential because it increases the maximum synchronous Windows wait. Cold native acceptance must run without a warm-up process, and another normal-startup failure requires developer review rather than another guessed increase.

## Decision

Retain the previous child ownership, cancellation, temporary streams, transfer limit, error propagation and all five native backend requirements. Use a fifteen-second deadline on Windows and retain two seconds on Unix. Two fresh Windows runs exceeded five seconds before the first useful operation; a separate fresh-runner diagnostic measured 3.25 seconds for cold copy and 0.27 to 0.34 seconds for warm calls. Fifteen seconds provides margin for variable startup while keeping failure bounded; a stalled synchronous Windows call can now wait up to that limit. Keep an independent sixteen-second Windows lifecycle assertion and the three-second Unix assertion. Use the native child fixture for lifecycle tests so they do not warm PowerShell before the real clipboard probe. Remove the temporary timing job after preserving its observations and require new cold Windows round trips plus lifecycle evidence before acceptance.

## Realized by

(none yet: recorded, not built)
