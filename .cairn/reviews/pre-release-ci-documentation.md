# Pre-release CI and documentation review

## RID-001 mechanism construction

The contract requires main push and pull-request coverage for Linux, macOS,
and Windows, build/test/format/lint/maintenance commands, locked dependencies,
and scheduled advisory checks. No maintained main-branch workflow exists yet.

On 2026-09-16 the eight validator tests first failed because the checker was
absent. After implementation all eight passed. The valid workflow fixture is
accepted. Removing each of three platforms and each of five required commands
is rejected, as are branch/path/activity filters, disabled jobs, conditional
required commands, allowed failures, unlocked and success-masked commands,
missing advisory coverage, floating actions, and write permissions.

Repository inspection has not yet passed. The checker must reject the actual
tree until the main-branch workflow exists. Native CI execution is not covered
by these local fixtures. Read-only GitHub inspection found no legacy branch
protection and one active ruleset with creation/update/deletion restrictions
but no required-status rule. Changing remote merge policy requires separate
developer authority; do not claim workflow presence enforces passing checks.

RID-002 and RID-003 remain pending. Do not publish packages or remove queued
developer decisions without review.

The installed Cairn only recognizes extensionless mechanism declarations
(LOOP-106). The initial `.md` declaration was not recognized; rename this new
declaration before checking. The older `.md` mechanism migration belongs to
the already-agreed RID-002 complete-mechanism work, not a tool kernel edit.

## Main workflow implementation and limits

The recorded failure baseline is `20260916T214748872Z-1234535`: eight fixture
tests pass, but the actual main workflow is missing. Adding `ci.yml` makes
that exact inspection pass: push, pull-request, and manual events select
Ubuntu 24.04, macOS 14, and Windows 2022; the weekly schedule selects the
advisory job. Required Cargo commands use locked dependencies and Rust 1.91.
Actions are pinned, checkout retains no credentials, jobs are time-bounded,
and failure is not allowed. Windows uses the established GNU build setup.

Fresh `cargo +1.91.0 fmt --all -- --check` passes. Fresh strict default-target
Clippy fails in the unchanged bundled crossterm source: one misspelled lint
and four `io::Error::new(ErrorKind::Other, ...)` diagnostics. No warning
suppression was added. More diagnostics may appear after these are repaired;
do not describe the five reported errors as an exhaustive lint inventory.

Ripwire quality-delta and test-gate against the workflow-only change exit
zero. They model no workflow symbols; the only quality row is ignored local
GitNexus metadata. These results do not verify the new Python checker that
was already committed. Its eight fixture tests and actual-tree inspection
are the relevant executable checks. Git diff whitespace checking passes.

No native GitHub run or actionlint run has been performed. No code in the
framework or bundled renderer was edited. Before claiming merge enforcement,
obtain developer approval for a remote required-status rule. Repairing the
strict-Clippy source failures also needs an explicit boundary expansion.

## RID-002 mechanism construction

Shawn approved the narrow lint, push, and required-status expansion on
2026-09-16. Its decision and commitment boundaries are recorded; package
publication and unrelated source repairs remain unauthorized.

Seven link/input tests first failed because the checker was missing. They now
pass using real temporary Git repositories. Removing or leaving a link target
untracked, deleting a heading, encoding a repository escape, missing image,
reference, and HTML targets, missing/untracked/ignored inputs, tracked-but-
ignored inputs, and legacy declarations all produce failures. Valid tracked
targets pass; external links and fenced examples do not become local targets.

The actual repository inspection fails as intended. It finds three broken
vendored Markdown links, the legacy `.md` declarations, and tracked inputs
ignored by local or root rules. Preserve old receipt history while migrating
declarations. This static checker does not establish acceptance freshness;
the complete mechanism inventory must run before the closing review.

## Declaration migration verification

The 62 legacy declarations now use the installed referee's extensionless
format. Their commands and requirement lists are unchanged. Self-inputs and
the inherited ABI runner now name the migrated paths. No historical receipt
or captured output was edited.

The inherited-runner regression first failed twice against the old paths.
After correction, both tests pass: all nine fixture commands run, and a
selected input omitted from the combined declaration is rejected. These
fixtures test command selection, not the underlying acceptance behavior.
The seven documentation fixtures, actual-tree link/input inspection, and
documentation-retention inspection also pass locally. Existing ignored
review logs and the tracked bundled Cargo.lock are no longer ignored.

GitNexus staged analysis reports low risk, six changed indexed symbols and
no affected indexed processes; it does not model every moved declaration.
Ripwire's inherited-runner edit check reports a parameter change, but the
actual function still has zero parameters on both sides of this edit. Its
baseline merges many same-named main functions; this is not a signature
change. Deleted document references inside checker code still need repair,
and complete fresh acceptance evidence remains outstanding.
