# Documentation retention

Status: Agreed 2026-09-14
Prefix: DOC

The developer chose to keep the repository's durable documentation limited to
Cairn contract artifacts. Historical product documentation and diagnostic
captures remain available in Git history at `5c069365`.

[DOC-001]
The tracked `docs` tree MUST contain files only under `docs/spec`,
`docs/commitments`, and `docs/decisions`. Those three paths MUST remain
trackable while every other path under `docs` remains ignored. The `.cairn`
records MUST remain present and tracked.
Falsifier: Git tracks a document outside the three retained directories, ignores
a document inside one of them, or the retained roadmap, glossary, overview,
current specification, current commitment, decision record, or Cairn mechanism
is absent.
Mechanism: `python3 scripts/check-documentation-retention.py` inspects tracked
paths and probes the ignore boundary without creating files.
