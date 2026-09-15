# Forward only curl proxy and certificate environment

Level: Judged
Decided by: Codex
Rests on: XIS-001,own-bounded-curl-requests-for-dialog-validation-and-autocomplete
Would be wrong if: A supported HTTP or HTTPS proxy or custom certificate store stops working, or curl still receives unrelated parent environment values.

## Decision

Clear the environment for both the curl version probe and request. Copy PATH; lowercase http_proxy, https_proxy, all_proxy, and no_proxy; uppercase HTTPS_PROXY, ALL_PROXY, and NO_PROXY; CURL_CA_BUNDLE, SSL_CERT_FILE, and SSL_CERT_DIR; and Windows PATH resolution values PATHEXT, SystemRoot, and WINDIR when present. Do not forward HTTP_PROXY, home and curl-config paths, netrc controls, TLS key logging, debug variables, locale values, or unrelated application state. Keep --disable so curl cannot load user configuration. Apply one helper to both commands so the probe and request have the same boundary.

## Realized by

(none yet: recorded, not built)
