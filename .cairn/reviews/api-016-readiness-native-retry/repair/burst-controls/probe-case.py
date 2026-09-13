import json, os, runpy, sys, types
from pathlib import Path
out = Path(__file__).resolve().parent
version, platform, limit = sys.argv[1:]
limit = int(limit)
case = f'{version}-{platform}-{limit}'
capture = out / case
capture.mkdir(exist_ok=True)
release = capture / 'release'
release.unlink(missing_ok=True)
module = runpy.run_path(str(out / (version+'.py')))
workflow = module['input_burst_workflow']
globals_ = workflow.__globals__
globals_['sys'] = types.SimpleNamespace(**{**vars(sys), 'platform': platform})
real_write = os.write
writes = []
def limited_write(fd, data):
    # A legal short write is supplied by this test transport, not measured on macOS.
    if release.exists() and len(writes) == 1:
        writes.append(dict(released=True, would_block=True))
        raise BlockingIOError()
    sent = real_write(fd, data[:limit])
    writes.append(dict(released=release.exists(), bytes=sent))
    return sent
globals_['os'] = types.SimpleNamespace(**{**vars(os), 'write': limited_write})
try:
    workflow(out / 'consumer.py', None, capture, release)
except AssertionError as error:
    assert version == 'original' or platform == 'linux', error
    assert limit < 2048 and not release.exists(), 'queued control was weakened'
    print('EXPECTED_SHORT_WRITE_REJECTION', repr(str(error)), flush=True)
else:
    assert version == 'corrected'
    assert sum(write.get('bytes',0) for write in writes) == 2048
    if platform == 'linux':
        assert writes == [dict(released=False, bytes=2048)]
        print('LINUX_SINGLE_WRITE_2048_RETAINED', flush=True)
    else:
        assert writes[0] == dict(released=False, bytes=limit)
        assert all(write['released'] for write in writes[1:])
        print('STREAMED_SHORT_WRITES_DELIVERED_2048', flush=True)
finally:
    (capture / 'writes.json').write_text(json.dumps(writes, indent=2)+'\n')
