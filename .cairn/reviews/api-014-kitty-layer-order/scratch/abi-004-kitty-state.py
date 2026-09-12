import json, hashlib
from kittens.tui.handler import result_handler
def main(args):
    pass
@result_handler(no_ui=True)
def handle_result(args, answer, target_window_id, boss):
    window = boss.window_id_map[target_window_id]
    manager = window.screen.grman
    result = {'image_count': manager.image_count, 'lines': window.screen.lines, 'columns': window.screen.columns}
    if manager.image_count:
        img = manager.image_for_client_id(1)
        if img:
            data = img.pop('data')
            img['bytes'] = len(data)
            img['sha256'] = hashlib.sha256(data).hexdigest()
        result['image_1'] = img
    return json.dumps(result)
