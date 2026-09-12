import json, sys, time, hashlib
from kitty.fast_data_types import add_timer
started = set()
def on_resize(boss, window, data):
    print('WATCH resize', time.monotonic(), repr(data), 'images', window.screen.grman.image_count, file=sys.stderr, flush=True)
    if window.id in started: return
    started.add(window.id)
    def capture(timer_id):
        try:
            manager = window.screen.grman
            result = {'time': time.monotonic(), 'image_count': manager.image_count, 'lines': window.screen.lines, 'columns': window.screen.columns}
            if manager.image_count:
                img = manager.image_for_client_id(1)
                if img:
                    data = img.pop('data')
                    img['bytes'] = len(data)
                    img['sha256'] = hashlib.sha256(data).hexdigest()
                result['image_1'] = img
            print('WATCH state', json.dumps(result), file=sys.stderr, flush=True)
        except Exception as error:
            print('WATCH error', repr(error), file=sys.stderr, flush=True)
    add_timer(capture, 2.5, False)
