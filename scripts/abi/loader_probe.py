"""Check the actual Koffi interface against independent compiler results."""
import json
import os
from pathlib import Path
import re
import struct
import subprocess

from .compiler_probe import c_type


def run_loader_probe(schema, audited, target, scratch):
    package = Path('bindings/typescript').resolve()
    compiler = package / 'node_modules/.bin/tsc'
    if not compiler.exists():
        raise RuntimeError('Install TypeScript dependencies with npm ci in bindings/typescript')
    dist = scratch / 'typescript'
    subprocess.run([
        str(compiler), os.path.join('src', 'ffi.ts'), '--target', 'ES2020', '--module', 'commonjs',
        '--esModuleInterop', '--resolveJsonModule', '--strict', '--skipLibCheck',
        '--outDir', str(dist),
    ], cwd=package, check=True, timeout=60)
    result = subprocess.check_output([
        'node', 'scripts/abi/inspect-loader.cjs', str(dist),
    ], text=True, timeout=30, env={
        **os.environ, 'NODE_PATH': str(package / 'node_modules'),
        'RTUI_LIBRARY_PATH': str(target / 'debug/libreactive_tui.so'),
    })
    actual = json.loads(result)
    for name, value in actual['errorValues'].items():
        if f'enum {name} {value}' not in audited['layouts']:
            raise RuntimeError('TypeScript error enum differs from native values: ' + name)
    if len(actual['errorValues']) != len([line for line in audited['layouts'] if line.startswith('enum R_TUI_ERROR_')]):
        raise RuntimeError('TypeScript error enum inventory is incomplete')
    expected_layouts = [line for line in audited['layouts'] if not line.startswith('enum ')]
    if actual['layouts'] != expected_layouts:
        raise RuntimeError('Koffi record/enum layouts differ from compiler results')

    def normalize(value):
        value = c_type(value.replace('bool', '_Bool'), schema['callbacks'])
        value = value.replace('*const ', '*').replace('*mut ', '*')
        value = value.replace('c_char', 'i8').replace('c_void', 'void').replace('()', 'void')
        value = value.replace('usize', 'u' + str(struct.calcsize('P') * 8))
        for name in schema['enums']:
            value = re.sub(r'\b' + name + r'\b', 'i32', value)
        return value

    expected = {
        name: 'callback(' + ','.join(normalize(p) for p in entry['parameters'])
        + ')->' + normalize(entry['result'])
        for name, entry in schema['functions'].items()
    }
    if actual['functions'] != expected:
        differences = {name: (expected[name], actual['functions'].get(name))
                       for name in expected if expected[name] != actual['functions'].get(name)}
        raise RuntimeError('Koffi calling signatures differ: ' + repr(differences))
    expected_fields = {name + '.' + field: normalize(value)
                       for name, record in schema['records'].items()
                       for field, value in record['fields'].items()}
    if actual['fields'] != expected_fields:
        raise RuntimeError('Koffi record field types differ from compiler declarations')
    print(f'Koffi agrees with independent compiler signatures and layouts: {len(expected)} exports')
