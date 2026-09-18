# Defer iTerm pixel measurement past the fixture watchdog and crop title-bar chrome

Level: Judged
Decided by: agent
Rests on: API-014
Would be wrong if: the rerun still exceeds the 45 s fixture watchdog or traffic-light pixels remain in the measured region
History: API-domain reversals narrowed over-broad repairs; this stays Judged by confining the change to one check script with no product or spec change and a rerun falsifier

## Decision

Opus diagnosed the app-iterm exit-1 as the probe's own 45 s watchdog starved by ~14 s/stage in-loop pixel scans, plus traffic-light chrome poisoning the geometry asserts. Fix in scripts/check-iterm-host.py only: collect the three screenshots in the loop and measure after the fixture-exit check; add screencapture -o and a top chrome crop calibrated from iterm-run-terminal stage-0 (buttons end y=121, image starts y=214 at 1364 px over a 570 pt window, chrome 167 px ~= 70 pt, guarded to fail loudly above H//3); capture the fixture child stdout/stderr to fixture-output.txt so the next probe error is not lost when iTerm is torn down. No numpy: post-exit measurement cost is off the watchdog path, so no new dependency. No product code changes.

## Realized by

(none yet: recorded, not built)
