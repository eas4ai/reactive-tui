# Rust API audit evidence

These are observational probes, not acceptance tests or Cairn receipts. Passing
probe assertions confirm defects at the revision in [the report](../../api-audit.md). When a
repair lands, convert the relevant observation into an expectation of correct
behavior under the new commitment; do not preserve a defect to keep these green.

Run from the repository root on the audited Linux toolchain:

```sh
python3 -B docs/analysis/rust-api-audit/run-probes.py
cargo check --locked --no-default-features
```

The first command builds the current native library and a dependency needed for
screen parsing, then runs eight bounded probes. The second failed at the audited
revision; its output is in no-default-features.log. probes.log records the final
eight reproductions. compile.log includes only unused-method warnings caused by
the private focus-manager harness. package-list.log is the successful Cargo file
listing, not proof that packaging/publishing succeeds.

No production source is modified. The report and evidence were produced after the binding commitment completed.
They are now tracked as baseline observations for rust-api-remediation. Raw logs
retain their original output, including trailing blank lines.
