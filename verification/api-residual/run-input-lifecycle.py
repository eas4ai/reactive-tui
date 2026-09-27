from datetime import datetime, timezone
import json
import os
from pathlib import Path
import runpy
import sys

root = Path(__file__).resolve().parents[2]
if sys.platform != 'linux':
    raise SystemExit('This diagnostic requires Linux /proc and private PTYs')
os.chdir(root)
stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
output = root / 'target/evidence/api-019-input-lifecycle' / stamp
output.mkdir(parents=True)
environment = {**os.environ, 'CARGO_TARGET_DIR': str(root / 'target'),
               'CARGO_INCREMENTAL': '0', 'CARGO_BUILD_JOBS': '8', 'RUST_TEST_THREADS': '8',
               'RAYON_NUM_THREADS': '8', 'LP_NUM_THREADS': '8', 'PYTHON_CPU_COUNT': '8',
               'GOMAXPROCS': '8', 'GOFLAGS': '-p=8'}
commands = []
os.environ.update(environment)
execute = runpy.run_path(str(root / 'scripts/check-widget-platforms.py'))['execute']


def run(name, command, timeout):
    try:
        execute(command, output / (name + '.out'), timeout)
        status, error = 0, None
    except Exception as failure:
        status, error = 1, str(failure)
    commands.append({'name': name, 'command': command, 'exit': status, 'error': error})
    (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
    print(name, status, output, flush=True)
    return status


if run('library', ['cargo', 'build', '--locked', '--lib', '--message-format=json-render-diagnostics'], 900):
    raise SystemExit('Consumer library build failed')
library = None
for line in (output / 'library.out').read_text().splitlines():
    try:
        message = json.loads(line)
    except json.JSONDecodeError:
        continue
    if message.get('reason') == 'compiler-artifact' and message.get('target', {}).get('name') == 'reactive_tui':
        library = next(path for path in message['filenames'] if path.endswith('.rlib'))
assert library
binary = root / 'target/api019-input-lifecycle-probe'
if run('consumer-compile', ['rustc', '--edition=2021', 'verification/api-residual/input-lifecycle.rs',
                          '--extern', 'reactive_tui=' + library, '-L', 'dependency=' + str(root / 'target/debug/deps'),
                          '-C', 'link-arg=-Wl,--threads=8', '-o', str(binary)], 180):
    raise SystemExit('Lifecycle consumer compile failed')
status = run('lifecycle', ['python3', '-B', 'verification/api-residual/input-lifecycle.py', str(binary), str(output / 'cases')], 60)
print('INPUT_LIFECYCLE ' + json.dumps(json.loads((output / 'cases/results.json').read_text())), flush=True)
print('Lifecycle diagnostics retained at', output, flush=True)
raise SystemExit(status)
