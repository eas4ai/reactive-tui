# Isolated Kitty image host

API-014 uses this Linux build for the host repair approved in
`api-014-api-020`. It leaves the installed terminal untouched. Stock Kitty 0.45.0
can skip a new image until another repaint; passing with this build does not
establish that an unmodified installation works.

`source.json` pins the upstream GPL-3.0 source archive and its SHA-256 digest.
`image-layer-order.patch` moves the normal-window image-layer query after
`send_cell_data_to_gpu`, which prepares placement counts. It changes no image
encoding, shader, pixel threshold, capture delay or transmission behavior.

`build.py` verifies the archive, applies the patch without fuzz, and builds under
`target/kitty-image-host`. It uses `/usr/bin/python3` with Python's CPU-count
override, disables LTO, and caps the C and Go build workers at 12. The source
tree must stay at its build path because Kitty's development launcher uses it.
The runtime remains local; no binary is committed or installed globally.

Install the build dependencies that Kitty's own build guide (`build.rst` under the pinned archive's documentation directory) lists:
a C compiler, Go, Python development headers, pkg-config, X11/Wayland headers,
font and image libraries, libcanberra, xxhash, and SIMDe. The required Symbols
Nerd Font Mono must be visible to fontconfig, or available with its license in
the distro Kitty directory `/usr/lib/kitty/fonts`. This builder copies that
packaged font into the private tree and records its hashes.

```sh
/usr/bin/python3 -B scripts/kitty-host/build.py --jobs 12
/usr/bin/python3 -B scripts/kitty-host/build.py --verify-only
```

`--archive PATH` accepts a local copy with the same pinned digest. Missing
development packages may also be extracted into a private prefix and supplied
through `PKG_CONFIG_PATH` and `CFLAGS`. On this machine GCC needed
`-Wno-error=discarded-qualifiers -Wno-error=maybe-uninitialized` for two existing
upstream diagnostics. They remain warnings; the source changes only the image
ordering. The dotted-underline caller clamps the dot count to at least one and
the callee initializes every gap before reading it. The qualifier warning comes
from `expand_tilde`, which temporarily replaces a slash while expanding
`~user/path`, then restores it. These image checks do not audit that upstream
path-expansion behavior; the input-ownership concern is recorded in the backlog.

`build.json` records the source and patch digests, compiler, Go version, build
command and environment, build-log digest, and every runtime file's digest.
Subsequent calls reject runtime changes. The acceptance runner retains this
record with its captures and verifies the runtime again after all host cases.

`reference.c` generates opaque 128×64 RGBA images independently using zlib and
OpenSSL base64. It waits one second after the text header, then sends each image
once. The existing driver checks first placement, replacement at a new position,
and removal with its unchanged four-second capture intervals and pixel checks.
There is no repaint request, retransmission, or Rust image encoder in this
reference path. Compile it with `cc -std=c11 -O2 -Wall -Wextra -Werror
reference.c -lz -lcrypto -o reference` from this directory.

The before/after investigation, its build evidence and the earlier independent
repaint controls were Cairn 1.x review records. They were removed from the tree
on 2026-09-19 in commit 7fd91fc7; git history keeps them in its parent.
