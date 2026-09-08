# Separate component name caches by registry identity

Surfaced from: REG-001
Captured: 2026-09-08T03:30:53.000Z

During REG-001 ownership review, get_cached_type_id was observed to share one thread-local cache across all ComponentRegistry values, using only a numeric version. Two different registries at the same version can therefore reuse another registry name map. ComponentRegistry::clone also copies its cache-version counter while sharing maps, so invalidation is not shared. Add isolated cross-registry and clone-invalidation regressions before a future cache repair. This is outside the confirmed concurrency-hang commitment and was not changed.
