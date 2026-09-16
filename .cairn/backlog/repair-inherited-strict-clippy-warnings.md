# Repair inherited strict clippy warnings

Surfaced from: CAT-001
Outside because: These are existing framework/dependency style warnings, not defects needed to satisfy catalog requirements.
Captured: 2026-09-16T14:06:50.520Z

Catalog verification found an unknown nested clippy namespace and four io_other_error warnings in the bundled crossterm source, plus collapsible_else_if in wizard.rs. These files were unchanged by widget-catalog. Strict catalog clippy passes with no-deps and the inherited wizard lint allowed.
