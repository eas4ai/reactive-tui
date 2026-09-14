# Resolve the 2026-09-14 pre-release audit findings before a public release

Surfaced from: unstated
Captured: 2026-09-14T14:37:26.864Z

Read-only adversarial audit of reactive-tui 0.0.7 at commit 7489a52b found the crate not ready for a public production release: 4 blockers (cannot publish, 104 MB package, no CI on main, README links deleted docs), 6 high (three C ABI memory-safety defects, render-worker panic leaves the alternate screen, animation hooks spawn a thread per frame, vulnerable dependencies), 12 medium and 19 low. All repository checks pass on Linux; Windows and macOS were not built. The detailed findings with file and line references follow.

## Scope and method

Read-only pre-release audit of reactive-tui 0.0.7 at commit 7489a52b (clean
tree, 2026-09-14 10:09 EDT). The coordinator ran the repository's own checks
on this Linux host (rustc 1.95.0) and dispatched five read-only reviewers by
area: C ABI and unsafe code, core runtime panics, external input surfaces,
concurrency and shutdown, release readiness. Reviewers could not run cargo or
modify files. The coordinator re-read the source for every Blocker and High
finding and for M1, M2, M3, M6, M7, M8 and M9 before recording them. No
exploit or proof-of-concept code was written. Nothing in the repository was
modified by the audit.

Full report with the same content:
https://claude.ai/code/artifact/14e6e9c6-25a7-4281-adb7-84ef31eef192

While the audit ran, HEAD advanced to f792cedf and a separate Codex session
began the crates-io-release-preparation commitment in this working tree at
10:21 EDT. Its uncommitted edits target B1, B2 and M10 (0.1.0, renamed forks
with pinned versions, registry libghostty-vt, rust-version 1.91, include
allowlist). That work was not reviewed and is not part of this entry.

## Verdict

Not ready for a public production release. The runtime is stronger than the
version suggests: the full suite passes, clippy and rustdoc are clean, and the
input and process boundaries are carefully defended. Release is blocked by
packaging, CI and documentation, and by a small number of memory-safety and
lifecycle defects listed under High.

Counts: 4 Blocker, 6 High, 12 Medium, 19 Low.

## What ran

| Check | Result |
| --- | --- |
| cargo build --locked | pass (89 s warm) |
| cargo check --locked --no-default-features | pass |
| cargo check --locked --features ffi | pass |
| cargo doc --locked --lib --no-deps | pass, 0 warnings |
| cargo clippy --locked --all-targets | pass, 0 warnings |
| cargo test --locked --no-fail-fast | 1,967 passed, 0 failed, 13 ignored, 80 binaries plus doctests, 102 s |
| cargo audit | 5 vulnerabilities, 3 unmaintained |
| cargo deny check | same 5 vulnerabilities, 3 unsound, 4 unmaintained, duplicate crates; no deny.toml so the license section is unconfigured noise |
| cargo publish --dry-run --no-verify | fails: path dependencies carry no version |
| cargo package --list | 16,492 files, 15,550 under .cairn/ |
| cargo check --target x86_64-pc-windows-msvc | could not run: oniguruma C build needs a Windows toolchain |

## Blockers

B1. The crate cannot be published. Cargo.toml:24 crossterm, :50
reactive-tui-macros, :53 suprtui and :74 libghostty-vt (git) have no version.
Adding versions is not enough: suprtui is publish = false, the macros crate is
not on crates.io, and a registry crossterm entry would resolve to stock 0.29.0
and drop the input-readiness patch that src/backend/crossterm/REACTIVE_TUI_PATCH.md
says the path dependency exists to keep. Fix: decide the channel; for
crates.io publish the macros crate, suprtui and the patched crossterm under
project-owned names and depend on a registry libghostty; for git-only say so
in the README and tag.

B2. The package would ship 104 MB of review receipts and Windows binaries.
Cargo.toml:14-20 excludes only directories that no longer exist. .cairn/ holds
15,550 tracked files (104 MB) and src/terminal/pty/windows/runtime/ holds
3.4 MB of conpty.dll and OpenConsole.exe. crates.io caps a package at 10 MB.
Fix: an include allowlist; decide whether tracked ConPTY binaries belong in
the crate or in the hash-verified download install-conpty-runtime.py already
performs.

B3. Nothing runs on push or pull request to main.
.github/workflows/clipboard-platforms.yml:3-6 triggers only on the branch
codex/clipboard-platform-verification and manual dispatch, runs macOS and
Windows only, never runs cargo test, clippy or fmt, and uploads from the
gitignored docs/analysis/**. Fix: one workflow on push and PR to main across
Linux, macOS and Windows running scripts/check-default-suite.py and
check-maintenance.py, with cargo audit scheduled.

B4. The README and the ABI policy point at deleted documentation. Commit
45b7a9ab deleted 421 files including the supported-API matrix, widget and
image acceptance records and the C ABI policy; .gitignore now ignores docs/*
except spec, commitments and decisions. README.md links 10 missing files,
include/README.md links three, and 8 of 35 Cairn mechanisms still read them:
scripts/check-api-documentation.py:14, check-api-widget-behavior.py:18,
check-api-residual.py:242, check-binding-abi.py:15-16,
check-widget-platforms.py:15,148, check-conpty-platform.py:18 and two scripts
under verification/. Fix: point the README at manual/, restore or relocate the
ABI policy and baselines, rerun every mechanism.

## High

H1. rtui_app_quit writes into freed memory. src/ffi/app.rs:318-345 does
Box::from_raw then App::run(self) (src/app.rs:124), which moves the App out
and frees the allocation; rtui_app_quit then writes running = false through
the stale pointer. Because run blocks, every real quit call is a write to
freed memory. The TypeScript binding avoids the function. Fix: a shared stop
flag with the existing waker, or remove rtui_app_quit from the ABI.

H2. Callback types are declared non-nullable and then null-checked.
src/ffi/reactive.rs:104-107 declares RTuiEffectCleanupCallback as a bare
extern "C" fn; :683 tests it against null, so NULL is intended. A NULL from C
is an invalid Rust value and the compiler may delete the check. Same shape for
RTuiRootComponentCallback at src/ffi/app.rs:91-92. foreign.rs already uses
Option<extern "C" fn> correctly. Fix: Option on every optional callback and
regenerate the header.

H3. textBufferResize desynchronises length, and about 78 exported functions
have no panic guard. src/ffi/text.rs:136-152 resizes the storage vectors and
capacity but not length; shrinking then rendering (:308-316, :392-400)
indexes out of bounds and panics inside an extern "C" function without
catch_panic, which on rustc 1.81 or newer aborts the host process. Growing
makes the next write (:222-233) push past the pre-filled elements and
invalidate pointers from textBufferGetCharPtr. Unguarded functions span
src/ffi/lib.rs, text.rs, stats.rs and terminal.rs. Fix: clamp length on
resize; wrap every exported body in the catch_panic helper the rtui_* family
uses.

H4. A render-worker panic leaves the shell on the alternate screen, and panic
messages are lost either way. src/backend/suprtui.rs:409-540 runs the worker
without catch_unwind and the worker owns alt-screen enter and leave
(src/backend/suprtui/output.rs:144,154). If it panics, shutdown (:226-241)
cannot deliver the restore command and only raw mode is restored. A main-thread
panic restores the terminal but its message is printed onto the alternate
screen that is then discarded. install_panic_handler (src/platform/unix.rs:431)
has zero callers, and if installed its |_| closure discards the panic info and
never chains the previous hook. Fix: catch_unwind per command in the worker,
a direct restore path in shutdown, a chained panic hook installed by App::run.

H5. Animation hooks rebuild their state every render and use_transition spawns
an OS thread per frame. src/hooks/animation.rs:364-384 (use_animation) and
:464-478 (use_spring) allocate controller, from, to and current-id with
Arc::new on every call; only value and state use hook storage, and no handle
has a Drop, so tasks keep writing after unmount. use_transition (:596-611)
runs an always-on effect that calls thread::spawn at :611 while the value
differs from the target, and any hook animation forces a render every frame
(src/app.rs:195-202), so a 300 ms transition at 60 fps accumulates about 18
racing tasks and threads. Fix: hook storage for handle state, cancel in Drop
like KeyframeOwner, drive through scheduler.schedule_timeout.

H6. Vulnerable and unsound dependencies. quick-xml 0.38.3 via syntect/plist
(RUSTSEC-2026-0194, -0195, both high; upgrade to 0.41); time 0.3.41 direct
(RUSTSEC-2026-0009; 0.3.47); lru 0.16.0 direct (RUSTSEC-2026-0002, -0253
unsound; 0.18.2); bytes 1.10.1 (RUSTSEC-2026-0007); crossbeam-epoch 0.9.18
(RUSTSEC-2026-0204). atty is declared at Cargo.toml:23, used nowhere in src/,
and carries an unfixable unaligned-read advisory; delete it. Unmaintained:
bincode and yaml-rust via syntect yaml-load, paste via rav1e. Fix: cargo
update, remove atty, add deny.toml and run it in CI.

## Medium

M1. A stale session-bus address makes the app exit before it draws.
src/app.rs:844-853 auto-enables the screen reader whenever the terminal is
interactive and DBUS_SESSION_BUS_ADDRESS is non-empty; any transport failure
(src/accessibility/platform/unix/transport.rs:16,162-181: unreachable address,
missing org.a11y.Bus, 3 s deadline) becomes a sticky error and publish(...)?
at :527 and :570 ends App::run. This is a recorded decision with a test, but
tmux sessions that outlive the desktop login, sudo -E and minimal desktops all
present that environment, and the builder doc implies auto-enable degrades.
Fix: record explicit versus automatic; log and drop an automatic connection on
failure; keep the fatal path for screen_reader(true).

M2. Dialog remote validation and autocomplete POST every keystroke to a URL
via curl. src/widgets/dialog/http.rs:181,197, called from
dialog/input/live/remote.rs:100 and dialog/autocomplete/live.rs:202. curl is
resolved from PATH with the full parent environment. The implementation is
careful (no redirects, protocol allowlist, 5 s and 64 KiB caps, escaped config
via stdin, killed and reaped), but the README only says HTTP image URLs are
rejected, which reads as no network. Fix: document it, consider a Cargo
feature, consider env_clear with an allowlist.

M3. ThreadedEventLoop livelocks at 512 queued events and stop() blocks on
stdin. src/platform/loop.rs:68-79 push spins on yield_now; :214-217 the input
thread spins while holding the queue mutex the consumers need; :246-255 stop
joins a thread blocked in read; no Drop. Public via src/platform/mod.rs:25,
unused in-tree. Fix: push outside the lock, reuse the socketpair cancellation
from input_receiver.rs, add Drop, or remove the type from the public API.

M4. TokioEventLoop start and stop panic inside an async context.
src/platform/loop.rs:444-474 calls Handle::block_on after try_current
succeeds, which tokio documents as panicking. Public, feature tokio, unused
in-tree. Fix: drop the sync start and stop or return an error in an async
context; document start_async and stop_async.

M5. No SIGTERM, SIGINT or SIGHUP handling. src/platform/unix.rs registers
SIGWINCH only. kill <pid> or a closed SSH session terminates without Drop and
leaves raw mode and the alternate screen. Ctrl+C is fine because raw mode
delivers it as a key. Fix: signal_hook::flag for TERM, INT and HUP that
requests a wake-and-stop.

M6. ReactiveRuntime holds RefCell borrows while running user effects.
src/reactive/runtime.rs:66-88 signal_changed and :124-140 flush_effects hold
self.effects.borrow() across effect.borrow().run(). An effect that calls
RuntimeContext::create_effect, unregister_effect or track_signal panics with
BorrowMutError; create_effect is public and used by Screen. Fix: collect the
handles, drop the borrows, then run.

M7. AdaptiveConfig panics on min_fps > max_fps and divides by zero on
min_fps = 0. src/display/adaptive.rs:89,181,246,258 use clamp, which asserts
min <= max, so a swapped config panics inside AppBuilder::build; with
min_fps 0 and auto-adapt the target walks to zero and :217
(1_000_000_000 / target_fps) panics in the main loop. Fix: validate
1 <= min_fps <= max_fps in with_config and return an error.

M8. Library code prints to stdout and stderr. About 80 println! and eprintln!
sites outside tests, for example src/escape/parser/mod.rs:189 ("buffer
full"), src/core/terminal.rs:84-85,349-359, src/render/reconcile.rs,
src/event/focus.rs:157, src/core/window.rs:569-625. In raw mode these corrupt
the display. Fix: route through the log crate, already a dependency.

M9. Three more C ABI ownership defects. src/ffi/animation.rs:288-290
rtui_animation_manager_add takes ownership with Box::from_raw without saying
so, so create, add, play, destroy is a use-after-free then double free.
src/ffi/app.rs:294-301 rtui_app_builder_build frees the builder even when
build() fails, so the obvious cleanup double-frees; the TypeScript binding
nulls its handle first, the header says nothing. src/ffi/stats.rs:255-293
LOG_CALLBACK is a plain static mut read and written without synchronisation;
setLogCallback on one thread while any renderer logs on another is a data
race. Fix: document consumption or keep caller ownership; consume the builder
only on success; AtomicPtr or RwLock for the callback.

M10. Identity and version metadata disagree. Three repository URLs
(Cargo.toml:9 entrepeneur4lyf, git remote eas4ai,
bindings/typescript/package.json:49 reactive-tui). Crate 0.0.7, TypeScript
package 0.1.0, macros 0.1.0, rtui_version() reports 0.1.0
(src/ffi/mod.rs:104-117). CHANGELOG.md:38 dates 0.0.7 as 2024-XX-XX; no git
tags. CHANGELOG.md:66 and CONTRIBUTING.md:13 claim Rust 1.70+ while the
engine crate is edition 2024 and CI pins 1.95; no rust-version.
CONTRIBUTING.md:35 names an example that lives under tests/integration/.
package.json lists a native/ directory that does not exist. Fix: one URL, one
version, a dated changelog entry, a tag, rust-version.

M11. Dead and vacuous code ships alongside the real thing. tests/integration/
(21 files) is referenced by no mod or [[test]] and never compiles.
src/ffi/{markdown,theme,syntax,platform}.rs and src/ffi/tests/integration.rs
(about 2,900 lines, 63 extern "C" functions) are not declared in mod.rs.
src/hooks/animation.rs:204-223 is a pub unsafe fn storing a raw pointer in a
static mut whose getter returns &'static mut, with zero callers. Of 30
sampled tests and check scripts, 5 assert nothing (for example
tests/production_readiness_test.rs::test_effect_cleanup,
tests/utility_paint_tests.rs "assume success if no panic") and 4 are weak.
Fix: delete or wire in; give vacuous tests an assertion or remove them.

M12. Vendored and duplicated crates complicate every downstream build.
src/backend/crossterm/Cargo.toml:15-16 is name crossterm, version 0.29.0,
indistinguishable from upstream; a consumer with its own crossterm = "0.29"
compiles two packages under one name. Cargo.lock has taffy 0.9.1 (root and
prelude re-export) and 0.13.0 (vendored renderer), vte 0.14.1 and 0.15.0.
The root chooses syntect default-fancy but comrak re-enables onig, so every
consumer compiles oniguruma C (cargo tree -i onig_sys) and needs a C compiler;
this is why the Windows cross-check failed here. Fix: rename the fork, align
taffy, comrak with default-features = false.

## Low

L1. Grid::auto_grid(columns: 0, ..) divides by zero with a non-empty list.
src/layout/grid.rs:287.

L2. Ref::update holds the value mutex while running the closure; calling
current() inside self-deadlocks. src/hooks/refs.rs:43-55.

L3. Throttled and debounced functions and ThreadSafeSignal::update call user
closures under a std Mutex; re-entrant use deadlocks. src/hooks/timer.rs:163,197,
src/reactive/hooks.rs. HookTimer::fire shows the right pattern.

L4. The fallback timer thread wakes every 1 ms for the process lifetime once
any timer hook runs without a scope. src/hooks/timer.rs:369-381.

L5. use_stagger creates a fresh, undriven scheduler each render.
src/hooks/animation.rs:566-584.

L6. core::Terminal::set_title writes the title unfiltered
(src/core/terminal.rs:391-395) while the retained-terminal API rejects control
characters; the child-title parser (src/terminal/parser.rs:424) drops only
0x00-0x1F, so DEL and C1 controls survive if forwarded to the host.

L7. Public Surface string writers store raw control characters and the diff
writer re-emits them (src/core/surface.rs:1031-1275, 2440-2451). Framework
paint paths strip earlier; custom widgets have no gate.

L8. Image decode peak memory is about 512 MiB from a small file: 256 MiB
limit plus an RGBA8 copy. src/widgets/display/image/decoded.rs:12,90-99,191-202.

L9. chafa and viu receive the image path positionally without "--"; a relative
path starting with "-" parses as an option.
src/widgets/display/image/external_renderer.rs:234,269.

L10. which and stty are spawned with plain .output(), bypassing the timeout
and process-group kill every other spawn gets. src/core/terminal.rs:383,
src/core/window.rs:292.

L11. File-explorer remove and copy have a metadata-then-open gap; cap-std
keeps it inside the root, so the worst case is acting on a different in-root
entry. src/widgets/display/file_explorer/worker/operations.rs:273-298,365-386.

L12. Unsolicited OSC 52 responses on stdin become Paste events although the
library never sends an OSC 52 query. src/platform/parser.rs:425-430,476-492.

L13. bufferRelease*Ptr rebuilds a Box<[T]> from a caller-supplied length; a
wrong length frees with the wrong layout.
src/ffi/lib.rs:359-368,406-415,453-462,498-507.

L14. rtui_element_add_child(parent, child) with parent == child frees the
child then writes the parent. src/ffi/component.rs:220-222.

L15. validate_pointer checks only null, alignment and address range
(src/ffi/pointer.rs:110-135); the "validate pointer before using" comments
overstate it, and destroyOptimizedBuffer and destroyTextBuffer have no
double-free guard.

L16. The C header example calls createRenderer with the wrong arity and a
textBufferAppendText that does not exist. include/reactive_tui.h:22,33.

L17. 359 of 4,831 public items lack a doc comment; missing_docs only warns.
pub fn create_test_image (src/core/surface.rs:2014) is not cfg(test)-gated;
the *_legacy control-sequence helpers (src/platform/ctlseqs.rs:233-248) have
zero references.

L18. install_panic_handler, if installed, would tear down the terminal for
panics the library itself catches on worker threads.
src/platform/unix.rs:431-443.

L19. Housekeeping: Cargo.toml:14-20 excludes three directories that do not
exist; 2,247 tracked files match .gitignore; .gitignore:37 un-ignores a log in
a directory that no longer exists; 5 unreviewed decisions sit in .cairn/queue.

## What held up

Read closely and found clean:

- Shutdown and poison tolerance: App::run always reaches backend.shutdown();
  App::drop uses PoisonError::into_inner end to end; updaters, accessibility,
  waker, performance and backend stop in order; two Apps per process work.
- Signals and input: SIGWINCH uses a self-pipe, callbacks run from a cloned
  list off the signal context, installation is ref-counted and the prior
  handler is chained. The input reader is a bounded sync_channel(64) with
  socketpair cancellation, joined on drop.
- Lock discipline: timers, effects, update dispatch, animation runtime and
  signal notification run user callbacks outside locks; the lock order across
  scheduler, waker and subscriptions is consistent.
- External processes: chafa, viu, clipboard helpers, curl and the PTY shell
  get separate argv, unlinked 0600 tempfiles for stdin and stdout, their own
  process group, a deadline, SIGKILL and a reap. chafa and viu output is
  parsed by vt100 into cells and never written raw.
- Images: HTTP sources rejected as documented; encoded input capped at 64 MiB
  by metadata and by take() on the open descriptor; regular-file check on the
  open handle; base64 bounded before decoding; GIF frames counted then decoded
  under limits; cancellable throughout.
- File explorer: root opened once with ambient authority; every read, preview
  and mutation goes through a cap-std Dir with FollowSymlinks::No, a 0700
  staging directory, entry and depth limits, a 16 KiB preview cap; rename uses
  renameat NOREPLACE on a validated leaf.
- Clipboard: read only on paste(), never logged, no OSC 52 emission.
- Embedded terminal: setsid, TIOCSCTTY, signals reset, CLOEXEC, nonblocking
  master; child killed and reaped on drop; output bounded; libghostty replies
  go only to the child; frames reject control characters.
- Environment: all 55 variable reads select capabilities, sizes or backends;
  REACTIVE_TUI_CONPTY_DIR is the only one that picks a file to load and it is
  SHA-256 pinned.
- C ABI: 20 header-to-Rust signature checks matched; string getters handle
  buffer-too-small with a trailing NUL; every returned string comes from
  CString::into_raw with a matching free; rtui_* surface, renderer and
  terminal routes are tracker-validated and double-free safe; foreign.rs
  enforces owner-thread confinement; the cdylib exports nothing without the
  ffi feature.
- Bounds and loops: Surface indexing saturates and bounds-checks; taffy to u16
  clamps; vdom diff depth cap 1000; component expansion cap 128 with
  duplicate-key rejection; router and focus walks carry visited sets; a 0x0
  terminal returns an error.
- Provenance: both vendored crates carry MIT notices and a written origin with
  a hash or revision; the ConPTY bundle has a license and a SHA-256 manifest.

## Not verified

- Windows and macOS: no build ran. Windows-gated code (13 files, ConPTY
  loader, job objects, reparse-point handling) was read for logic only.
- Terminal acceptance for Kitty, Ghostty, iTerm2, Sixel and Orca: the scripts
  are machine-bound (display server, eight PATH tools, patched Kitty, D-Bus)
  and were not re-run.
- No fuzzing, Miri or sanitizer run. Findings are from reading and from the
  checks listed above. Medium and Low items not named in the method section
  rest on the reviewing agent's reading.
- src/backend/crossterm and src/backend/engine (40,000 lines) were read only
  where a finding led there, chiefly the render worker's panic path.

## Suggested order of work

1. Decide the distribution channel (B1). The rest of packaging depends on it.
   The crates-io-release-preparation commitment in progress is this decision.
2. Fix the package boundary and CI (B2, B3).
3. Repair the C ABI (H1, H2, H3, M9) and regenerate the header.
4. Make terminal restoration unconditional (H4, M5).
5. Rework the animation hooks (H5).
6. Update dependencies and add deny.toml (H6).
7. Restore or relink documentation (B4, M10) and rerun all 35 mechanisms.
8. Then the Medium items in the order that matches the user base: M1 for
   tmux and SSH users, M2 for security review of dialogs, M3 and M4 if the
   public event-loop types stay public.
