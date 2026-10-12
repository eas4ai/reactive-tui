# Presentation

Prefix: PIP

`present` in src/backend/suprtui.rs sends the frame to the render worker
over a rendezvous channel and waits for the reply, which the worker sends
only after layout, paint, write and flush; the App's event loop is blocked
for the whole terminal write. The manual (manual/rendering-and-backends.md,
Limits) says the App keeps the last presented geometry until a frame is
presented successfully. The rasterizer is in rasterizer.md.

## Observed

(none yet)

## Draft

[PIP-001] `present` MUST return the frame's geometry as soon as layout and paint finish; the worker MUST write and flush the bytes afterwards; a present MUST wait for the previous frame's flush before submitting, so at most one frame is in flight.
Falsifier: `present`'s wall time includes the flush of the frame it submitted (measured with a slow writer), or two frames are in flight.
Mechanism: present-pipeline
Status: Agreed 2026-09-22

[PIP-002] A flush failure MUST be reported by the next `present` or by `shutdown`, MUST force a full repaint of the next frame, and the App MUST keep the geometry of the last frame whose flush was acknowledged.
Falsifier: A failed flush is never reported, or the frame after a failure is emitted as a diff.
Mechanism: present-pipeline
Rationale: This changes the manual's "keeps the last presented geometry until a frame is presented successfully" to "until the next present reports the previous frame's failure"; Consequential, decided before building.
Status: Agreed 2026-09-22
