# Repair Kitty image-layer selection after placement changes

Surfaced from: API-014
Captured: 2026-09-12T14:08:20.984Z

Kitty 0.45.0 queries whether a normal window needs image layers before preparing graphics placement counts. The installed native library deterministically reports false with a loaded image and valid placement, then true after preparation. An independent C sender reproduces missing images; an extra wait does not recover them, but an X11 repaint does without retransmission. Evidence and a self-contained diagnostic are in .cairn/reviews/api-014-kitty-layer-order/. Proposed host repair: prepare graphics data before selecting the image-layer path, then validate a full isolated host build against unchanged pixel checks. Repairing the terminal host requires an explicitly agreed scope extension; no host source or installed binary has been changed.
