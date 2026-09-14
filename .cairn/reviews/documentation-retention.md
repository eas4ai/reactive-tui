# Review: documentation-retention

commit: dac8d6e8
findings: []
Status: Complete

## Scope and boundary

Compared the completed Rust API candidate at `5c069365` with the cleanup
candidate. The change removes 420 tracked paths outside `docs/spec`,
`docs/commitments`, and `docs/decisions`; adjusts `.gitignore`; adds the DOC-001
contract, mechanism, decision, and evidence; and removes dead documentation
inputs from retained historical mechanisms. No file under `src`, `bindings`,
`tests`, `examples`, `reactive-tui-macros`, `include`, or `benches` changed.

The retained documentation inventory contains 164 tracked files: 14
specifications, 10 commitments, and 140 decisions. The diff deletes no file
under those directories and no `.cairn` artifact. Removed material remains in
Git history at `5c069365`.

## Failure demonstration and corrected case

The committed failure receipt at `20260914T130008072Z` ran the actual retention
mechanism before deletion. It rejected the repository and listed the 420 legacy
documentation paths. This establishes that the mechanism fails when tracked
documentation crosses the agreed boundary.

The corrected receipt at `20260914T130949655Z` passed against the committed
cleanup candidate and reported 164 tracked Cairn documents. Independent review
also confirmed that a path directly under `docs` is ignored while probe paths
inside each retained directory are trackable. `git diff --check` passed.

## Limits and self-audit

The mechanism verifies repository retention and ignore behavior. It does not
claim that historical links inside Cairn records resolve in the current tree;
those records describe completed commitments whose removed supporting documents
remain available at the named historical commit. Historical mechanisms no
longer declare removed documents as live inputs.

Ripwire quality-delta and test-gate reported no working-tree code changes or
test obligations. That comparison is against the committed HEAD, so it supports
only the observation that the review added no code changes. DOC-001's committed
failure and passing receipts are the acceptance evidence.

No runtime code, dependency, public API, native ABI, generated source, terminal
behavior, or build configuration changed. No finding remains.
