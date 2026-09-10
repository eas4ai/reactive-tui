# Copy Windows symbolic links through owned reparse handles

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: Copy follows the link target, changes its stored target, writes through an ambient destination path, leaves failed staging data, or changes process-wide privileges.
History: The native Windows fixture can create an external-target symlink, but cap-std rejects copying its absolute target before the filesystem call. Unix already copies raw link contents. The explorer must preserve the link while refusing to read the external file.

## Decision

Read the bounded symbolic-link reparse payload through a no-follow source handle and apply it to a newly created file or directory inside the existing owned staging directory. Preserve the exact payload and report native permission errors. Publish only with the existing no-overwrite rename and let staging cleanup remove failures. Verify file and directory links, outside-target preservation, no-overwrite and cleanup on native Windows.

## Realized by

Implementation: `src/widgets/display/file_explorer/worker/operations/symlink.rs`.

Behavior checks: `src/widgets/display/file_explorer/worker/tests.rs`, `tests/api_widget_behavior/file_explorer.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
