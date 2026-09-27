#!/usr/bin/env python3
"""Run isolated regression binaries repeatedly, with process-level deadlines."""
import json
import subprocess

TARGETS = ("registry_concurrency", "simple_performance_test")
command = ["cargo", "test", "--locked", "--no-run", "--message-format=json"]
for target in TARGETS:
    command.extend(["--test", target])
build = subprocess.run(command, text=True, stdout=subprocess.PIPE, check=True, timeout=180)
binaries = {}
for line in build.stdout.splitlines():
    message = json.loads(line)
    if message.get("reason") == "compiler-artifact" and message.get("executable"):
        name = message["target"]["name"]
        if name in TARGETS:
            binaries[name] = message["executable"]
assert set(binaries) == set(TARGETS), "cargo did not produce every regression binary"
for target, executable in binaries.items():
    for repetition in range(25):
        try:
            result = subprocess.run(
                [executable, "--test-threads=4"], text=True,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=30,
            )
        except subprocess.TimeoutExpired as error:
            raise SystemExit(f"{target} stalled on repetition {repetition + 1}: {error.stdout!r}") from error
        if result.returncode:
            print(result.stdout)
            raise SystemExit(f"{target} failed on repetition {repetition + 1}: {result.returncode}")
    print(f"PASS {target}: 25 concurrent-harness runs", flush=True)
