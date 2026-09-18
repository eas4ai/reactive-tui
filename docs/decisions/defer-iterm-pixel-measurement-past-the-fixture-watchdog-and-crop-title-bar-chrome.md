# Defer iTerm pixel measurement past the fixture watchdog and crop title-bar chrome

Level: Judged
Decided by: agent
Rests on: API-014
Would be wrong if: the rerun still exceeds the 45 s fixture watchdog or traffic-light pixels remain in the measured region
History: API-domain reversals narrowed over-broad repairs; this stays Judged by confining the change to one check script with no product or spec change and a rerun falsifier

## Decision

Opus diagnosed the app-iterm exit-1 as the probe's own 45 s watchdog starved by ~14 s/stage in-loop pixel scans, plus traffic-light chrome poisoning the geometry asserts. Fix in scripts/check-iterm-host.py only: collect the three screenshots in the loop and measure after the fixture-exit check; add screencapture -o and a top chrome crop; capture the fixture child stderr to fixture-stderr.txt (stdout stays on the fixture pty so the TUI renders) so the next probe error is not lost when iTerm is torn down. Chrome is the ~28 pt title bar, a safe underestimate whose guard fails loudly on geometry shifts. No numpy: post-exit measurement cost is off the watchdog path, so no new dependency. No product code changes.

## Realized by

155d2632 Repair iTerm host capture: defer measurement past fixture watchdog, crop title-bar chrome, keep fixture output
3d524a78 Keep fixture stdout on its pty and crop the shadowless title bar
