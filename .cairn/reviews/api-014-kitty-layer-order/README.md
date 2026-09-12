# Kitty image-layer ordering investigation

API-014 remains failed at receipt `20260912T132336815Z`. No Rust image code,
acceptance assertion, timeout or installed terminal was changed in this action.
Cargo, test, linker, Mesa and Rayon limits remain at 8; hosts ran serially.

## Finding

The installed Kitty 0.45.0 can omit a newly placed image until another repaint.
The original failing capture contains the complete, valid red/blue image data.
Replaying the bytes with different writers, tracing, or extra event-loop activity
usually passes, so passing reruns do not establish a repair.

An independent C sender, with no reactive-tui dependency, sends independently
generated compressed RGBA images. It clears the previous image, lets that screen
render for one second, then sends one new image command. All three reference
runs omit the green/yellow placement at the ordinary screenshot deadline.
Three further controlled runs also omit that placement after an additional
one-second wait. Requesting an X11 repaint then reveals 4,096 green and 4,096
yellow pixels in every run, without any further image transmission.

The installed library also reproduces a concrete ordering defect deterministically:

| Order | Loaded images | Valid placements | Select image layers for this frame |
| --- | ---: | ---: | --- |
| Query, then prepare (host order) | 1 | 1 | false |
| Prepare, then query | 1 | 1 | true |

The decoded pixel hash is identical in both cases. The stale query becomes true
on the next call after preparation. The corrected ordering has been tested at
this native-library boundary; a complete corrected terminal build has not been
built or tested.

## Source connection

Kitty 0.45.0's `prepare_to_render_os_window` asks
`screen_needs_rendering_in_layers` before `send_cell_data_to_gpu` for each normal
window. The latter prepares graphics placements and updates their counts.
The image-layer predicate uses those counts. Finally, `draw_cells` selects the
path that can paint images only when the earlier window-level flag is true.
This stale decision explains why another repaint can recover a valid image.

Primary source locations, inspected against the downloaded v0.45.0 archive:

- [Render preparation](https://github.com/kovidgoyal/kitty/blob/v0.45.0/kitty/child-monitor.c#L748): query at line 748, preparation at line 806.
- [Graphics preparation](https://github.com/kovidgoyal/kitty/blob/v0.45.0/kitty/shaders.c#L595): count update at lines 595-606, predicate at 1055-1059, paint-path selection at 1177-1178.
- [Placement counts](https://github.com/kovidgoyal/kitty/blob/v0.45.0/kitty/graphics.c#L2469): `grman_has_images` reads the counts computed by `grman_update_layers`.

The same ordering and count predicate appear in the inspected v0.47.0 and
master sources. Those versions were not executed; this does not claim that
every environment or newer build reproduces the observed failure.

A concrete proposed host repair is to compute the normal window's image-layer
requirement after preparing its GPU cell/graphics data, as the tab-bar path
already does. Validate it with the independent sender and unchanged library
host captures, including placement removal. This is a proposal, not a completed
host repair. A patched test host would not repair stock terminals used by users.

## Reproduce the native-library result

From this repository, using the recorded installed Kitty build:

```sh
kitty +runpy "import runpy; runpy.run_path('.cairn/reviews/api-014-kitty-layer-order/reproduce-layer-order.py')"
```

`layer-order.out` and `layer-order.err` retain that command's actual output.
Exit 0 means the diagnostic reproduced the stale predicate and verified the
opposite ordering. It is not a passing API-014 acceptance check. The script
retains the native manager object and uses PyDLL to hold the GIL during reads.
It does not modify the installed library or depend on downloaded Kitty tests.

## Retained experiments

`captures/` preserves all completed diagnostic captures, including failures.
`scratch/` preserves the exploratory source and aggregate results under their
original temporary filenames. Those exploratory programs retain their original
`/tmp` paths; the focused reproduction above is self-contained. `frames/` retains
their exact wire fixtures. The independent reference is
`scratch/abi-004-kitty-reference.c` with `frames/reference-frames/`.

- `diagnostic-0`: original Rust sender, missing first image.
- `expose-1`: original Rust sender, missing first image recovered after repaint.
- `reference-0` through `reference-2`: independent sender, missing second placement.
- `reference-control-0` through `reference-control-2`: missing placement remains
  absent after waiting; `stage-1-after-expose.png` then contains the image.
- `state-0` through `state-5`: diagnostic command-syntax errors, not image results.
  The corrected `state2` runs pass and do not explain the original failure.
- Parser partition tests cover 70 fixed/seeded chunkings with correct decoded
  bytes and one placement. Other writer and event-loop experiments are controls,
  not acceptance passes or proof of a repair.

`provenance.json` identifies the committed source, installed host and library,
probe binary and downloaded source archive. `manifest.json` records retained
file hashes. No upstream source archive or host executable is vendored here.
