# Use an owned temporary machine-store certificate on dedicated Windows test desktops

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: The fixture alters an existing certificate, runs on a desktop without explicit dedicated mode, requires interactive privilege elevation, fails to remove its certificate, or lets the untrusted case succeed.
History: Native user-store certificate import repeatedly timed out before the HTTPS probe; adding force did not fix it. Earlier platform reversals require actual native trust and cleanup evidence. This remains Judged because the fixture runs only with explicit dedicated-desktop mode and owns a freshly generated certificate.

## Decision

Import the freshly generated HTTPS fixture certificate into the dedicated Windows machine Root store using certutil under the runner existing rights. Remove only its exact fingerprint in a finally block before the untrusted probe. Retain command deadlines and fail if import or cleanup fails. Require --dedicated-desktop for native desktop widget verification and Windows HTTPS trust setup; pass it from the disposable hosted workflow. Do not change the application TLS verification behavior.

## Realized by

Implementation: `scripts/check-dialog-http.py`, `scripts/check-widget-platforms.py`.

Behavior checks: `.cairn/reviews/api-011-windows-34505587248-https.out`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
