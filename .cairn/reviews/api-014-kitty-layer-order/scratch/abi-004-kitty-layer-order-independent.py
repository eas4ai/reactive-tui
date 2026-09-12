import sys, json, ctypes, hashlib
from pathlib import Path
from kitty.options.types import defaults
from kitty.fast_data_types import Screen, set_options

def parse_bytes(screen, data):
    data = memoryview(data)
    while data:
        buffer = screen.test_create_write_buffer()
        consumed = screen.test_commit_write_buffer(data, buffer)
        data = data[consumed:]
        screen.test_parse_written_data(None)

import kitty.fast_data_types as fdt
lib = ctypes.PyDLL(fdt.__file__)
has_images=lib.grman_has_images
has_images.argtypes=[ctypes.c_void_p]; has_images.restype=ctypes.c_bool
def run(query_before_update):
    set_options(defaults)
    screen=Screen(None,33,88,100,9,18,0,None)
    data=Path('/tmp/abi-004-kitty-frames-quiet/prefix.bin').read_bytes()+Path('/tmp/abi-004-kitty-frames-quiet/0.bin').read_bytes()
    parse_bytes(screen,data)
    manager=screen.grman
    image=manager.image_for_client_id(1)
    assert image['root_frame_data_loaded'] and image['refs.count']==1
    result={'image_count':manager.image_count,'loaded':image['root_frame_data_loaded'],'refs':image['refs.count'],'sha256':hashlib.sha256(image['data']).hexdigest()}
    if query_before_update: result['render_with_image_layers']=has_images(id(manager))
    layers=manager.update_layers(0,-1.0,1.0,2.0/88,2.0/33,88,33,9,18)
    if not query_before_update: result['render_with_image_layers']=has_images(id(manager))
    result['prepared_placements']=len(layers)
    result['next_render_with_image_layers']=has_images(id(manager))
    return result
results={'host_order_query_then_prepare':run(True),'corrected_order_prepare_then_query':run(False)}
Path('/tmp/abi-004-kitty-layer-order-independent-results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
assert results['host_order_query_then_prepare']['render_with_image_layers'] is False
assert results['corrected_order_prepare_then_query']['render_with_image_layers'] is True
assert all(row['prepared_placements']==1 for row in results.values())
