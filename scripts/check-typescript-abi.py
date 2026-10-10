#!/usr/bin/env python3
"""Typecheck the whole package and run bounded, previously audited native calls."""
import importlib.util
import os
from pathlib import Path
import subprocess


def main():
    os.chdir(Path(__file__).resolve().parents[1])
    # Audit before importing/running any consumer function.
    subprocess.run(['python3', '-B', 'scripts/check-binding-abi.py'], check=True, timeout=300)
    package = Path('bindings/typescript').resolve()
    subprocess.run(['node', str(package / 'scripts' / 'build.cjs')], check=True, timeout=70)
    subprocess.run([
        str(package / 'node_modules/.bin/tsc'), os.path.join('tests', 'consumer-types.ts'), '--noEmit',
        '--strict', '--target', 'ES2020', '--module', 'commonjs', '--moduleResolution',
        'node', '--esModuleInterop',
    ], cwd=package, check=True, timeout=60)
    subprocess.run([str(package / 'node_modules/.bin/tsc'), '-p', 'tsconfig.examples.json'],
                   cwd=package, check=True, timeout=60)
    subprocess.run(['npm', 'run', 'lint'], cwd=package, check=True, timeout=60)
    spec = importlib.util.spec_from_file_location('ffi_runtime', 'scripts/check-ffi-runtime.py')
    runtime = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runtime)
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target')).resolve()
    runtime.run_in_terminal([
        'env', 'RTUI_LIBRARY_PATH=' + str(target / 'debug/libreactive_tui.so'),
        'node', str(package / 'tests' / 'native-consumer.cjs'),
    ])


if __name__ == '__main__':
    main()
