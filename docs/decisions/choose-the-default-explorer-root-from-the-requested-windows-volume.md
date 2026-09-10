# Choose the default explorer root from the requested Windows volume

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: A caller-specified root widens, paths outside that root become readable, or a convenience builder cannot browse its requested drive or UNC share.
History: Prior API reversals require native evidence and explicit ownership. The current native failure shows that slash resolves on the working drive while a convenience builder requests a different drive. This remains Judged because explicit root capabilities stay unchanged.

## Decision

When FileExplorerBuilder has no caller-specified root and receives an absolute Windows current path, choose that path volume or UNC share root. Record whether root_path was called so current_path never replaces an explicit boundary. Preserve Unix defaults and verify native convenience builders plus explicit-root confinement.

## Realized by

`FileExplorerBuilder` tracks whether `root_path` was supplied. On Windows,
`current_path` derives an unconfigured boundary from the absolute path's volume
or UNC share. Native unit cases cover drive, verbatim and UNC paths while
preserving explicit roots. The native convenience-builder failure is retained
in `.cairn/reviews/api-011-windows-34500402680-filesystem-app.out`; the correction
awaits its native run.
