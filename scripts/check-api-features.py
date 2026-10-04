#!/usr/bin/env python3
"""Require independent Cargo feature builds and runtime behavior."""
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FEATURES = ("tokio", "tokio-util", "debug", "debug_patches", "async-capabilities", "ffi", "embedded-terminal", "wgpu-graphics")


def main():
    os.chdir(ROOT)
    environment = {**os.environ, "CARGO_INCREMENTAL": "0", "CARGO_TERM_COLOR": "never"}
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version=1"], text=True,
    ))
    package = next(p for p in metadata["packages"] if p["name"] == "reactive-tui")
    if set(package["features"]) != {"default", "simd", *FEATURES}:
        raise RuntimeError("Update feature acceptance for the changed Cargo feature catalog")
    configurations = [([], []), ([], ["--no-default-features"])]
    configurations += [([], ["--no-default-features", "--features", feature]) for feature in FEATURES]
    configurations += [([], ["--no-default-features", "--features", ",".join(FEATURES)]),
                       (["+nightly"], ["--no-default-features", "--features", "simd"]),
                       (["+nightly"], ["--all-features"])]
    for toolchain, flags in configurations:
        cargo = ["cargo", *toolchain]
        print("Feature configuration: " + " ".join(toolchain + flags or ["default"]), flush=True)
        subprocess.run(cargo + ["check", "--locked", "--all-targets", *flags],
                       env=environment, check=True, timeout=600)
        result = subprocess.run(cargo + ["test", "--locked", "--test", "api_feature_configurations", *flags],
                                env=environment, capture_output=True, text=True, timeout=600)
        print(result.stdout, result.stderr, flush=True)
        if result.returncode or not re.search(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", result.stdout):
            raise RuntimeError("Feature behavior did not execute passing tests")
    for flags in ([], ["--no-default-features"]):
        subprocess.run(["cargo", "test", "--locked", "--test", "api_component_expansion", *flags],
                       env=environment, check=True, timeout=300)
    print("PASS API-015: feature catalog, independent builds and runtime behavior", flush=True)


if __name__ == "__main__":
    main()
