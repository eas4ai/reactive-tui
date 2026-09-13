# Reconcile residual claims to the shipped public surface

Level: Judged
Decided by: Codex
Rests on: API-019, docs/api-audit.md, docs/residual-api-inventory.md
Would be wrong if: The retained Theme inheritance leaks across instances, documented editor selection behavior fails, transition metadata changes terminal painting, or a current public API actually promises App theme context, undo and redo, or GPU rendering.
History: Earlier API remediation work found several compiling metadata and helper surfaces described as integrated behavior; those reversals require executable checks plus explicit limits instead of treating prose or fields as implementation.

## Decision

Verify the public Theme object's parent inheritance, child override, mutation invalidation, CSS resolution, and isolation between independent instances. Do not invent an App theme provider that is absent from the public API. Verify TextEditor and SyntaxEditor selection replacement and Unicode movement; document that they expose no undo or redo contract. Retain TransitionConfig animation ID, custom properties, and hardware preference as compatibility metadata, remove integration claims from their documentation, and verify the terminal transition painter independently of those fields.

## Realized by

857f7b772ce808cb08b88629b06af35b55584295 Complete residual API behavior coverage
