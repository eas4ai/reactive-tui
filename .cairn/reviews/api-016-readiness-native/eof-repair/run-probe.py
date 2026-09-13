import json, subprocess, sys, time
from pathlib import Path
out = Path(__file__).resolve().parent
results = []
for version, case, expected in [
    ('original', 'eof', 'timeout'),
    ('corrected', 'eof', 'EOF_DRAIN_RETURNED_AND_RESTORATION_PASSED'),
    ('corrected', 'raw-leak', 'RAW_RESTORATION_REJECTED'),
    ('corrected', 'never-drained', 'DRAIN_DEADLINE_REJECTED'),
]:
    command = [sys.executable, '-B', str(out/'consumer.py'), str(out/(version+'.py')), case, str(out/(version+'-'+case+'.bin'))]
    started = time.monotonic()
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    timed_out = False
    try:
        output, _ = child.communicate(timeout=3)
    except subprocess.TimeoutExpired:
        timed_out = True
        child.kill()
        output, _ = child.communicate(timeout=2)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait(timeout=2)
    row = dict(version=version, case=case, command=command, timed_out=timed_out,
               elapsed_seconds=time.monotonic()-started, returncode=child.returncode, output=output.decode())
    results.append(row)
    (out/(version+'-'+case+'.out')).write_bytes(output)
    (out/'results.json').write_text(json.dumps(results, indent=2)+'\n')
    if expected == 'timeout':
        assert timed_out, row
    else:
        assert not timed_out and child.returncode == 0 and expected in row['output'], row
    print(version, case, 'PASS', flush=True)
