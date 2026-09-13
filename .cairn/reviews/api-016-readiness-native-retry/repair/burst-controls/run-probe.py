import json, os, signal, subprocess, sys, time
from pathlib import Path
out = Path(__file__).resolve().parent
results = []
for version, platform, limit in [('original','darwin',511), ('corrected','darwin',511), ('corrected','linux',2048), ('corrected','linux',511)]:
    command=[sys.executable, '-B', str(out/'probe-case.py'), version, platform, str(limit)]
    log=out/f'{version}-{platform}-{limit}.out'
    with log.open('wb') as stream:
        child=subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            child.wait(timeout=5)
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait(timeout=2)
    row=dict(command=command, returncode=child.returncode, log=str(log))
    results.append(row)
    (out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
    print(log.read_text(), flush=True)
    assert child.returncode == 0, row
