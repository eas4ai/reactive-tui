#!/usr/bin/env python3
"""Reproduce audit findings. Passing assertions confirm defects, not correctness."""
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
subprocess.run(['cargo', 'build', '--locked', '--features', 'ffi'], cwd=root, check=True, timeout=300)
subprocess.run(['cargo', 'test', '--locked', '--test', 'suprtui_renderer', '--no-run'], cwd=root, check=True, timeout=300)
with tempfile.TemporaryDirectory(prefix='rust-api-audit-', dir=root / 'target') as scratch:
    scratch = Path(scratch)
    source = Path(__file__).with_name('probes.rs').read_text().replace('__REPO_ROOT__', str(root))
    (scratch / 'probes.rs').write_text(source)
    vt100 = max((root / 'target/debug/deps').glob('libvt100-*.rlib'), key=lambda p: p.stat().st_mtime)
    subprocess.run(['rustc', '--edition=2021', '--test', str(scratch / 'probes.rs'),
                    '--extern', 'reactive_tui=' + str(root / 'target/debug/libreactive_tui.rlib'),
                    '--extern', 'vt100=' + str(vt100), '-L', 'dependency=' + str(root / 'target/debug/deps'),
                    '-o', str(scratch / 'probes')], cwd=root, check=True, timeout=90)
    subprocess.run([str(scratch / 'probes'), '--test-threads=1', '--nocapture'], cwd=root, check=True, timeout=30)
