# Commitment: wgpu-graphics

Status: Agreed 2026-09-16
Requirements: GPU-001, GPU-002, GPU-003, GPU-004, GPU-005

## Deliverable

Add an optional wgpu graphics canvas to Reactive-TUI. Demonstrate it with a
shaded, continuously spinning cube in the widget catalog, sized to the
available terminal viewport. Keep a CPU fallback and measure the complete
render-to-terminal path before making performance claims.

## Boundaries

This commitment may add the optional graphics module, its canvas integration,
feature-gated dependencies, cube shaders, catalog demonstration, focused tests,
measurement tooling, Cairn mechanisms, and runnable documentation.
Existing terminal output remains owned by the current renderer.

It does not replace widget layout or the host renderer, create a separate
window, introduce a scene/game engine, port Three.js, require terminal image
protocols, change existing APIs incompatibly, or guarantee a frame rate.
Platform claims require evidence on the named platform and host terminal.

## Activation

This is a queued next-iteration commitment, not authorization to implement
wgpu now. The current catalog remains active, including its open full-width,
spacing, colored column-span, and wireframe-quality findings. Creating this
record does not close those findings or reorder the existing release work.
Shawn confirmed the written specification on 2026-09-16. Implementation still
requires an implementation plan and explicit roadmap activation.

Done-when: GPU-001 through GPU-005 have fresh passing evidence, the GPU path
has run on a real adapter, fallback and cleanup checks pass, reproducible
measurements are recorded, and final review finds no unresolved requirement
defect. Every mechanism must demonstrate a violating case before its pass
counts as acceptance. Specification approval does not claim implementation
or acceptance evidence.
