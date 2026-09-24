#!/usr/bin/env python3
"""Prove opt-in GPU dependency wiring and minimum-toolchain consumers."""

import copy
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
NATIVE_FEATURES = {"std", "parking_lot", "vulkan", "metal", "dx12", "wgsl"}


def validate_manifest(manifest):
    dependency = manifest.get("dependencies", {}).get("wgpu", {})
    features = manifest.get("features", {})
    if not isinstance(dependency, dict) or dependency.get("optional") is not True:
        raise ValueError("wgpu must be an optional dependency")
    if features.get("wgpu-graphics") != ["dep:wgpu"]:
        raise ValueError("missing explicit opt-in wiring")
    pending = list(features.get("default", []))
    visited = set()
    while pending:
        feature = pending.pop()
        if feature in {"wgpu-graphics", "wgpu", "dep:wgpu"} or feature.startswith("wgpu/"):
            raise ValueError("default features enable wgpu")
        if feature not in visited:
            visited.add(feature)
            pending.extend(features.get(feature, []))
    if dependency.get("default-features") is not False:
        raise ValueError("explicit native features required")
    if not NATIVE_FEATURES <= set(dependency.get("features", [])):
        raise ValueError("native backend or shader feature missing")


def validate_default_tree(tree):
    if any(line.startswith("wgpu v") for line in tree.splitlines()):
        raise ValueError("default dependency graph includes wgpu")


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        self.manifest = {
            "dependencies": {"wgpu": {
                "optional": True, "default-features": False,
                "features": sorted(NATIVE_FEATURES),
            }},
            "features": {"default": ["tokio"], "wgpu-graphics": ["dep:wgpu"]},
        }

    def test_corrected_manifest_passes(self):
        validate_manifest(self.manifest)

    def test_missing_dependency_fails(self):
        self.manifest["dependencies"].clear()
        with self.assertRaisesRegex(ValueError, "optional"):
            validate_manifest(self.manifest)

    def test_nonoptional_dependency_fails(self):
        self.manifest["dependencies"]["wgpu"]["optional"] = False
        with self.assertRaisesRegex(ValueError, "optional"):
            validate_manifest(self.manifest)

    def test_missing_opt_in_fails(self):
        self.manifest["features"]["wgpu-graphics"] = []
        with self.assertRaisesRegex(ValueError, "opt-in"):
            validate_manifest(self.manifest)

    def test_default_alias_fails(self):
        self.manifest["features"].update(default=["alias"], alias=["wgpu-graphics"])
        with self.assertRaisesRegex(ValueError, "default"):
            validate_manifest(self.manifest)

    def test_default_dependency_feature_fails(self):
        self.manifest["features"]["default"] = ["wgpu/vulkan"]
        with self.assertRaisesRegex(ValueError, "default"):
            validate_manifest(self.manifest)

    def test_missing_native_features_fail(self):
        for feature in NATIVE_FEATURES:
            with self.subTest(feature=feature):
                candidate = copy.deepcopy(self.manifest)
                candidate["dependencies"]["wgpu"]["features"].remove(feature)
                with self.assertRaisesRegex(ValueError, "feature missing"):
                    validate_manifest(candidate)

    def test_implicit_defaults_fail(self):
        self.manifest["dependencies"]["wgpu"].pop("default-features")
        with self.assertRaisesRegex(ValueError, "explicit"):
            validate_manifest(self.manifest)

    def test_default_tree_rejects_wgpu(self):
        with self.assertRaisesRegex(ValueError, "graph includes"):
            validate_default_tree("reactive-tui v0.1.0\nwgpu v27.0.1\n")
        validate_default_tree("reactive-tui v0.1.0\ntaffy v0.9.1\n")


def run(command, capture=False):
    print("RUN " + " ".join(command), flush=True)
    result = subprocess.run(
        command, cwd=ROOT, env={**os.environ, "CARGO_TERM_COLOR": "never"},
        text=True, stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.STDOUT if capture else None, check=True, timeout=1200,
    )
    if capture:
        print(result.stdout, flush=True)
    return result.stdout or ""


def main():
    tests = unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests)
    if not unittest.TextTestRunner(verbosity=2).run(tests).wasSuccessful():
        return 1
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    validate_manifest(manifest)
    minimum = manifest["package"]["rust-version"]
    cargo = ["cargo", f"+{minimum}.0"]
    tree = run(cargo + ["tree", "--locked", "-p", "reactive-tui", "--edges", "normal", "--prefix", "none"], capture=True)
    validate_default_tree(tree)
    for flags in ([], ["--features", "wgpu-graphics"]):
        run(cargo + ["check", "--locked", "-p", "reactive-tui", "--lib", *flags])
        output = run(cargo + ["test", "--locked", "-p", "reactive-tui", "--test", "api_feature_configurations", "--test", "widget_catalog_behavior", *flags], capture=True)
        if len(re.findall(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", output)) != 2:
            raise ValueError("both consumer test targets must execute passing tests")
        run(cargo + ["build", "--locked", "-p", "reactive-tui", "--example", "widget_catalog", *flags])
    print(f"PASS GPU-001: optional graph and existing consumers on Rust {minimum}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
