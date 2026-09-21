# Render external image tools into owned clipped cells off the App thread

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014
Would be wrong if: Chafa or Viu cannot produce bounded symbol output that preserves their advertised sizing and colors in the App frame.
History: The existing image decisions retain all advertised modes and require one owned frame output; external modes currently fall back to internal ASCII in App.

## Decision

Run explicitly selected Chafa and Viu in the retained image worker after decoding, with replaceable requests for layout and option changes. Force static symbol output, parse captured ANSI into a bounded cell grid using the existing vt100 dependency, and compose styled cells through normal App layout and clipping. Never forward child control sequences to the host. Keep process timeout and output limits, cancel obsolete work, surface tool failures, and prove source changes, resize and removal with real executables.

Automatic mode retains graphics priority. When environment detection has no
graphics protocol, the worker probes Chafa then Viu with the same cancellable
process runner and caches the choice for that retained worker. Missing tools
select internal ASCII. Capability probes never read App input.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`live/worker.rs` reuses decoded sources for layout changes and executes selected
external modes with generation-aware cancellation. `live/cells.rs` parses bounded
captured output and creates clipped styled runs. The existing vt100 test dependency
is now a normal dependency; no new package version was introduced. `ExternalRenderer`
forces static symbol output and applies Viu aspect/background/quality before invocation.

Real Chafa 1.18.1 and Viu 1.6.1 Kitty captures pass source update, movement,
enlargement and removal. A forced ASCII output fails the same color check. Worker
fixtures verify retained pixels after file deletion and child cancellation/reaping.
Auto selection also has a passing GNOME Terminal/Chafa pixel capture and isolated
priority/missing-tool tests. The broader image acceptance remains in progress.
