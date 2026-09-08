#!/usr/bin/env python3
"""Run every inherited acceptance declaration and reject missing coverage."""
import os
from pathlib import Path
import shlex
import signal
import subprocess

MECHANISMS = (
    "maintenance-format", "maintenance-lint", "maintenance-ffi",
    "default-suite-repair", "registry-cache-isolation", "registry-concurrency",
    "app-wakeups", "embedded-terminal", "suprtui-renderer",
)
EXPECTED = {f"{prefix}-{number:03}" for prefix, count in
            (("MNT", 4), ("DFT", 4), ("CCH", 2), ("REG", 3),
             ("WAK", 5), ("EMB", 6), ("RND", 6))
            for number in range(1, count + 1)}


def main():
    os.chdir(Path(__file__).resolve().parents[1])
    combined = Path('.cairn/mechanisms/binding-inherited.md').read_text()
    declared_inputs = set(combined.split('inputs:\n')[1].split('requirements:')[0].splitlines())
    covered = set()
    commands = []
    for name in MECHANISMS:
        path = Path('.cairn/mechanisms') / (name + '.md')
        text = path.read_text()
        inputs = set(text.split('inputs:\n')[1].split('requirements:')[0].splitlines())
        if not inputs <= declared_inputs or '  - ' + str(path) not in declared_inputs:
            raise RuntimeError('Combined declaration omits inputs from ' + name)
        requirements = text.split('requirements:\n')[1].split('reviewed:')[0]
        covered.update(line.strip()[2:] for line in requirements.splitlines()
                       if line.strip().startswith('- '))
        command = [line.removeprefix('command: ') for line in text.splitlines()
                   if line.startswith('command: ')]
        if len(command) != 1:
            raise RuntimeError('Expected one command in ' + str(path))
        commands.append((name, shlex.split(command[0])))
    if covered != EXPECTED:
        raise RuntimeError('Inherited requirement coverage differs: ' + repr(covered ^ EXPECTED))
    for name, command in commands:
        print('Inherited acceptance: ' + name, flush=True)
        process = subprocess.Popen(command, start_new_session=True)
        try:
            status = process.wait(timeout=600)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise RuntimeError('Inherited acceptance timed out: ' + name) from None
        if status:
            raise RuntimeError(f'Inherited acceptance failed: {name} (exit {status})')
    print('All 30 inherited requirements passed through nine acceptance commands', flush=True)


if __name__ == '__main__':
    main()
