# Rename Windows explorer entries relative to owned directory handles

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: The native call follows an untrusted link, replaces an existing target, or does not preserve the directory capability under concurrent filesystem changes.
History: The native Windows no-overwrite check fails: SetFileInformationByHandle rejects the directory-relative rename and short target buffers. The explorer already owns source and destination directory capabilities; converting them to ambient paths would lose that protection.

## Decision

Use NtSetInformationFile with FileRenameInformation, the existing owned source handle and the destination directory handle. Keep ReplaceIfExists false, supply at least the full structure size plus the variable UTF-16 name, and translate NTSTATUS to an actionable IO error. Keep names as native UTF-16. Verify short and long names, files and directories, no overwrite, and real App copy/move/rename on Windows.

## Realized by

Implementation: `src/widgets/display/file_explorer/worker/operations/rename.rs`.

Behavior checks: `src/widgets/display/file_explorer/worker/tests.rs`, `tests/api_widget_behavior/file_explorer.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.

References: [Microsoft FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information) defines directory-relative targets and buffer sizing; [NtSetInformationFile](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntsetinformationfile) defines the DELETE access requirement and completion status.
