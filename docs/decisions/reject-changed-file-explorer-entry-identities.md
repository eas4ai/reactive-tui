# Reject changed file-explorer entry identities

Level: Judged
Decided by: Codex
Rests on: XIS-003,pre-release-file-explorer-identity-safety
Would be wrong if: The platform cannot provide stable device and file identifiers for capability-scoped metadata, or a supported filesystem can replace a checked name during the final path-based remove.

## Decision

Treat the device and file identifier pair as the entry identity. For copy, compare metadata from each opened file or directory handle with the metadata inspected before the race point; compare symbolic-link metadata again after reading its contents. For remove, open regular files and directories without following links, compare handle identity with the inspected identity, and recheck the named entry immediately before a path-based file or link removal. Abort with a clear changed-entry error on every mismatch. Keep directory deletion handle-based through remove_open_dir. Document that concurrent replacement makes the operation fail instead of acting on the replacement.

## Realized by

- 12ece746ab9d11845eafab13b4a5d3b89348b5bd fix: reject changed file-explorer identities
