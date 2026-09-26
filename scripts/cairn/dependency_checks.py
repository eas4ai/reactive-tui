#!/usr/bin/env python3
"""BAR-008: cargo deny's advisory, bans, license and source checks pass for
the whole workspace, against the RustSec advisory database as cargo deny
fetches it at the start of the run, and every advisory ignore and bans skip
in deny.toml states its reason.

When the database fetch itself fails (no network), the run reaches no
verdict: the script prints no result line, so Sudus records it as
unverified rather than as a failure of the tree.
"""

from __future__ import annotations

import sys
import tomllib

from _common import ROOT, report, run

COMMAND = ["cargo", "deny", "--workspace", "--locked", "check", "advisories", "bans", "licenses", "sources"]
FETCH_FAILED = "failed to fetch advisory database"


def unexplained_entries(policy: dict) -> list[str]:
    """Each advisory ignore, bans skip or skip-tree entry that gives no reason."""
    found = []
    sections = (("advisories", "ignore", "id"), ("bans", "skip", "crate"), ("bans", "skip-tree", "crate"))
    for section, key, name in sections:
        for entry in policy.get(section, {}).get(key, []):
            if isinstance(entry, dict):
                reason = entry.get("reason")
                if isinstance(reason, str) and reason.strip():
                    continue
                label = entry.get(name, entry)
            else:
                label = entry
            found.append(f"[{section}] {key} entry {label!r} gives no reason")
    return found


def main() -> int:
    try:
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        report("BAR-008", False, f"deny.toml cannot be read: {error}")
        return 1
    result = run(COMMAND, timeout=900, interleave=True)
    output = result.stdout or ""
    print(output, end="" if output.endswith("\n") else "\n")
    if FETCH_FAILED in output:
        # No result line: Sudus records the run as unverified.
        print("BAR-008 unverified: cargo deny could not fetch the advisory database, so it reached no verdict")
        return 1
    failed = []
    if result.returncode != 0:
        summary = next((line.strip() for line in output.splitlines()
                        if "advisories" in line and "bans" in line and "sources" in line), "")
        failed.append(f"cargo deny exited {result.returncode}" + (f" ({summary})" if summary else ""))
    failed += unexplained_entries(policy)
    if failed:
        report("BAR-008", False, "; ".join(failed))
        return 1
    report("BAR-008", True, "advisories, bans, licenses and sources pass for the workspace; every ignore and skip gives its reason")
    return 0


if __name__ == "__main__":
    sys.exit(main())
