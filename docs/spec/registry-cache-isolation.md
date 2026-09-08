# Registry name lookup isolation

Status: Agreed 2026-09-08
Prefix: CCH

The developer confirmed registry cache isolation followed by a fresh full-suite
assessment. Public registration and component construction behavior is retained.

[CCH-001]
Named component lookup MUST use the registrations of the requested registry, independently of other registries used on the same thread.
Fresh registries MUST NOT reuse names or component types from dropped registries.
Falsifier: alternating registries with equal registration counts resolves the wrong component type, misses a registered name, or exposes another registry's name.
Mechanism: scripts/check-registry-cache-isolation.sh, independent-registry lookup regressions.

[CCH-002]
Cloned registries MUST observe completed registration, clear and re-registration operations through any clone, including across threads.
Named lookup MUST preserve constructor reentry and the existing registry concurrency requirements.
Falsifier: a clone returns stale names after a completed shared mutation, a constructor reenters a held map lock, or an inherited requirement fails.
Mechanism: scripts/check-registry-cache-isolation.sh and inherited acceptance mechanisms.
