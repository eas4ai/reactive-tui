import importlib.util
from pathlib import Path
root = Path(__file__).resolve().parents[4]
directory = Path(__file__).resolve().parent
(directory / "release").unlink(missing_ok=True)
spec = importlib.util.spec_from_file_location("entry_points", root / "scripts/check-api-entry-points.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
try:
    module.input_burst_workflow(directory / "input-burst-registry", directory / "unused-screen", directory, directory / "release")
except AssertionError as error:
    print("EXPECTED BASELINE FAILURE:", error)
    captured = (directory / "crossterm-input-burst.bin").read_bytes()
    print("CAPTURE:", repr(captured))
    assert b"COUNT 1024" in captured, "baseline did not expose expected 1024-byte stall"
    assert b"ENTRY_POINT_CLEAN_EXIT" not in captured
else:
    raise AssertionError("unrepaired registry fixture unexpectedly passed")
