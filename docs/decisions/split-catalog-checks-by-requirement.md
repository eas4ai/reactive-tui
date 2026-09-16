# Split catalog checks by requirement

Level: Judged
Decided by: Shawn
Rests on: CAT-001, CAT-002, CAT-003
Would be wrong if: A catalog dependency is omitted from a requirement-specific mechanism.

## Decision

Shawn approved independent shell, motion, and media/documentation mechanisms. Each keeps the complete shared build inputs. A failure in unfinished animation or documentation must not fail the shell requirement.

## Realized by

- 5e0058b6dd9d02716e5cfcbe7ab0b9010f3785f4 fix: isolate catalog acceptance evidence by requirement
