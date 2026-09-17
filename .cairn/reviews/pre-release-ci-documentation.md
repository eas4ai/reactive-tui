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

## Strict lint repair diagnostics, 2026-09-17

Receipt 20260917T113050161Z-909848 reproduces five bundled-crossterm errors
on Rust 1.91.0. Correct the existing misspelled unnecessary_wraps lint name
and use Error::other with unchanged Other kinds and messages. The next local
strict run identifies a duplicate optimizer test-module attribute and the
wizard's nested else-if; remove the stale commented external-test-module
attributes and flatten that equivalent branch. No lint gate is relaxed.

Editing-time checks: strict all-target Clippy passes, formatting passes,
library tests pass (1051 passed, 8 ignored), bundled crossterm tests pass
(94 passed, 7 ignored), and wizard integration tests pass (12 passed).
The first wizard build failed with Disk quota exceeded in the isolated
temporary target. Cargo clean removed only this task's generated root-crate
artifacts there (24.9 GiB, rebuildable); the subsequent build and tests pass.
The user's shared target and running builds were not changed. New acceptance
receipts still require a committed candidate.

GitNexus impact: parser-error helper medium, eleven direct parser callers;
cursor and keyboard-status helpers low, two direct callers each; poll/reset
and test module have no indexed inbound callers. No indexed processes are
affected. Rust-analyzer and Ripwire additionally identify the wizard event
handler's call, which GitNexus misses. Edit checks for the parser helper,
cursor helper and wizard method report unchanged contracts and no
incompatible callers. Ripwire quality-delta exits two for recent churn in
the keyboard-status helper; test-gate exits four with sixty suggested tests
and 260 statically untested symbols. These are not passing tests or proof of
new behavior regressions. Full maintained-suite reruns remain outstanding.

## Dependency policy repair diagnostics, 2026-09-17

Receipt 20260917T113752918Z-971432 rejects the optional wgpu dependency graph:
hexf-parse 0.2.1 has a disallowed license and six older dependency versions
produce five duplicate-package groups. The recorded Judged decision keeps
the approved graph and scopes each exception to an exact package version;
advisory denial, global license rules, duplicate denial and source checks
remain enforced. Windows GNU is included in the policy target graph.

The new policy regression fails before configuration repair and passes
afterward. All twelve validator tests pass. The actual corrected DQC-001
command passes cargo audit, all four cargo-deny checks and atty reachability.
These are editing-time diagnostics; a fresh committed acceptance receipt is
still required. GitNexus finds no indexed inbound callers or execution flows
for the changed test class (low risk); Ripwire reports an unchanged class
contract and no incompatible callers. Cargo.lock is unchanged.

## Panic replay repair review and diagnostics, 2026-09-17

Receipt 20260917T115325551Z-1107399 shows a real TRL-001 regression: the
DQC-003 logging change removed the panic replay after screen restoration.
The new candidate reports failure-only panic text through backend-owned
writers, keeps normal diagnostics in logging and resumes the original payload.
Four new focused regressions pass; the original worker/main PTY cases pass,
and the signal/prior-hook/foreign-owner cases and DQC-003 diagnostics pass.
Strict all-target Clippy and formatting pass. Performance-context integration
tests pass (10), and hook lifecycle integration tests pass (7).

The serial library suite passes (1055 passed, 8 ignored). The parallel suite
fails one animation-cancellation assertion (1054 passed, 1 failed, 8 ignored).
An animation-only parallel run also fails a midpoint/completion assertion
(18 passed, 1 failed), without executing the new panic tests. Those existing
tests share RUNTIME and immediately expect an update even though its atomic
update guard can return when another update owns the runtime. This outstanding
test-integrity finding is not resolved by a serial-only passing run.

Fresh read-only reviewer panic_cleanup_review returns fix-first. The review
finds a P1 introduced teardown-order regression: App::run ends the implicit
local-hook scope before dropping App. Component unmount callbacks then lose
their local arena. Source inspection confirms Scope::drop removes its binding
before App::drop clears mounted components. Preserve the prior teardown scope
and demonstrate the corrected normal-exit behavior before accepting this repair.
The reviewer changed no files and ran no builds. Its requested model/effort
was gpt-5.6-sol/high; realized settings and token usage are unobservable.

Ripwire quality-delta exits two: worker complexity increases 37 to 41, shared
panic-cleanup branches are duplicated, capture writers duplicate existing test
patterns, recent-churn indicators fire, and name-based dead-code results miss
new trait dispatch and executed tests. The code-quality corrections must keep
restoration/reporting together and remove shared cleanup duplication. Test-gate
exits four: 64 suggested test files and 139 statically untested impacted symbols;
this is not a passing test result. Complete maintained-suite checks remain pending.

### Corrected candidate and fresh review

App teardown now remains inside the same implicit local-hook scope as its run.
The first teardown fixture incorrectly reused hook resources already cleared
by App cleanup; that fixture did not establish a valid red/green result. The
replacement creates a fresh Hooks owner during root destruction and verifies
that the local arena is still bound. All five focused tests pass, with the
actual-PTY fixture ignored only because the separate mechanism invokes it.
Local-hook integration tests pass (13). The latest serial library suite passes
(1056 passed, 8 ignored). Fresh strict all-target Clippy, formatting, whitespace,
hook-lifecycle tests (7), and performance-context tests (10) pass.

The shared cleanup helper removes the duplicated panic-cleanup branches, and
TerminalOutput owns restoration plus replay; worker complexity returns to its
prior value. The latest Ripwire quality-delta still exits two (35 diagnostic
rows, 17 gating): tiny test-fixture duplicates, recent churn, a larger test
module, and dispatch/test dead-code misses remain. These diagnostics are not
reported as a clean quality gate. No production abstraction is added merely to
eliminate tiny capture-writer or text-element fixture matches.

Fresh read-only reviewer panic_cleanup_fresh_review returns ship for this
bounded repair, with no blocking findings. The parent inspected the complete
diff and ran the checks above. Requested reviewer settings are gpt-5.6-sol/high;
runtime settings and token usage are unobservable. No reviewer changed files
or ran builds. Original payload preservation still assumes later owned-resource
destructors do not panic. The preexisting writer-panic-during-unwind restoration
hazard is not claimed repaired. The separate parallel animation-test failure,
native verification, remaining audit repairs, and fresh committed receipts are
still outstanding; this review does not accept the complete Fable remediation.

## DQC-004 mechanism repair findings, 2026-09-17

Receipt 20260917T122035754Z-2213988 fails after the private build target reaches
its quota. Cargo's primary JSON diagnostics were omitted from its report.
Symbol-light dev/test profiles retain explicit debug assertions and overflow
checks. The first two fixtures fail before that implementation and pass after.
The former inventory also assumes graphics targets can compile in the FFI
graph. Separate required-feature graphs retain every intended target rather
than exempting the four optional graphics tests. A positive feature fixture
fails before correction and passes after; missing default/FFI/graphics targets
remain failures. The first graphics command incorrectly used dependency name
wgpu, not approved feature wgpu-graphics; this agent mistake is corrected.

A warm shared-output run also fails a Taffy type identity check despite both
types reporting version 0.9.2. The facade's unhashed rlib can be overwritten
by a build with different dependency features. A cold run using separate
default, FFI, graphics and mutant output directories passes without changing
the Rust test or dependencies. This supports artifact mixing rather than a
source-type defect; the underlying Cargo behavior is not proven fixed.

Read-only inspection of that passing report finds another mechanism hole:
ffi_export_inventory searches masked code for extern "C", but its lexer masks
the ABI string too. It reports no exports, and the validator accepts the empty
list. Correct the scanner and require a nonempty export inventory before this
repair is accepted. Preserve rejection of comments, strings and test helpers.

The corrected full diagnostic run passes: 85 intended integration targets
compile across the three isolated graphs, and the source inventory names 243
C-ABI functions. Both sampled originals pass and their isolated equality
mutants fail at the assertions. An empty-export fixture fails against the old
validator and passes after correction. The scanner fixture initially fails on
its new root argument, then exposes an attribute-line/function-line mismatch;
the corrected fixture passes and excludes comments, raw strings and test-only
callbacks. Do not describe its initial argument error as a lexer failure.
Fresh whitespace checking passes. These are editing-time results, not a fresh
committed DQC-004 receipt. The separate parallel animation failure remains open.

Ripwire's earlier quality-delta exits two with three gating rows: a same-name
report complexity row attributed to an unchanged Swift file, recent churn and
the expanded validator class. Test-gate exits four with 144 statically untested
symbols and no recognized Python unittest roots. Executed fixtures and the
full checker establish coverage; these static diagnostics are not claimed clean.

Independent reviewer test_integrity_review returns fix-first. Matching only a
masked identifier allows a commented extern declaration to cross a newline and
invent an export from a live function call. The borrowed test-only range helper
also excludes cfg(not(test)) and cfg(any(test, feature = "ffi")), which can ship
in production. These source-established findings are recorded before correction;
the reviewer ran no tests or builds. Add regression cases for comment crossings,
declaration trivia and production cfg alternatives before accepting the repair.

Both new export regression tests reproduce the actual defects before correction:
ordinary_call is falsely counted and both production-gated exports are missing.
The corrected scanner restores C ABI tokens only after live extern keywords and
skips comment trivia. Test-only exclusion evaluates test as false and leaves
other platform/feature atoms unknown; not(test) and any(test, feature) remain
possible production declarations while all(test, feature) is excluded. Nested
cfg truth-table cases, comments between declaration tokens and a Rust ABI with
a commented C example pass. The full 27-test validator suite passes, and the
full checker again passes with 85 targets, 243 C-ABI functions and both assertion
mutants rejected. These remain editing-time results, not committed evidence.

Final Ripwire quality-delta exits two with 11 gating rows, including conservative
scanner/other-audit-loop duplication, test-fixture duplication, recent churn and
class length. Its same-name report row still points at unchanged Swift source;
the new cfg evaluator has complexity 22. Test-gate exits four with no recognized
unittest roots. These diagnostics are disclosed, not characterized as clean.

Parent inspection finds the same cfg-range problem also affects unsafe_test_hooks:
production mutable globals under not(test) or any(test, feature) can be ignored.
This is within DQC-004's shipped-hook obligation. Record and demonstrate this
separate finding before reusing the conservative cfg filter for that audit.

The unsafe-hook production-cfg fixture actually fails with an empty finding list
before correction and passes after, retaining detection of both production
globals and exclusion of the all(test, feature) helper. The full checker passes
again, including its 28-test validator invocation, all 85 compiled targets,
243 named C-ABI functions and the two rejected assertion mutants. Whitespace
checking passes. No production Rust source or dependency is changed here.

Fresh independent reviewer test_integrity_fresh_review returns ship for this
bounded repair. Its read-only inspection and five nested cfg cases plus a
mocked-source nested-comment/exclusion probe pass; it reruns no builds or full
suite. Parent separately reruns all 28 validator tests: pass, 24.935 seconds.
The source inventory is not a binary-export proof; mutation is sampled; future
implied/default required-feature gates need feature-closure handling. Current
Cargo target gates are direct ffi/wgpu-graphics and compile completely. Failed
static diagnostics and wider audit/native findings remain open. Requested
reviewer Sol/high settings and token usage are not runtime-observable.

## API-008 evidence-retention repair, 2026-09-17

Receipt 20260917T125556374Z-3885772 runs all 13 clipboard API tests and both
process-cleanup tests successfully, then fails because the verifier still reads
deleted docs/analysis/clipboard-platforms records. Preserve the five-native-
backend requirement; relocate producer, verifier and workflow artifact capture
to retained .cairn/reviews/clipboard-platforms and declare that evidence input.
The output-directory regression fixture fails against the old path and passes
after correction. All 12 synthetic validator tests pass, rejecting absent,
stale, dirty, wrong-platform, damaged, nonlocal and markerless evidence. The
native disposable-desktop guard rejects both Windows and macOS before builds
or clipboard access. Synthetic unit records never enter the acceptance folder.

The full editing-time API checker reruns all 13 behavior tests and both lifecycle
tests successfully, then correctly rejects uncommitted native source inputs.
Fresh native acceptance remains unverified until source is committed and real
backend records are regenerated. No historical output or receipt is rewritten.
Whitespace checking passes. WinBoat answers SSH with Windows 10.0.26200.8037.
The configured macOS host 10.66.231.181 still returns No route to host; ping
fails despite a route via ztkti3lasa. Tailscale is not installed, and read-only
ZeroTier peer inspection needs unavailable interactive sudo authentication.
Do not install/reconfigure networking or substitute hosted CI to bypass that
external host condition. All wider audit findings remain open.

Reviewer clipboard_retention_review stops after wake because it misinterprets
the parent's marker as a separate reconciliation task; it examines no code.
Clarify that read-only review helps finish the same parent-owned API-008 action
without clearing its marker. Fresh reviewer clipboard_retention_fresh_review
returns ship for the bounded path repair only. It independently runs all 12
validator tests, reproduces the old-path failure with an in-memory override,
and passes whitespace checking without editing source or running native builds.
Requested reviewer settings are Terra/high then Sol/medium; runtime settings
and usage are unobservable. Parent Ripwire quality-delta exits two with one
gating RECORDS churn row; test-gate exits four with no recognized unittest
roots. These are not clean static-check claims or five-native-backend passes.

## API-008 local native evidence, 2026-09-17

The committed source snapshot 06d9dedfded5c51451aa8fe08d729e6ab5b68542
passes five real clipboard round trips and both process-cleanup tests on each
private Linux desktop: Wayland, xsel and xclip. WinBoat passes the same native
round trips and process tests, plus its Windows console adapter test. All four
producer records have input digest
14ae7807cc5b0a47cca534a72c0a61bced20267f76acca25cc30408dd4bd10bf.
The parent reads every record and captured output and confirms their output
SHA-256 hashes. No synthetic fixture enters the acceptance directory.

WinBoat uses an isolated clone of that exact committed Git bundle. Its first
build fails because dlltool.exe is unavailable on PATH. Adding Rust's bundled
tool directory exposes a second failure: dlltool cannot start its assembler.
The parent installs the repository's GNU build prerequisite in this task's
private VM directory, without changing system PATH or Rust's default toolchain.
The MSYS2 20260611 portable archive matches its published SHA-256 checksum and
Windows validates its Authenticode signature before extraction. Signed pacman
updates and the mingw-w64-x86_64-gcc installation pass. The native producer then
exits zero. These tooling failures are not counted as passing source tests.

The Windows captures also contain unused-import, unused-mut and dead-code
warnings. This native clipboard pass is not a strict Windows lint pass. Keep
those diagnostics for the supported-platform gate; do not suppress them here.

The configured Mac briefly answers sw_vers with 26.6.1 and reports Rust 1.97.1,
then subsequent SSH requests time out or return No route to host. No macOS
clipboard probe runs. The parent asks whether its clipboard may be replaced
by test values; that answer is still pending. Current real macOS evidence is
required before API-008 can pass. Do not use hosted CI, alter host networking,
weaken native coverage or clear developer queue markers to bypass this limit.

The real verifier now rejects missing Darwin evidence, rather than missing
Linux evidence. Full staged whitespace checking reports native JSON CRLF and
captured test-output blank lines at EOF. Preserve those producer bytes and
their recorded hashes; do not edit captured evidence to satisfy formatting.
The handwritten review passes its separate staged whitespace check.

### Native macOS completion after developer approval

Shawn supplies shawnmcallister@10.66.231.181, approves replacing the Mac's
clipboard without restoring it, and requests a 24-hour caffeinate process.
That process remains running during native verification. The parent installs
Rust 1.91.0 alongside existing toolchains, without changing the default. Rustup
also performs its automatic self-update from 1.29.0 to 1.29.1. No host networking
is changed and no hosted CI is used.

An isolated Mac clone of the same committed Git bundle passes five real pbcopy/
pbpaste round trips and both process-cleanup tests on macOS 26.6.1 arm64. Its
producer exits zero. The source digest matches all four earlier backend records.
The parent retrieves and reads the native JSON and both captured outputs, then
confirms their SHA-256 hashes. The unchanged five-backend verifier now passes.
The Mac capture contains a dead-code warning for NodeId::serial; this clipboard
pass is not a strict macOS lint pass. Fresh formal API-008 acceptance and the
remaining Fable gates still need to run. Historical captures remain unchanged.

## API-020 probe-path finding, 2026-09-17

Formal API-008 receipt 20260917T133856696Z-565850 passes all 12 validator tests,
13 clipboard API tests, both process-cleanup tests and the five-native-backend
verifier. Its committed native records include the developer-approved Mac run.

The next formal gate, API-020 receipt 20260917T133928714Z-578610, builds the
image_host_probe successfully in the configured private CARGO_TARGET_DIR, then
fails normal capture. The retained host log proves GNOME Terminal tries to
execute the missing repository target/debug/examples/image_host_probe instead.
The capture driver's default path ignores the binary just built by the closure
runner. A stale binary at that path could also test the wrong source. Record
this finding before repair. Preserve the failed capture and receipt unchanged.
Have the closure runner select the exact example executable Cargo reports and
pass it explicitly to both normal and forced-timeout captures. Neither a
successful build nor a serial test pass replaces actual process-cleanup proof.

### Probe-path repair verification

The runner now selects Cargo's compiler-artifact executable for the named
example and passes it to both captures. The original runner failed three of
four regression tests; the repaired runner passes all four, including paths
with spaces, wrong artifacts, missing executable and build failure.

The real editing check passed at
api-020-closure/20260917T134220159161Z. Both commands use the private target's
executable. Normal capture verifies image update, movement and removal.
Normal and forced-timeout results both report no remaining owned processes.
These diagnostics precede committed acceptance evidence, not replace it.
Preserve captured logs and screenshots byte for byte.

Fresh read-only reviewer /root/api020_launch_review returned ship with no
blocking findings. It read current full sources and the structural diff;
the parent supplied the original changed lines. It ran no checks. Native
cleanup is established by the parent's real run, not mocked unit results.

Ripwire's qualified edit checks found no incompatible callers. Its quality
delta exits 2 for churn and test-fixture findings; test-gate exits 4 because
it does not recognize unittest discovery. Neither static gate is reported
as passing. Actual unit tests and native cleanup passed. GitNexus cannot
index this hidden helper and the primary graph transport is closed; source
inspection and Ripwire found main's two calls. No production Rust changes.
Wider Fable findings remain open.

## API-018 retained-documentation finding, 2026-09-17

Formal receipt 20260917T135150727Z-910875 fails only inventory: the checker
opens deleted docs/supported-api.md. Retained results show all selected
Props/CSS tests, three rustdoc builds, three doctest configurations, Cargo
examples, and Rust/C/TypeScript/Python examples pass. This is not an API-018
acceptance pass. Preserve the failed receipt and 64 output files unchanged.

Source inspection also finds collect_examples searches README, docs root,
include and TypeScript guides but omits manual chapters. Its captured
examples.json confirms no manual files. The mechanism omits manual from its
inputs. A broken example in the maintained manual could therefore escape
the check. The crate's introduction still names the deleted matrix.

Repair these findings separately: publish a source-grounded supported-module
matrix in manual, link it from the manual and README, point the checker and
crate introduction there, collect maintained manual examples, and declare
all affected inputs. Use isolated regression fixtures to prove a missing
module or missing linked evidence fails and nested manual examples enter
the compilation inventory. Do not infer fresh native or release readiness
from module coverage or compilation. Retain approved host/protocol limits.

### Documentation capture collision

Review of the preserved baseline results finds two consumer-library entries
with different SHA-256 values but the same consumer-library.out path. Props
and examples each rebuild the facade; the second capture overwrites the
first. The earlier JSON entry can no longer be checked against its output.
Retain the historical outputs unchanged. Give the two phases distinct log
names, prove both captured hashes match separate retained files, and rerun
the complete checker. This is a capture-integrity repair, not new API scope.
