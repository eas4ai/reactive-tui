import importlib.util
import json
from collections import Counter
from pathlib import Path
import subprocess
import tempfile
import time
import uuid
from PIL import Image
root = Path.cwd()
spec = importlib.util.spec_from_file_location("host", root / "scripts/check-iterm-host.py")
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)
output = root / "docs/analysis/widget-platforms/darwin/color-diagnostic/native-reference"
output.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix="rtui-native-image-reference-") as work:
    work = Path(work)
    png = work / "input.png"
    fixture = Image.new("RGBA", (56,68))
    fixture.putdata([((255,0,0,255) if x < 28 else (0,0,255,255)) if y < 28 else (0,0,0,0) for y in range(68) for x in range(56)])
    fixture.save(png)
    binary = work / "reference"
    subprocess.run(["swiftc", str(root / ".cairn/reviews/api-011-native-image-reference.swift"), "-o", str(binary)], check=True, timeout=60)
    title = "RTUI-NATIVE-REFERENCE-" + uuid.uuid4().hex
    with (output / "host.log").open("wb") as log:
        child = subprocess.Popen([str(binary), str(png), title], stdout=log, stderr=log)
    try:
        deadline = time.monotonic() + 12
        while True:
            windows = [w for w in host.owned_windows(child.pid, output) if w.get("kCGWindowIsOnscreen") and w.get("kCGWindowName") == title]
            if len(windows) == 1: break
            if child.poll() is not None or time.monotonic() >= deadline: raise RuntimeError("Native reference window missing")
            time.sleep(.1)
        time.sleep(1)
        screenshot = output / "reference.png"
        subprocess.run(["/usr/sbin/screencapture", "-x", "-l", str(windows[0]["kCGWindowNumber"]), str(screenshot)], check=True, timeout=10)
        with Image.open(screenshot) as image:
            colors = Counter(image.convert("RGB").getdata())
        report = {"colors": colors.most_common(20), "pure": host.measurements(screenshot)}
        (output / "pixels.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
    finally:
        if child.poll() is None:
            child.terminate()
            try: child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)
