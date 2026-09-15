# Review: example-cleanup

commit: e7f32ae9
findings:
  - resolved: the public example inventory and documentation advertised six programs that the developer found broken in Kitty; those sources and invitations are removed.
  - resolved: renderer, wakeup, and embedded-terminal checks depended on three deleted app sources; their narrow programs now live under `tests/runtime_probes/` with private target names and the same PTY assertions.
  - resolved: the review found that the deterministic embedded-terminal PTY probe does not reproduce or refute the reported Kitty segfault; the unresolved crash itself is preserved in `.cairn/backlog/reproduce-the-embedded-terminal-kitty-crash.md` rather than misreported as fixed by EXC-001.

## Scope examined

I re-read EXC-001, its falsifier, the commitment boundaries, the mechanism
declaration, the passing receipt and captured output, and the full structural
change from `91c604f8` through `e7f32ae9`. I inspected the public example
directory, all renamed mechanism inputs, the README and manual changes, the
renderer and embedded-terminal specification wording, and the dialog check that
lost its compile-only demo step.

Tilth reports exactly one file under `examples/`: `gradient_blocks.rs`. The
committed EXC-001 run also scanned tracked text for the six deleted basenames and
compiled that example with locked Rust 1.91 dependencies. The scan intentionally
allows the current contract, validator, historical evidence, reviews, and Cairn
backlog to name deleted programs. It does not allow README, manual, manifest,
mechanism, or ordinary script references.

## Mechanism attacks

The validator suite uses isolated temporary trees. It rejects an extra removed
source, a missing gradient source, a stale tracked command, and a failing compile.
The failed-compile case also makes removing the compile invocation fail the suite.
It accepts an unrelated `rtui_dialog_engine_create` API name so the basename scan
does not confuse retained dialog API symbols with the deleted app. I ran the
suite first against the checker scaffold and observed all cases fail because the
validator did not exist. I also observed the Cairn baseline fail before the
implementation. Seven validator cases and the corrected repository then passed.

The three relocated PTY programs preserve the framework checks rather than the
deleted apps. The wakeup probe observed a background signal, idle sleep, resize,
quit, and terminal restoration. The embedded-terminal probe observed child input,
background redraw, resize, Ctrl+C forwarding, Ctrl+Q host exit, worker failure,
and cleanup. The renderer probe observed state change, Escape and Ctrl+C exit,
error and panic restoration. The dialog gate passed 25 lifecycle, 41 dialog, 12
modal, and 2 HTTP behavior tests without relying on a demo that merely compiled.

The combined wakeup gate's runtime and PTY portions passed, but its inherited
repository-wide specification lint ended nonzero on ten pre-existing
multi-obligation sentences outside EXC-001. I do not count that command as a pass.
Cairn remains authoritative about whether those inherited mechanism inputs need
fresh review or repair before this commitment is Done.

## Graph and language-server review

GitNexus classified every deleted Rust type or entry point and both modified
Python checker entry points as LOW risk. It found no affected production
execution flow. Its staged Markdown mapping reported several same-named headings
in untouched manuals; the staged Git status and Tilth structural diff confirmed
the actual 27-file implementation scope. Rust-analyzer reloaded the changed Cargo
manifest and resolved both moved `Counter` types at their new probe paths. The
locked builds exercised all three moved Rust programs.

Ripwire's commitment-wide review found the renderer PTY helper's ten dependants
and no unaccounted production blast radius. Its DMM score was 0.585; the
complexity component is distorted by moving existing probe bodies to new paths,
which Ripwire documents as new symbol identity. The current quality delta has no
tracked-code finding; its only row is generated `.gitnexus/meta.json` content.

No unresolved finding belongs to EXC-001. The Kitty crash remains explicitly
unfixed and captured outside this commitment. No executable code changed during
this review.
