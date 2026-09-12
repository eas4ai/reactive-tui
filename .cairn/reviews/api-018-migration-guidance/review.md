# API-017 migration guide regression repair

The formal API-017 failure at 20260912T190647647Z stopped at the migration
consistency guard. API-018 corrected ownership guidance in the packaged copy
without correcting its canonical C ABI source. The package also needs a different
relative link to the native guide. The original failing receipt and exact diff
are retained; native runtime checks had not run in that attempt.

The canonical C ABI guide now describes the already verified API-001 ownership
repair. The existing guard still compares the complete packaged text against the
two canonical guides. It rebases only the explicit native-components Markdown
link for the package's directory. No API, ABI, or ownership rule changed.

Two safe mutations to the actual packaged file proved that prose drift and an
incorrect relative link still fail the guard. Original bytes were restored in
finally. All local Markdown links in these guides resolve. The complete corrected
API-017 development mechanism passed, including the ABI audit and C/TypeScript
native consumer workflows. Formal evidence must follow the committed repair.

Code graph transport was unavailable. Focused source inspection and Ripwire's
qualified impact/uses checks supplemented it; zero static callers did not cover
the known subprocess invocations. Edit-check passed. Quality-delta exited 2 and
test-gate exited 4, so neither is claimed as passing. Their original outputs and
changed-path findings are retained. The test gate lists six documentation
headings and no indexed tests; the actual API-017 mechanism was executed.
