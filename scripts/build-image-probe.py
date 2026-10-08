#!/usr/bin/env python3
"""Build the image_host_probe example and print its executable path as JSON.

scripts/check-widget-platforms.py runs this as a child of its bounded
runner, so Cargo and the compiler stay in the runner's process group and
under its deadline.
"""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = "image_host_probe"


def executable(messages: str) -> str:
    """The executable Cargo reported for the example, from its JSON messages."""
    for line in messages.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(message, dict):
            continue
        target = message.get("target") or {}
        if (message.get("reason") == "compiler-artifact" and target.get("name") == EXAMPLE
                and "example" in target.get("kind", []) and message.get("executable")):
            return message["executable"]
    raise RuntimeError(f"Cargo did not report the {EXAMPLE} example executable")


def main() -> int:
    result = subprocess.run(
        ["cargo", "build", "--locked", "--example", EXAMPLE, "--jobs", "8", "--message-format=json"],
        cwd=ROOT, check=True, timeout=600, stdout=subprocess.PIPE, text=True)
    print(json.dumps(executable(result.stdout)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
