from pathlib import Path
from datetime import datetime, timezone
import json
import os
import runpy

root = Path(__file__).resolve().parents[2]
os.chdir(root)
os.environ.update(CARGO_BUILD_JOBS='12', RUST_TEST_THREADS='12', CARGO_TARGET_DIR=str(root / 'target'))
output = root / 'target/evidence/api-019-local-scope-consumer' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
output.mkdir(parents=True)
execute = runpy.run_path(str(root / 'scripts/check-widget-platforms.py'))['execute']
command = ['rustc', '--edition=2021', 'verification/api-residual/local-owner.rs',
           '--extern', 'reactive_tui=' + str(root / 'target/debug/libreactive_tui.rlib'),
           '-L', 'dependency=' + str(root / 'target/debug/deps'), '-C', 'link-arg=-Wl,--threads=12']
records = []
def run(name, args, expect_failure=False):
    error = None
    try:
        execute(args, output / (name + '.out'), 900)
    except Exception as failure:
        error = str(failure)
    records.append({'name': name, 'command': args, 'error': error})
    (output / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')
    assert (error is not None) == expect_failure, records[-1]
    print(name, 'observed failure' if error else 'passed', output, flush=True)
run('library', ['cargo', 'build', '--locked', '--lib'])
binary = root / 'target/api019-local-owner-probe'
run('consumer-compile', command + ['-o', str(binary)])
run('scoped-retention', [str(binary)])
assert 'LOCAL_OWNER_SCOPE_OK' in (output / 'scoped-retention.out').read_text()
run('thread-confinement', command + ['--cfg', 'send_local', '-o', str(root / 'target/api019-local-owner-forbidden')], expect_failure=True)
diagnostic = (output / 'thread-confinement.out').read_text()
assert 'E0277' in diagnostic and 'cannot be sent between threads safely' in diagnostic
