# Hold the HTTP cancellation fixture open until teardown

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: The fixture can complete before cancellation, a missing close passes, or a blocked close escapes its deadline.
History: The native macOS run exposed a cancellation fixture reaching REMOVED before REQUESTS 1 was observed; 50 local repetitions passed. Prior ownership decisions require real cancellation and bounded cleanup rather than timer-based guesses.

## Decision

Keep the loopback response pending until fixture teardown for the engine cancellation case. Continue to require the server-observed request before sending cancel. Measure close_dialog itself against the existing one-second bound instead of including unrelated App startup and rendering. Preserve result, submission and resource assertions. Demonstrate that omitting close is rejected by the bounded App harness, then run corrected native macOS and Windows checks.

## Realized by

- dc8763df109a7f31409784e801fc5c660d64ddd0 Keep the HTTP cancellation fixture pending until close
