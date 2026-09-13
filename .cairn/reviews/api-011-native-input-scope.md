# API-011 native input scope revalidation

The Windows ConPTY record and the macOS/Windows widget records were produced at
`1da03726fc3d99d1f467d7c22342b955c411a406`. Their original digests included the
whole `tests` tree. Python-only Orca and Linux image-capture harnesses cannot be
compiled into, imported by, or executed by those native jobs, but changes to them
invalidated all three records.

The `native-v2` scopes retain the manifests, build script, complete library and
macro sources, every Rust fixture each job compiles, every helper each job runs,
the workflow, and the ABI baseline used by ConPTY. The widget scope discovers all
tracked `tests/api_widget_behavior/*.rs` files from `HEAD` and rejects a new
untracked Rust fixture. Checker scripts remain declared by the Cairn mechanism, so
changes to checking logic still require a fresh API-011 check.

Before updating record metadata, Git reported no changed file between the recorded
commit and `7922483fcb5c8e197b901db3d1cbf3e54940cafb` in either native-v2 scope.
Independent digests at both commits matched:

- ConPTY: `25039b687af03a68d38268e0dd23fff18a20328f38cbb6107e201bbbfffc83ee`
  across 579 tracked files.
- Native widgets: `15386d14f332cd388cf2485b10895c9bfbe20b10308eb3d97dbec439332aeb7b`
  across 620 tracked files.

The retained native output bytes and their SHA-256 entries are unchanged. This is
metadata revalidation against a narrower dependency scope, not a new native run.
