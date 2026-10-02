# Graphics canvas

Prefix: GFX

src/graphics, behind the `wgpu-graphics` feature, is a demo today: a
spinning cube or torus drawn by one full-screen shader on a hardware wgpu
adapter, or by a hand-written CPU renderer when none is usable
(HybridCubeRenderer), read back at most 800 by 600 pixels and 20 times a
second on an unnamed thread, and shown only as half-block text runs. It is
an object the application owns, not a widget, and no check builds it.
Images already reach the terminal as Kitty, Sixel or iTerm2 pixels through
the painter's image planes (src/layout/paint_tree/suprtui/images.rs,
src/backend/suprtui/graphics.rs), with block-glyph fallback through the
blitters (blitters.md); the terminal's graphics support is read from
environment variables only (src/core/capabilities.rs). The developer's
rulings for this contract are in roadmap.md, section graphics-canvas.

## Observed

(none yet)

## Draft

[GFX-001] The framework MUST provide a `Canvas` widget, placed in an Element tree like the Chart and filling the rectangle its parent allots up to 4096 by 4096 pixels, that draws a scene the application describes: stroked and filled paths of lines, quadratic and cubic curves and arcs, with stroke width, joins, caps and dashes; rectangles, rounded rectangles and ellipses; solid, linear-gradient and radial-gradient paint with opacity, whose colors accept the tokens the layout utility classes accept as well as exact colors; images; text in the font the application supplies, else on Linux the first file of fontconfig's sorted match for `monospace` (`fc-match -s monospace`) that the canvas can load, else a monospace font the crate bundles, with reference scenes drawn in the bundled font; a CellGrid drawn through a glyph atlas in a single instanced draw; and nested transforms and clips.
Falsifier: The reference scene of a listed feature, drawn on the software renderer, is not the same picture (GFX-002) as its checked-in reference image; a theme token in a paint resolves to a color other than the active theme's; on Linux with no application font, text is drawn in a font other than the first loadable file of `fc-match -s monospace`, or in no font when none loads; or drawing a CellGrid on the hardware adapter issues more than one draw call.
Mechanism: canvas-scenes
Rationale: The developer ruled on 2026-09-27 that the canvas's features are designed for the GPU and are not reduced for weak hardware.
Status: Agreed 2026-09-27

[GFX-002] The canvas MUST render on a hardware wgpu adapter (Vulkan, Metal or DX12, discrete or integrated) whenever one is usable and otherwise on the software renderer, where a software adapter such as WARP or lavapipe counts as none; the software renderer MUST draw every feature GFX-001 lists, and for every reference scene the two MUST draw the same picture: over the whole picture the mean difference per channel is at most 1 of 255, and no block of 8 by 16 pixels differs in mean color by more than 8 of 255 in any channel; the canvas MUST report which renderer draws, with the adapter's name or the reason for the software renderer.
Falsifier: On the Linux development host, the macOS test host or the Windows test host, a canvas test renders on the software renderer or skips although the host has a hardware adapter; a reference scene drawn on the hardware adapter and on the software renderer is not the same picture; or the reported renderer is not the one that drew.
Mechanism: canvas-hosts
Rationale: The developer ruled on 2026-09-27 to keep the software renderer as the architecture has it, so a machine without a usable GPU still draws the canvas, more slowly.
Status: Agreed 2026-09-27

[GFX-003] Canvas rendering MUST run on a named worker thread (`rtui-canvas-*`) that owns the adapter, the device and the software renderer and publishes each finished frame through the AppWaker; the App thread MUST never wait for a GPU submission, a readback or a software render; a scene submitted while the worker is busy MUST replace any scene still waiting, and the worker MUST impose no interval of its own between frames.
Falsifier: A GPU submission, readback or software render runs on the App thread; the thread that renders is not named `rtui-canvas-`; two scenes wait at once; or, at 40 by 12 cells with a scene submitted every 16 ms for one second, fewer than 50 frames are rendered.
Mechanism: canvas-scenes
Status: Agreed 2026-09-27

[GFX-004] On the Windows test tablet's integrated GPU (Intel Iris Xe) in a release build, the canvas MUST render and read back the reference animation scene at 1920 by 960 pixels (240 by 60 cells of 8 by 16 pixels) within 16.6 ms, and turn it into block glyphs for 240 by 60 cells within 16.6 ms, each at the 95th percentile of 300 frames on the hardware adapter.
Falsifier: Either 95th percentile exceeds 16.6 ms on the tablet's hardware adapter.
Mechanism: canvas-hosts
Rationale: The developer ruled on 2026-09-27 that the Windows tablet, a Dell XPS 13 9315 2-in-1 with an i7-1250U and Iris Xe graphics, is the lowest common denominator; speed bounds apply to the hardware adapter, not the software renderer.
Status: Agreed 2026-09-27

[GFX-005] The canvas MUST show its frame as Kitty graphics when the host supports them, otherwise as Sixel, otherwise as the block glyphs BLT-002 chooses, and an application or environment override MUST replace the choice; a pixel frame MUST replace the previous one in place, never clearing the screen or rewriting a cell outside the canvas; with Kitty on the same machine, frames MUST travel through shared memory; and a frame not yet written when a newer one is ready MUST be dropped, so frames never queue.
Falsifier: A host whose capability report has Kitty graphics receives Sixel or block glyphs without an override, one with only Sixel receives block glyphs, or an override is ignored; the bytes between two canvas frames contain `ESC[2J` or write a cell outside the canvas; with shared memory accepted, a Kitty frame is sent any other way; or, with a slow writer, more than one canvas frame waits to be written.
Mechanism: canvas-output
Rationale: The developer confirmed on 2026-09-27 that the GPU draws at the App's frame rate and that slower links and Sixel drop frames rather than fall behind.
Status: Agreed 2026-09-27

[GFX-006] On Unix, the startup queries of INP-011 MUST also ask whether the terminal accepts Kitty graphics sent directly and through shared memory, and the backend MUST read Sixel support from the device-attributes reply (attribute 4); a terminal that answers neither MUST keep the environment detection it has today; no graphics reply byte MAY reach the App as an event, and the replies MUST arrive within INP-011's 200 ms wait.
Falsifier: In a pseudo-terminal that answers the Kitty graphics query with OK, the capability report has no Kitty graphics, or shared memory when only the direct query was accepted; one whose device-attributes reply lists 4 reports no Sixel; one that answers device attributes without 4 and sets `TERM_PROGRAM=WezTerm` loses its Sixel; or a graphics reply byte reaches the App as a key.
Mechanism: canvas-output
Status: Agreed 2026-09-27

[GFX-007] A fault MUST never end the App or panic: when the adapter cannot be used, the device is lost or a readback fails, the canvas MUST switch to the software renderer for the rest of its life and report why; when the software renderer also fails, the canvas MUST show a message in its own area.
Falsifier: An injected adapter, device-loss or readback fault ends the App, panics or leaves the canvas blank with no message, or rendering continues on the hardware adapter after a device loss.
Mechanism: canvas-scenes
Status: Agreed 2026-09-27

[GFX-008] The widget catalog's Motion page and the animation showcase's Shader page MUST draw the cube and the torus as canvas scenes, on the hardware adapter when one is usable and on the software renderer with `--cpu`, and src/graphics MUST keep no renderer other than the canvas's two.
Falsifier: Either page draws the cube or the torus other than through the Canvas widget, `--cpu` does not select the software renderer, or src/graphics holds another renderer.
Mechanism: canvas-scenes
Status: Agreed 2026-09-27

[GFX-009] On a terminal that takes pixels, the App MUST NOT wait in `present` while a canvas's picture is made ready: its pixels copied, written to shared memory, or encoded as base64 or Sixel. That work MUST run on a thread named `rtui-picture-*` or on the canvas's `rtui-canvas-*` worker, and until a new picture is ready the terminal keeps showing the last one; a picture that arrives while another of the same canvas is being made ready MUST replace any picture still waiting, so at most one waits; a picture made ready for cells that no longer show its canvas when it is written, because the canvas moved or something now covers it, MUST NOT be written, and the canvas's newest picture is made ready again for the cells as they are. In a release build that writes the terminal's bytes to memory, with a new picture of 1920 by 960 pixels (240 by 60 cells) submitted every frame, the App's wait in `present` MUST stay within 16.6 ms at the 95th percentile on the Windows test tablet, the macOS test host and the Linux development host, with Kitty graphics in the command, with Sixel, and on macOS and Linux with Kitty through shared memory.
Falsifier: A canvas's picture is copied, written to shared memory or encoded on a thread whose name starts with neither `rtui-picture-` nor `rtui-canvas-`; two pictures of one canvas wait to be made ready at once; a picture is written over a cell that no longer shows its canvas; or, on one of the three hosts in a release build that writes the terminal's bytes to memory, over a run in which at least 30 pictures of 1920 by 960 pixels reach the terminal, the 95th percentile of the App's wait in `present` exceeds 16.6 ms with one of the outputs the host offers.
Mechanism: canvas-pictures
Rationale: Measured on 2026-09-29 (backlog item pixel-output-holds-the-app): with a picture of that size the App waited in `present` for a median 129 ms with Kitty and 479 ms with Sixel on the tablet, so a key waited up to half a second; the developer ruled on 2026-09-27 that the tablet is the floor and that features are not cut for weak hardware.
Status: Agreed 2026-10-01
