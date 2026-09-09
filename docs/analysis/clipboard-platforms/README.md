# Native clipboard evidence

API-008 requires current results from Wayland (wl-copy/wl-paste), Xsel,
Xclip, macOS (pbcopy/pbpaste), and Windows PowerShell. Fixture passes do
not substitute for these runs. Missing or stale records fail acceptance.

Commit the clipboard inputs before recording results. From the repository root:

```
python3 scripts/check-clipboard-platforms.py --backend wayland
python3 scripts/check-clipboard-platforms.py --backend xsel
python3 scripts/check-clipboard-platforms.py --backend xclip
```

Linux runs create private KWin/DBus or Xvfb desktops and clean up their own
clipboard servers. They require the named tools, KWin, DBus, Xvfb and cat
under /usr/bin. They do not select the user's desktop clipboard.

Run the following only on disposable native desktop sessions, with the same
committed source and its Rust/Zig build prerequisites. Each replaces that
session's clipboard with test text:

```
python3 scripts/check-clipboard-platforms.py --backend macos --dedicated-desktop
python3 scripts/check-clipboard-platforms.py --backend windows --dedicated-desktop
```

Each run checks backend selection and five exact copy/paste round trips,
including Unicode, quotes, empty text, trailing newlines and a longer value.
It also runs native deadline and cancellation tests that assert the owned
process has exited. The JSON record includes the source digest, platform,
commit, tool hashes and hashes of captured test output. Copy all three
backend files into this directory and commit them. The verifier rejects
missing, stale or damaged results; it does not authenticate a remote runner.

`--allow-dirty --output target/clipboard-platform-development` is for local
development only. Such results cannot satisfy the acceptance mechanism.

The command deadline is two seconds and the text transfer limit is 64 MiB.
Timeout and hook-owner cleanup stop and reap the direct child; Unix also
stops its process group. Successful copy tools may retain a background
clipboard server, as required by Wayland and X11 selection ownership.
Commands use private temporary files to avoid blocking on inherited pipes.
The deadline is cooperative between system calls, not a real-time guarantee
against a stalled kernel or filesystem. Native macOS and Windows behavior
remains unverified until their records are supplied.
