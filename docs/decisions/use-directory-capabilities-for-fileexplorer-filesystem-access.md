# Use directory capabilities for FileExplorer filesystem access

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: A relative traversal or symlink lets FileExplorer access paths outside the configured root, or a new dependency replaces a sufficient existing rooted filesystem abstraction.
History: FileExplorer previously used ambient std filesystem paths and did not enforce its configured root. This implements the rooted ownership decision without treating a canonical-path string check as protection against path replacement.

## Decision

Use cap-std directory handles for FileExplorer reads, previews and operations relative to the configured root. Open that root once per configuration on the worker. Keep public paths as PathBuf and derive labels separately. Create destinations exclusively so another entry cannot be overwritten. Copy and cross-filesystem move must report partial results and preserve the source when copying fails. No new filesystem permissions are granted; host operating-system permissions still apply. The dependency provides directory-relative path resolution, including confinement of symlinks, rather than adding local unsafe path traversal code. Source reference: https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html and https://github.com/bytecodealliance/cap-std.

The operation layer uses cap-fs-ext for explicit no-follow and nonblocking opens, rustix for Unix atomic rename without replacement, and Windows handle-based rename with ReplaceIfExists false. Copying stages data privately before publication; same-filesystem moves preserve rename semantics and report cross-filesystem failures without deleting the source. The existing transitive time crate formats file timestamps in UTC. Windows API reference: https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info. These choices remain subject to platform checks.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/display/file_explorer`.

Behavior checks: `tests/api_widget_behavior/file_explorer.rs`, `tests/api_widget_behavior/orca_data.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
