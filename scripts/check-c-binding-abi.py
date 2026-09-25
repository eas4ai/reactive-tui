#!/usr/bin/env python3
"""Audit declarations before compiling and running real C and C++ consumers."""
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    os.chdir(Path(__file__).resolve().parents[1])
    subprocess.run(['python3', '-B', 'scripts/check-binding-abi.py'], check=True, timeout=300)
    spec = importlib.util.spec_from_file_location('ffi_runtime', 'scripts/check-ffi-runtime.py')
    runtime = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runtime)
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target')).resolve()
    with tempfile.TemporaryDirectory(prefix='c-abi-consumer-', dir=target) as scratch:
        for compiler, language, standard in [('clang', 'c', 'c11'), ('clang++', 'c++', 'c++17')]:
            binary = Path(scratch) / language.replace('+', 'p')
            subprocess.run([
                compiler, '-x', language, '-std=' + standard, '-Wall', '-Wextra',
                '-Werror', '-Wno-missing-field-initializers', '-pedantic-errors',
                '-Iinclude', 'tests/binding_abi_consumer.c', '-L' + str(target / 'debug'),
                '-Wl,-rpath,' + str(target / 'debug'), '-lreactive_tui', '-o', str(binary),
            ], check=True, timeout=60)
            runtime.run_in_terminal([str(binary)])


if __name__ == '__main__':
    main()
