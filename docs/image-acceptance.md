# Image acceptance

API-014 uses `scripts/check-api-images.py`. It checks the existing image
implementation and records fresh host captures; declaring the mechanism alone
does not establish acceptance. Current results live in `.cairn/evidence/API-014`.

| Route | Required behavior and evidence |
| --- | --- |
| Widget decoder | File, base64, data URL, encoded memory and raw RGB/RGBA preserve known pixels; invalid data and resource limits return errors. |
| Platform image | Encoded files and memory share decoding; crop, scale, offsets and protocol payloads have pixel and boundary assertions. |
| App | Measured placement, parent clipping, independent images, source replacement, resize, removal and GIF playback have captured-frame tests. |
| Surface and graphics writer | Source regions, alpha, text layering, bounded placement, write-failure retry and cleanup have output assertions. |
| Linux hosts | Kitty and Ghostty graphics; Xterm Sixel; WezTerm inline; Chafa/Viu through Kitty; Auto and Surface fallback through GNOME Terminal. App, standalone and Surface routes retain their applicable host cases. |
| macOS and Windows | The native widget verifier checks current committed input digests and captured-output hashes, including image, platform-image, App and Surface tests. macOS also requires iTerm geometry captures. |

The Linux driver runs in private Xvfb displays using software rendering. It
measures actual screenshot pixels during source changes, movement and removal.
Full-screen Sixel cases check that output does not scroll away the stage label.
GIF cases change frames without replacing their source. External renderer cases
also enlarge the image. A forced ASCII case must fail the exact-color assertions;
startup failures and timeouts cannot satisfy that negative control.

The runner uses the pinned, repaired Kitty 0.45.0 build described in
[`scripts/kitty-host/README.md`](../scripts/kitty-host/README.md). Stock Kitty
0.45.0 can leave a valid image invisible until another repaint. The approved
repair prepares image placements before choosing the paint path. Acceptance
verifies the private runtime hashes before and after capture and also runs an
independent C sender through the same pixel assertions. This does not install a
fix for users of stock Kitty; those users still need a corrected host build.

The runner requires Ghostty, Xterm, WezTerm, GNOME Terminal, Chafa, Viu,
Xvfb, ImageMagick's `import`, and Python Pillow on the test host. External image
tools must be on `PATH`. It records executable paths and renderer versions.
Linux image hosts and the Kitty builder use at most 12 workers per worker pool.
The test binary uses this repository's `target` directory, which must also be
the `CARGO_TARGET_DIR` when that environment variable is set. Run:

```sh
CARGO_TARGET_DIR="$PWD/target" cairn check API-014
```

Timestamped screenshots, pixel measurements and logs are retained under
`.cairn/reviews/api-image-hosts`. Commit them with the receipt after the check;
they are outputs, not inputs to their own check. Preserve failed captures.
The initial development captures remain in `docs/analysis/api-image-hosts` as
an immutable declared input archive.

The existing URL decision retains local paths and base64 data URLs. HTTP and
HTTPS image sources return an explicit unsupported-source error. The approved
iTerm2 3.7 exception covers color accuracy and transparency only; movement,
replacement and removal remain required. WezTerm retains exact inline color
acceptance. Neither software-rendered Linux captures nor iTerm geometry proves
support on other terminal versions or hardware configurations.
