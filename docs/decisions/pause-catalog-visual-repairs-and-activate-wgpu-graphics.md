# Pause catalog visual repairs and activate wgpu graphics

Level: Judged
Decided by: Shawn
Rests on: GPU-001, GPU-002, GPU-003, GPU-004, GPU-005, CAT-001, CAT-002
Would be wrong if: The catalog findings are erased or treated as completed, or unrelated release work is silently dropped.

## Decision

Shawn explicitly directed pausing the catalog and cube fixes to implement the approved wgpu commitment first. Move Current to wgpu-graphics, preserve the catalog review findings for resumption, and keep the existing release work pending. GPU rendering can improve cube quality; terminal width and spacing still require layout repairs.

## Realized by

- 62f0a93a4ca5c61b8f591c160ad930eab455a6e7 docs: activate wgpu graphics and pause catalog visual repairs
