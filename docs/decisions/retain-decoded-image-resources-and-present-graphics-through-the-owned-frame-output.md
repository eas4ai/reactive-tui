# Retain decoded image resources and present graphics through the owned frame output

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014 API-016
Would be wrong if: Image controls cannot share actual layout clipping and frame ownership without changing the public Image data shape or emitting untracked terminal output.
History: The recorded API reversals require preserving public contracts and making ownership explicit. Keep the public Image data fields and the existing SuprTUI worker; do not replace either with a new renderer. This implementation choice remains Judged and does not reduce any host or protocol claim.

## Decision

Make Image and ImageBuilder produce a private retained App component that owns decoded resources, updates on source changes and reports loading or decoding errors visibly. Keep the public standalone Image rendering methods. Carry decoded image paint metadata through component expansion and the existing layout bridge; image clipping, placement and fallback use the same presented geometry as other elements. The SuprTUI worker owns protocol transmission and cleanup inside its checked frame output, including movement, hiding, removal, resize and shutdown. Never encode graphics escapes as text nodes or write outside the backend owner. Reuse the corrected image decoding and protocol serializers, and repair the parallel platform and Surface adapters against the same behavior. Keep each loading worker bounded to the active request and the newest pending request, discard stale results, notify App through its existing wake path and release the worker on unmount. Retain format hints and authored classes in the builder. Actual host and protocol checks remain necessary for acceptance.

## Realized by

The retained worker and ImagePaint metadata now carry decoded resources through
the existing component bridge and painter. The owned SuprTUI output supports Kitty
transmission, masking, movement, source updates, removal, resize and shutdown;
actual App and backend captures exercise those paths. Linux Kitty and Ghostty
screenshots verify pixels, updates, movement and removal. Sixel, inline, Chafa and Viu now also have App host captures; the platform adapter
shares decoded pixels and corrected serializers. Animated GIF playback also passes timed App and host checks. Surface and DiffWriter now share graphics output and have actual host captures, including failed-write recovery; remaining platform coverage is still pending. See the image section of docs/widget-acceptance.md and the current
commitment review for the editing-check results and remaining work.
