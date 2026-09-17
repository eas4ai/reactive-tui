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

## RID-003 mechanism construction

Eight housekeeping fixtures first failed because the checker was missing.
They now pass against real temporary Git repositories. They reject an ignored
tracked file, removed literal paths and exceptions, removed anchored-glob
prefixes, stale package include/exclude rules, a malformed manifest, and a
queued decision. The rejected queue file remains untouched. Correct negative
ignore exceptions and package-relative nested rules pass.

A ninth test first rejected intentional OS noise and Cairn's live marker.
After listing those intentional local outputs explicitly, all nine pass.
Generic artifact globs do not by themselves prove a retained exclusion is
useful; the closing review must inspect that intent. GitNexus reports low
impact for the local-noise constant with no indexed processes affected.

The actual tree still fails on removed example and diagnostic-log rules,
the bundled documentation's obsolete book-output rule, and nine unreviewed
decisions. This is the required violating-tree demonstration, not a pass.
The developer controls queue review; source repairs and native CI execution
remain unfinished.

The committed RID-003 failure baseline is
`20260916T220625987Z-1436272`. Remove the obsolete examples-blech ignore,
the removed diagnostic-log exception, and the bundled docs/book rule (its
sole ignore file has no remaining producer or tracked target). Fresh fixture
tests and real-tree inspection must still reject the unchanged decision
queue. This repair does not claim RID-003 passes.

## EMB-003 revised mechanism review, 2026-09-17

Examined candidate 3356ca4c8202e7d75d4c9a0ee793cc17aff6d23d without changing
source: the old and current EMB-003 contract, embedded-terminal declaration,
check-embedded-terminal.sh, check-embedded-terminal-pty.py, the internal
runtime probe, the child-input integration test, and the keyboard-mode test.
The revision replaces the removed shell example with the internal acceptance
probe. The script builds and drives that exact App-based probe; its quit key
is Ctrl+Q. Integration checks require text/Enter and child Ctrl+C delivery;
the keyboard test distinguishes normal and application cursor mode and checks
Escape, Enter, Unicode text, and Ctrl+C against actual encoded bytes.

Safe violating case: driving /usr/bin/true with the real PTY harness exits
one with "embedded shell did not produce b'RTUI>'; exit=0". A successful
noninteractive process cannot satisfy the prompt assertion. Corrected case:
build the current internal probe using Rust 1.91.0 and an isolated target,
then run the unchanged PTY harness. All three cases pass: interactive input,
background output, two resizes, child interruption, host Ctrl+Q, terminal
restoration and child reaping; bounded-input worker-error cleanup; and the
cell-95 C1-text regression. These editing-time diagnostics are not Cairn
acceptance receipts. The full mechanism still needs its committed-tree run.

No mismatch was found between the revised EMB-003 contract and its checks.
Separate open technical finding: the inherited renderer gate runs the shipped
specification lint, which currently reports twelve pre-existing multi-
obligation sentences. The direct lint run failed; repair the writing without
weakening any obligation before claiming the complete mechanism set passes.
The configured macOS SSH host returned "No route to host"; native macOS
verification is outstanding, not replaced by workflow inspection.

## EMB-006 revised mechanism review, 2026-09-17

Examined candidate 1caf5d16466cde20f661ca89c20e8f906c9e9089. The same example-
to-internal-probe revision applies to EMB-006. The launcher actually invokes
App::run, rather than directly painting a terminal. The PTY checks verify raw
mode was entered and the original termios, visible cursor, and alternate
screen were restored on both normal Ctrl+Q and an application worker error.
The preceding real-probe positive cases and /usr/bin/true negative case apply
here too: a launcher bypassing App cannot satisfy raw-mode and restoration
assertions. No source was changed while examining these checks.

The inherited renderer command runs its internal tests, integration tests,
an actual probe build, deterministic PTY fixtures and live PTY assertions,
then the shipped specification lint under set -eu. Running that exact lint
confirms the twelve writing failures; the gate does not mask them. Its repair
is separate work, not a reason to remove renderer inheritance. No mismatch
with the revised EMB-006 requirement was found. The full acceptance run is
still pending and must not be inferred from the three direct PTY passes.

## RND-005 revised mechanism review, 2026-09-17

Examined candidate 2268947559f408366389113f5a1eb42a3d482333: previous and
current RND-005, the renderer declaration and shell gate, the App-based
internal renderer launcher, and both PTY scripts. The revision names the
internal probe instead of the removed example; restoration obligations and
falsifier are unchanged. The script builds that probe and tests normal
Escape and Ctrl+C, a real controlled input error, and a real Rust panic.
It requires changed termios during execution, exact restoration afterward,
cursor restoration before alternate-screen exit, expected diagnostic text,
and the correct exit status. It does not accept merely emitting quit bytes.

The safe /usr/bin/true case is rejected with "initial counter frame missing;
exit=0". The actual current probe built with Rust 1.91.0 passes all four live
PTY cases; four predicate fixtures also pass and reject partial escapes,
unfinished updates, wrong dimensions, stale counters and cleared screens.
These are direct diagnostics, not full-mechanism acceptance receipts. The
separately recorded specification-lint failures still block that full gate.
No RND-005 mechanism mismatch was found; no source was changed during review.

## RND-006 revised mechanism review, 2026-09-17

Examined candidate c2f99b013fef7efd4b55b66dccf15df7f167728f using the sources
and live comparisons recorded immediately above. RND-006 now names the
internal acceptance probe; it still requires an App-routed counter change
and working documented quit keys. The harness sends Space and requires a
new completed frame with Count: 1; the source counter updates through
RootComponent::handle_event. It requires successful, bounded exit after
both Escape and Ctrl+C. The no-frame /usr/bin/true case fails, the actual
probe passes both normal input/quit cases, and the four predicate fixtures
reject stale or incomplete counter updates. No mismatch was found and no
source was changed while reviewing. Full committed acceptance is pending.
