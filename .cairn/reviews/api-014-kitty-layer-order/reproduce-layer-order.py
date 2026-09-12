"""Reproduce Kitty 0.45's stale image-layer predicate; this is not acceptance."""

import ctypes
import hashlib
import json
from pathlib import Path

import kitty.fast_data_types as native
from kitty.options.types import defaults


def parse(screen, data):
    remaining = memoryview(data)
    while remaining:
        buffer = screen.test_create_write_buffer()
        consumed = screen.test_commit_write_buffer(remaining, buffer)
        assert consumed > 0
        remaining = remaining[consumed:]
        screen.test_parse_written_data(None)


def inspect_order(prepare_first):
    native.set_options(defaults)
    screen = native.Screen(None, 33, 88, 100, 9, 18, 0, None)
    parse(screen, Path(__file__).with_name("first-image.bin").read_bytes())
    manager = screen.grman
    image = manager.image_for_client_id(1)
    assert image["root_frame_data_loaded"] and image["refs.count"] == 1

    # PyDLL keeps the GIL while this installed-library diagnostic reads the
    # native object. The manager reference owns it for every call below.
    library = ctypes.PyDLL(native.__file__)
    has_images = library.grman_has_images
    has_images.argtypes = [ctypes.c_void_p]
    has_images.restype = ctypes.c_bool

    result = {
        "image_count": manager.image_count,
        "loaded": image["root_frame_data_loaded"],
        "refs": image["refs.count"],
        "sha256": hashlib.sha256(image["data"]).hexdigest(),
    }
    assert result["sha256"] == (
        "1d0c0a26ec059fa2d7d813ecbe3f7e10184b20aeb44b544b54590fbad3595ea4"
    )
    if not prepare_first:
        result["render_with_image_layers"] = has_images(id(manager))
    layers = manager.update_layers(0, -1.0, 1.0, 2.0 / 88, 2.0 / 33, 88, 33, 9, 18)
    if prepare_first:
        result["render_with_image_layers"] = has_images(id(manager))
    result["prepared_placements"] = len(layers)
    result["next_render_with_image_layers"] = has_images(id(manager))
    return result


results = {
    "host_order_query_then_prepare": inspect_order(False),
    "corrected_order_prepare_then_query": inspect_order(True),
}
print(json.dumps(results, indent=2))
assert results["host_order_query_then_prepare"]["render_with_image_layers"] is False
assert results["corrected_order_prepare_then_query"]["render_with_image_layers"] is True
assert all(row["prepared_placements"] == 1 for row in results.values())
