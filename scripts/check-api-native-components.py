#!/usr/bin/env python3
"""Audited, compiled C and TypeScript consumers of native component behavior."""
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve()
SOURCE = ROOT / 'verification/api-native-components'


def execute(command, timeout=300, **kwargs):
    print('+', ' '.join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), cwd=ROOT, check=True, timeout=timeout, **kwargs)


def workflow(terminal_type, command, screen, captures, name, error=False):
    terminal = terminal_type(command + ['error' if error else 'host'], screen,
                             captures / (name + '.bin'), (64, 16))
    try:
        if not error:
            frame = terminal.wait_for('count0', lambda text: 'Right' in text and 'Edit:' in text)
            assert 'Left   Right' in frame, 'native layout spacing was not painted'
            terminal.send(b'x')
            terminal.wait_for('count1')
            terminal.send(b'p')
            terminal.wait_for('"omega" count1')
            terminal.send(b'e')
            terminal.wait_for('Edit:界é')
            terminal.send(b'd')
            terminal.wait_for('Enter name')
            terminal.send(b'\x1b[200~Ada\x1b[201~')
            terminal.wait_for('Ada')
            terminal.send(b'\r')
            terminal.wait_for('result=Ada')
            # Closing the dialog restores the foreign target, with state intact.
            terminal.send(b'x')
            terminal.wait_for('count2')
            terminal.resize((48, 12))
            terminal.wait_for('count2', lambda text: 'Left   Right' in text)
            terminal.send(b'\x03')
        terminal.finish()
    finally:
        terminal.close()
    print('PASS', name, 'native frames, input/results, ownership and restoration', flush=True)


def main():
    os.chdir(ROOT)
    execute(['python3', '-B', 'scripts/check-binding-abi.py'])
    spec = importlib.util.spec_from_file_location('entry_points', ROOT / 'scripts/check-api-entry-points.py')
    entry_points = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(entry_points)
    captures = ROOT / '.cairn/reviews/api-native-components' / time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())
    captures.mkdir(parents=True)
    print('Captures:', captures, flush=True)
    failures = []
    with tempfile.TemporaryDirectory(prefix='api-native-components-', dir=TARGET) as temporary:
        scratch = Path(temporary)
        native = scratch / 'consumer'
        # A strict C consumer must compile against the shipped public header.
        try:
            execute(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-Iinclude', SOURCE / 'consumer.c',
                     '-L' + str(TARGET / 'debug'), '-Wl,-rpath,' + str(TARGET / 'debug'),
                     '-lreactive_tui', '-pthread', '-o', native])
        except subprocess.CalledProcessError:
            failures.append('C compilation')
        package = ROOT / 'bindings/typescript'
        try:
            execute(['node', package / 'scripts/build.cjs'])
            execute([package / 'node_modules/.bin/tsc', SOURCE / 'consumer.ts', '--strict', '--target',
                     'ES2020', '--module', 'commonjs', '--moduleResolution', 'node', '--esModuleInterop',
                     '--resolveJsonModule', '--skipLibCheck', '--types', 'node', '--typeRoots',
                     package / 'node_modules/@types', '--outDir', scratch / 'typescript'])
        except subprocess.CalledProcessError:
            failures.append('TypeScript compilation')
        if failures:
            raise SystemExit('Native consumer failures: ' + ', '.join(failures))
        execute(['cargo', 'test', '--locked', '--test', 'suprtui_renderer', '--no-run'])
        vt100 = max((TARGET / 'debug/deps').glob('libvt100-*.rlib'), key=lambda p: p.stat().st_mtime_ns)
        screen = scratch / 'screen'
        execute(['rustc', '--edition=2021', ROOT / 'verification/api-entry-points/screen.rs', '--extern',
                 'vt100=' + str(vt100), '-L', 'dependency=' + str(TARGET / 'debug/deps'), '-o', screen])
        env = {**os.environ, 'RTUI_LIBRARY_PATH': str(TARGET / 'debug/libreactive_tui.so'),
               'NODE_PATH': str(package / 'node_modules')}
        typescript = scratch / 'typescript/verification/api-native-components/consumer.js'
        commands = {'C': [str(native)], 'TypeScript': ['env', *[f'{key}={env[key]}' for key in ('RTUI_LIBRARY_PATH', 'NODE_PATH')],
                                                     'node', str(typescript)]}
        for name, command in commands.items():
            try:
                execute(command, timeout=60)
                for error in (False, True):
                    workflow(entry_points.Terminal, command, screen, captures,
                             name + ('-error' if error else '-workflow'), error)
            except (AssertionError, OSError, subprocess.SubprocessError) as exception:
                failures.append(name)
                print('FAIL', name, str(exception), flush=True)
    if failures:
        raise SystemExit('Native consumer failures: ' + ', '.join(failures))
    print('All native component consumer workflows passed', flush=True)


if __name__ == '__main__':
    main()
