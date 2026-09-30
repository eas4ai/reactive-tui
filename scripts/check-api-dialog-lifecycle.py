#!/usr/bin/env python3
"""Run the dedicated dialog lifecycle acceptance target."""

import os
from pathlib import Path
import re
import signal
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parent.parent
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", CARGO_INCREMENTAL="0")
    commands = [
        ["cargo", "test", "--locked", "--test", "api_dialog_lifecycle"],
        ["cargo", "test", "--locked", "--lib", "widgets::dialog::"],
        ["cargo", "test", "--locked", "--lib", "widgets::display::modal::"],
        ["cargo", "test", "--locked", "--test", "api_widget_behavior", "dialog_engine_http_"],
    ]
    for command in commands:
        print("+ " + " ".join(command), flush=True)
        child = subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE,
                                 stderr=subprocess.STDOUT, text=True, start_new_session=True)
        try:
            output, _ = child.communicate(timeout=600)
        except subprocess.TimeoutExpired:
            if os.name == "posix":
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass  # The group finished at the deadline.
            else:
                subprocess.run(["taskkill", "/PID", str(child.pid), "/T", "/F"], timeout=10, check=False)
                child.kill()
            output, _ = child.communicate()
            print(output, end="", flush=True)
            print("Dialog lifecycle acceptance exceeded 600 seconds", file=sys.stderr)
            return 1
        print(output, end="", flush=True)
        if child.returncode:
            return child.returncode
        if command[1] == "test" and not re.search(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", output):
            print("The selected dialog behavior tests did not execute", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
