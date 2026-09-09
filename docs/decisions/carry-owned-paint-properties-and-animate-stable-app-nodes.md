# Carry owned paint properties and animate stable App nodes

Level: Judged
Decided by: Codex
Rests on: API-010
Would be wrong if: Component props are replaced, animation state leaks between Apps or survives removal, idle Apps spin, transformed input bounds disagree with painted cells, or the public NodeSpec shape changes.
History: The recorded clipboard deadline reversals concern native process startup. This decision preserves the existing App ownership and acknowledged-frame contracts and will be checked with independent terminal frames.

## Decision

Store explicit styles and gradients in the Element metadata introduced by API-005, independently of typed component props, and propagate them to component output. Preserve public NodeSpec construction; carry private paint properties alongside its preorder tree into the supported painter. Explicit StyleBuilder values form the base and utility classes override that base; cache only parsing from the default base. Each App owns animation and transition state by parent-scoped rendered keys, using its existing bounded frame deadlines and removing state when nodes disappear. Sample styles before painting and register the resulting geometry only after presentation. Reuse the existing gradient stops and animation easing/property definitions. Sample gradients in local cell coordinates; blend opacity into underlying cell colors. Translations use terminal cells, scaling changes cell placement and box extent, and rotation moves upright glyphs to rounded cell positions rather than rotating host glyph bitmaps. Clip whole wide graphemes and keep transformed hit bounds aligned with the rendered box. Record terminal approximations and test intermediate frames, keyed continuity, resets, removal and App isolation. Preserve existing animation APIs; the keyframe and relative-value repairs remain separately verified under API-013.

## Realized by

4603bdc Paint gradients and owned animation frames through App

## Worker boundary

Taffy 0.9.1 stores compact lengths in a raw tagged pointer and its Style is not Send or Sync. Keep those values on the thread that evaluates layout. Element metadata stores an owned serialized style snapshot using Taffy's existing serde feature; the worker reconstructs the style and reports invalid values as layout errors. Send the owned Element tree to the worker, then build the private paint specification there. No unsafe Send/Sync implementation is introduced.
