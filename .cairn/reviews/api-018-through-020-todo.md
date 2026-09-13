# Remaining API commitment work

- Done: API-018 documentation, compatible Props defaults, public examples and supported-API matrix. Expanded formal acceptance and all 34 inherited requirements passed on the corrected source.
- In progress: API-019 inventory and behavior checks for all residual audit concerns; implement and verify any findings.
- Pending: API-020 image-capture timeout cleanup, complete current regression evidence, adversarial review and production self-audit.

Build and test concurrency remains capped at 12. Existing approvals remain in force.

Latest API-019 checkpoint: local scopes, mapping-test discovery, performance
ownership and Updater dispatch have focused passing checks. The approved
gesture coordinate-unit repair passes four boundary tests and nine existing
mouse tests; complete gesture routing and other residual families remain open.

Current inherited regression: API-011 reader verification exposed malformed
cache signals (repaired with focused checks) and a remaining libatspi
use-after-free confirmed by Valgrind. Full acceptance remains failing;
see .cairn/reviews/api-011-cache-wire/review.md for the scope decision.
