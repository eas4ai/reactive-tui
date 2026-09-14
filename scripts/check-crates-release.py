#!/usr/bin/env python3
"""Validate the crates.io package graph and archive boundaries."""

from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.1.0"
RUST_VERSION = "1.91"
REPOSITORY = "https://github.com/eas4ai/reactive-tui"

PACKAGES = {
    "root": (Path("Cargo.toml"), "reactive-tui", 700),
    "ghostty-sys": (
        Path("crates/libghostty-vt-sys/Cargo.toml"),
        "reactive-tui-libghostty-vt-sys",
        20,
    ),
    "ghostty": (
        Path("crates/libghostty-vt/Cargo.toml"),
        "reactive-tui-libghostty-vt",
        50,
    ),
    "macros": (Path("reactive-tui-macros/Cargo.toml"), "reactive-tui-macros", 20),
    "crossterm": (
        Path("src/backend/crossterm/Cargo.toml"),
        "reactive-tui-crossterm",
        120,
    ),
    "engine": (
        Path("src/backend/engine/Cargo.toml"),
        "reactive-tui-suprtui",
        120,
    ),
}

REQUIRED_FILES = {
    "root": {"README.md", "CHANGELOG.md", "LICENSE", "build.rs", "src/lib.rs", "manual/README.md"},
    "ghostty-sys": {"README.md", "LICENSE", "UPSTREAM.md", "build.rs", "src/lib.rs"},
    "ghostty": {"README.md", "LICENSE", "UPSTREAM.md", "src/lib.rs"},
    "macros": {"README.md", "LICENSE", "src/lib.rs"},
    "crossterm": {"README.md", "LICENSE", "REACTIVE_TUI_PATCH.md", "src/lib.rs"},
    "engine": {"README.md", "LICENSE", "LICENSE-OpenTUI", "UPSTREAM.md", "src/lib.rs"},
}

LIBRARY_NAMES = {
    "root": "reactive_tui",
    "ghostty-sys": "libghostty_vt_sys",
    "ghostty": "libghostty_vt",
    "macros": "reactive_tui_macros",
    "crossterm": "crossterm",
    "engine": "suprtui",
}


def fail(message: str) -> None:
    raise AssertionError(message)


def load_manifest(path: Path) -> dict:
    with (ROOT / path).open("rb") as source:
        return tomllib.load(source)


def dependency_tables(manifest: dict):
    for table_name in ("dependencies", "build-dependencies"):
        yield table_name, manifest.get(table_name, {})
    for target_name, target in manifest.get("target", {}).items():
        for table_name in ("dependencies", "build-dependencies"):
            yield f"target.{target_name}.{table_name}", target.get(table_name, {})


def check_metadata() -> dict[str, dict]:
    manifests = {key: load_manifest(path) for key, (path, _, _) in PACKAGES.items()}
    for key, manifest in manifests.items():
        package = manifest["package"]
        expected_name = PACKAGES[key][1]
        if package.get("name") != expected_name:
            fail(f"{key}: package name must be {expected_name}")
        if package.get("version") != VERSION:
            fail(f"{key}: version must be {VERSION}")
        if package.get("rust-version") != RUST_VERSION:
            fail(f"{key}: rust-version must be {RUST_VERSION}")
        if package.get("repository") != REPOSITORY:
            fail(f"{key}: repository must be {REPOSITORY}")
        if not package.get("description"):
            fail(f"{key}: description is required")
        if not package.get("readme"):
            fail(f"{key}: readme is required")
        if not package.get("license") and not package.get("license-file"):
            fail(f"{key}: license metadata is required")
        if not package.get("include"):
            fail(f"{key}: an explicit include boundary is required")
        if package.get("publish") is False:
            fail(f"{key}: package is still marked publish = false")
        library_name = manifest.get("lib", {}).get(
            "name", package["name"].replace("-", "_")
        )
        if library_name != LIBRARY_NAMES[key]:
            fail(f"{key}: Rust library name must remain {LIBRARY_NAMES[key]}")

        for table_name, dependencies in dependency_tables(manifest):
            for name, dependency in dependencies.items():
                if not isinstance(dependency, dict):
                    continue
                if "git" in dependency:
                    fail(f"{key}: {table_name}.{name} still uses git")
                if "path" in dependency and "version" not in dependency:
                    fail(f"{key}: {table_name}.{name} has a path without a version")

    root_dependencies = manifests["root"]["dependencies"]
    expected = {
        "crossterm": ("reactive-tui-crossterm", f"={VERSION}"),
        "reactive-tui-macros": (None, f"={VERSION}"),
        "suprtui": ("reactive-tui-suprtui", f"={VERSION}"),
    }
    for alias, (package_name, version) in expected.items():
        dependency = root_dependencies.get(alias)
        if not isinstance(dependency, dict):
            fail(f"root: {alias} must be a detailed dependency")
        if dependency.get("version") != version:
            fail(f"root: {alias} must use exact version {version}")
        if package_name is not None and dependency.get("package") != package_name:
            fail(f"root: {alias} must alias package {package_name}")

    unix_dependencies = manifests["root"]["target"]["cfg(unix)"]["dependencies"]
    ghostty = unix_dependencies.get("libghostty-vt")
    if not isinstance(ghostty, dict):
        fail("root: libghostty-vt must be a detailed dependency")
    if ghostty.get("package") != "reactive-tui-libghostty-vt":
        fail("root: libghostty-vt must alias reactive-tui-libghostty-vt")
    if ghostty.get("version") != f"={VERSION}":
        fail(f"root: libghostty-vt must use exact version ={VERSION}")

    ghostty_dependencies = manifests["ghostty"]["dependencies"]
    ghostty_sys = ghostty_dependencies.get("libghostty-vt-sys")
    if not isinstance(ghostty_sys, dict):
        fail("ghostty: libghostty-vt-sys must be a detailed dependency")
    if ghostty_sys.get("package") != "reactive-tui-libghostty-vt-sys":
        fail("ghostty: libghostty-vt-sys must alias reactive-tui-libghostty-vt-sys")
    if ghostty_sys.get("version") != f"={VERSION}":
        fail(f"ghostty: libghostty-vt-sys must use exact version ={VERSION}")

    changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    if "## [0.1.0] - 2026-09-14" not in changelog:
        fail("CHANGELOG.md needs the dated 0.1.0 release entry")
    return manifests


def archive_files(manifest_path: Path) -> set[str]:
    command = [
        "cargo",
        "package",
        "--list",
        "--manifest-path",
        str(manifest_path),
    ]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        sys.stderr.write(result.stdout)
        sys.stderr.write(result.stderr)
        fail(f"cargo package --list failed for {manifest_path}")
    return {line.strip() for line in result.stdout.splitlines() if line.strip()}


def allowed(key: str, path: str) -> bool:
    automatic = {".cargo_vcs_info.json", "Cargo.lock", "Cargo.toml", "Cargo.toml.orig"}
    if path in automatic or path in REQUIRED_FILES[key]:
        return True
    if key == "root" and path.startswith(("src/backend/crossterm/", "src/backend/engine/")):
        return False
    if path.startswith("src/"):
        return True
    if key == "root":
        return path.startswith("manual/") or path in {
            "tests/api_widget_behavior/accessibility_probe.rs",
            "tests/api_widget_behavior/conpty_probe.rs",
            "tests/api_widget_behavior/image_host_probe.rs",
        }
    if key == "engine":
        return path.startswith("tests/")
    if key == "ghostty-sys":
        return path.startswith("tools/") or path == "build.rs"
    return False


def check_archives() -> None:
    for key, (manifest_path, _, maximum) in PACKAGES.items():
        files = archive_files(manifest_path)
        missing = REQUIRED_FILES[key] - files
        if missing:
            fail(f"{key}: archive misses {sorted(missing)}")
        unexpected = sorted(path for path in files if not allowed(key, path))
        if unexpected:
            fail(f"{key}: unexpected archive paths: {unexpected[:12]}")
        if len(files) > maximum:
            fail(f"{key}: archive has {len(files)} files; limit is {maximum}")
        print(f"{key}: {len(files)} bounded archive files")


def main() -> int:
    check_metadata()
    check_archives()
    print("publish order: reactive-tui-libghostty-vt-sys, reactive-tui-libghostty-vt, reactive-tui-macros, reactive-tui-crossterm, reactive-tui-suprtui, reactive-tui")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f"release check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
