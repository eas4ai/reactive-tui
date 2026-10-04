# Artifact consistency audit

A Python metadata audit completed successfully. It read review artifacts and frozen source bytes only; it did not import, compile or execute project code.

- All 769 captured Rust paths appear exactly once in the coverage ledger.
- All 68 finding IDs are unique; 64 evidenced findings and 4 qualified risks are indexed.
- Subsystem finding counts match the index: core 19, native 24, terminal 17, widgets 8.
- Indexed primary source paths/line numbers exist within the frozen files.
- Indexed report links resolve to their exact finding headings.
- Absolute local Markdown link targets exist.
- Every captured frozen input matches its recorded SHA-256.
- Coverage totals: 548 production/module/build files; 265 focused deep and 283 structural. Across all 769 files: 265 deep, 312 structural, 192 inventory-only.

This is a report consistency check. It is not a test, compiler, lint, benchmark, visual, ABI or runtime validation result. Findings remain unfixed because the requested work is read-only review.
