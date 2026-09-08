# Registry cache isolation review

## Scope and mechanism plan

Exercise independent registries with equal registration counts and different
name/type assignments on one thread. Exercise clones made before registration,
clones mutated after lookup, clear/re-register and cross-thread publication.
Use real type-erased instances and check their TypeId, not a cache counter.
Keep the existing constructor reentry and registry concurrency acceptance.
Record the fresh full-suite assessment separately from passing requirements.
