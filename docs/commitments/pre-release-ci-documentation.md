# Commitment: pre-release-ci-documentation

Status: Agreed 2026-09-14
Requirements: RID-001, RID-002, RID-003

## Activation

Shawn approved this existing roadmap commitment on 2026-09-16 by answering
`ok` to escalation `cat-001-cat-002-cat-003-rid-001-rid-002-rid-003-loop-087`.
The catalog requirements pass and its closing review is clean. Preserve its
pause history and the graphics evidence. This approval does not promote
unrelated backlog work or authorize publication.
See [the decision](../decisions/start-the-approved-pre-release-ci-and-documentation-commitment.md).

## Deliverable

Run the maintained suites on every supported operating system for main-branch
changes, schedule advisory checks, relink all tracked documentation and Cairn
inputs, and remove stale repository housekeeping rules.

## Boundaries

This commitment changes GitHub workflows, README and include links, manual and
Cairn source documents, mechanism declarations, ignore rules, and stale tracked
artifacts. Developer review controls the decision queue. It does not create
release archives or publish anything.

Done-when: RID-001 through RID-003 pass; push, pull-request, and scheduled event
fixtures select the required jobs; every local link and mechanism input exists;
all mechanisms have fresh evidence; the developer has reviewed the queued
decisions; final review finds no stale ignore or package rule.
