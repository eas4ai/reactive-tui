# Windows terminal runtime

Windows terminal sessions use Microsoft's redistributable ConPTY implementation,
version 1.24.260710001. Distribute the `reactive-tui-conpty` directory beside your
application executable. From this source checkout, install it without downloading
anything:

```sh
python scripts/install-conpty-runtime.py path/to/application-directory --arch x64
```

Choose `x86`, `x64`, or `arm64` to match the application executable. The installer
includes all three native console hosts so Windows can select its own architecture
when the application runs under emulation. Include the runtime's MIT license.
For a Cargo example, the application directory is `target/debug/examples`; for a
normal debug binary it is `target/debug`.

An application may instead set `REACTIVE_TUI_CONPTY_DIR` to an absolute directory
containing the same bundle. The loader verifies each file against hashes compiled
into the library, holds the files against replacement while sessions use them,
and loads the DLL by absolute path. Missing, damaged or mismatched files cause an
actionable launch error. Updating the bundle requires updating the library's pin
and verifying native behavior together.

The native OS implementation on the Windows Server 2022 verification runner
leaks one handle for each bare pseudoconsole create/close cycle. The pinned SDK
has no per-cycle growth in the same diagnostic. The library therefore does not
fall back to that implementation when its runtime is missing. Native Windows x64 probes now exercise input, resize, restart,
backpressure, child/descendant cleanup and the App workflow with both GNU and
MSVC toolchains. The failed-launch handle probe initializes independent Windows
process-launch bookkeeping before measuring PTY failures; its deliberate-leak
control still fails. See [the widget inventory](widget-acceptance.md) for evidence
and the remaining committed verification obligations.

Package: [Microsoft.Windows.Console.ConPTY](https://www.nuget.org/packages/Microsoft.Windows.Console.ConPTY/1.24.260710001).
