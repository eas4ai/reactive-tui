#!/usr/bin/env python3
"""Audited, compiled C and TypeScript consumers of native component behavior."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve()
SOURCE = ROOT / 'verification/api-native-components'


def execute(command, timeout=300, **kwargs):
    print('+', ' '.join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), cwd=ROOT, check=True, timeout=timeout, **kwargs)


def workflow(terminal_type, command, screen, captures, name, mode='host'):
    terminal = terminal_type(command + [mode], screen,
                             captures / (name + '.bin'), (64, 16))
    try:
        if mode != 'error':
            frame = terminal.wait_for('count0', lambda text: 'Right' in text and 'Edit:' in text)
            assert 'Left   Right' in frame, 'native layout spacing was not painted'
            terminal.send(b'x')
            if mode == 'event-error':
                terminal.finish()
                return
            if mode == 'negative':
                try:
                    terminal.wait_for('count1')
                except AssertionError as error:
                    assert "did not paint 'count1'" in str(error), str(error)
                    terminal.send(b'\x03')
                    terminal.finish()
                    print('PASS safe violating input callback was rejected; terminal restored', flush=True)
                    return
                raise AssertionError('Acceptance incorrectly allowed the violating input callback')
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
    captures = ROOT / 'target/evidence/api-native-components' / time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())
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
            execute(['node', package / 'scripts' / 'build.cjs'])
            execute([package / 'node_modules/.bin/tsc', SOURCE / 'consumer.ts', '--strict', '--target',
                     'ES2020', '--module', 'commonjs', '--moduleResolution', 'node', '--esModuleInterop',
                     '--resolveJsonModule', '--skipLibCheck', '--types', 'node', '--typeRoots',
                     package / 'node_modules/@types', '--outDir', scratch / 'typescript'])
        except subprocess.CalledProcessError:
            failures.append('TypeScript compilation')
        if failures:
            raise SystemExit('Native consumer failures: ' + ', '.join(failures))
        execute(['cargo', 'test', '--locked', '--features', 'ffi', '--test', 'suprtui_renderer', '--no-run'])
        vt100 = max((TARGET / 'debug/deps').glob('libvt100-*.rlib'), key=lambda p: p.stat().st_mtime_ns)
        screen = scratch / 'screen'
        execute(['rustc', '--edition=2021', ROOT / 'verification/api-entry-points/screen.rs', '--extern',
                 'vt100=' + str(vt100), '-L', 'dependency=' + str(TARGET / 'debug/deps'), '-o', screen])
        env = {**os.environ, 'RTUI_LIBRARY_PATH': str(TARGET / 'debug/libreactive_tui.so'),
               'NODE_PATH': str(package / 'node_modules')}
        typescript = scratch / 'typescript/verification/api-native-components/consumer.js'
        env_program = shutil.which('env')
        assert env_program, 'env executable is required for the TypeScript PTY consumer'
        commands = {'C': [str(native)], 'TypeScript': [env_program, *[f'{key}={env[key]}' for key in ('RTUI_LIBRARY_PATH', 'NODE_PATH')],
                                                     'node', str(typescript)]}
        negative = scratch / 'violating-consumer'
        execute(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-DAPI_NATIVE_NEGATIVE', '-Iinclude', SOURCE / 'consumer.c',
                 '-L' + str(TARGET / 'debug'), '-Wl,-rpath,' + str(TARGET / 'debug'),
                 '-lreactive_tui', '-pthread', '-o', negative])
        workflow(entry_points.Terminal, [str(negative)], screen, captures, 'violating-input-callback', 'negative')
        for name, command in commands.items():
            try:
                execute(command, timeout=60)
                for mode in ('host', 'error', 'event-error'):
                    workflow(entry_points.Terminal, command, screen, captures,
                             name + '-' + mode, mode)
                    print('PASS', name, mode, flush=True)
                for family in ('confirmation', 'toast', 'progress', 'autocomplete', 'wizard'):
                    dialog_workflow(entry_points.Terminal, command, screen, captures, name, family)
            except (AssertionError, OSError, subprocess.SubprocessError) as exception:
                failures.append(name)
                print('FAIL', name, str(exception), flush=True)
    if failures:
        raise SystemExit('Native consumer failures: ' + ', '.join(failures))
    print('All native component consumer workflows passed', flush=True)


def dialog_workflow(terminal_type, command, screen, captures, language, family):
    options = {'kind': family, 'title': 'Native ' + family}
    if family == 'confirmation': options['message'] = 'Proceed?'
    if family == 'toast': options.update(message='Toast ready', duration=1500)
    if family == 'progress': options.update(message='Processing', progress=0.25)
    if family == 'autocomplete': options.update(prompt='Search', suggestions=['Alpha', 'Beta'])
    if family == 'wizard': options['steps'] = [
        {'id': 'first', 'title': 'First', 'content': 'First panel'},
        {'id': 'second', 'title': 'Second', 'content': 'Second panel'},
    ]
    terminal = terminal_type(command + ['dialog', json.dumps(options, separators=(',', ':'))],
                             screen, captures / f'{language}-dialog-{family}.bin', (64, 20))
    def click(label):
        frame = terminal.wait_for(label)
        lines = frame.splitlines()
        y = next(index for index, line in enumerate(lines) if label in line)
        x = lines[y].index(label)
        terminal.send(f'\x1b[<0;{x+1};{y+1}M\x1b[<0;{x+1};{y+1}m'.encode())
    try:
        if family == 'confirmation':
            terminal.wait_for('Proceed?'); terminal.send(b'\r'); result = 'confirmed'
        elif family == 'toast':
            terminal.wait_for('Toast ready'); result = 'confirmed'
        elif family == 'progress':
            terminal.wait_for('75.0%'); click('Cancel'); result = 'cancelled'
        elif family == 'autocomplete':
            terminal.wait_for('Search'); terminal.send(b'\x1b[200~Al\x1b[201~')
            terminal.wait_for('Alpha'); terminal.send(b'\x1b[B\r'); result = 'selected'
        else:
            terminal.wait_for('First panel'); click('Next')
            terminal.wait_for('Second panel'); click('Finish'); result = 'confirmed'
        terminal.wait_for('completion=' + result)
        terminal.send(b'\x03'); terminal.finish()
    finally:
        terminal.close()
    print('PASS', language, family, 'native paint, interaction and actual result', flush=True)


if __name__ == '__main__':
    main()
