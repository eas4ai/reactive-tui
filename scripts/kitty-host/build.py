#!/usr/bin/env python3
"""Build and verify the approved isolated Kitty image host on Linux."""

import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import runpy
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PIN = json.loads((HERE / "source.json").read_text())
PATCH = HERE / "image-layer-order.patch"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def runtime_files(source):
    files = {}
    for path in sorted(source.rglob("*")):
        relative = path.relative_to(source)
        if relative.parts[0] == "build" or not path.is_file():
            continue
        if not path.resolve().is_relative_to(source):
            raise RuntimeError("Kitty runtime escapes its isolated directory: " + str(path))
        files[str(relative)] = digest(path)
    return files


def prepare_source(cache, archive, recipe, execute):
    source = cache / PIN["directory"]
    prepared = cache / "prepared.json"
    if source.exists():
        if not prepared.exists() or json.loads(prepared.read_text()) != recipe:
            raise RuntimeError("Unrecognized incomplete Kitty build: " + str(cache))
        return source

    saved_archive = cache / "source.tar.gz"
    if not saved_archive.exists():
        pending = cache / "source.download"
        if archive:
            shutil.copyfile(archive, pending)
        else:
            with urllib.request.urlopen(PIN["url"], timeout=30) as response:
                with pending.open("wb") as output:
                    shutil.copyfileobj(response, output)
        if digest(pending) != PIN["sha256"]:
            raise RuntimeError("Kitty source archive digest mismatch: " + str(pending))
        pending.replace(saved_archive)
    if digest(saved_archive) != PIN["sha256"]:
        raise RuntimeError("Cached Kitty source archive digest mismatch")

    with tempfile.TemporaryDirectory(prefix="extract-", dir=cache) as temporary:
        temporary = Path(temporary)
        with tarfile.open(saved_archive) as package:
            package.extractall(temporary, filter="data")
        extracted = temporary / PIN["directory"]
        execute(["patch", "--batch", "--fuzz=0", "-p1", "-d", str(extracted),
                 "-i", str(PATCH)], cache / "patch.out", 30)
        extracted.rename(source)
    prepared.write_text(json.dumps(recipe, indent=2) + "\n")
    return source


def verify(cache, recipe):
    record = json.loads((cache / "build.json").read_text())
    if record["recipe"] != recipe:
        raise RuntimeError("Kitty build recipe does not match this checkout")
    source = cache / PIN["directory"]
    actual = runtime_files(source)
    if not actual or actual != record["files"]:
        raise RuntimeError("Kitty runtime integrity mismatch: " + str(cache))
    executable = source / "kitty/launcher/kitty"
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise RuntimeError("Verified Kitty executable is missing or not executable")
    return executable, record


def ensure_host(archive=None, jobs=12, verify_only=False):
    if platform.system() != "Linux":
        raise RuntimeError("The isolated Kitty host build is for Linux acceptance")
    if not 1 <= jobs <= 12:
        raise ValueError("Kitty build jobs must be between 1 and 12")
    recipe = {
        "source": PIN,
        "builder_sha256": digest(Path(__file__)),
        "patch_sha256": digest(PATCH),
        "machine": platform.machine(),
        "python": "/usr/bin/python3",
        "build_flags": ["build", "--vcs-rev", "rtui-image-order-" + digest(PATCH)[:12]],
        "lto": False,
    }
    key = hashlib.sha256(json.dumps(recipe, sort_keys=True).encode()).hexdigest()[:20]
    cache = ROOT / "target/kitty-image-host" / key
    cache.mkdir(parents=True, exist_ok=True)
    with (cache / "build.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if (cache / "build.json").exists():
            return verify(cache, recipe)
        if verify_only:
            raise RuntimeError("No verified Kitty host build: " + str(cache))
        execute = runpy.run_path(str(ROOT / "scripts/check-widget-platforms.py"))["execute"]
        cpu_count = execute(["env", "PYTHON_CPU_COUNT=" + str(jobs), "/usr/bin/python3",
                             "-c", "import os; print(os.cpu_count())"],
                            cache / "cpu-count.out", 10).strip()
        if cpu_count != str(jobs):
            raise RuntimeError("/usr/bin/python3 must support PYTHON_CPU_COUNT to cap Kitty workers")
        source = prepare_source(cache, archive, recipe, execute)
        # Distro Kitty may ship its required font privately, outside fontconfig.
        # Copy that dependency with its license; runtime hashes retain its identity.
        packaged_fonts = Path("/usr/lib/kitty/fonts")
        if not (source / "fonts/SymbolsNerdFontMono-Regular.ttf").exists():
            if (packaged_fonts / "SymbolsNerdFontMono-Regular.ttf").is_file():
                (source / "fonts").mkdir(exist_ok=True)
                for name in ("SymbolsNerdFontMono-Regular.ttf", "LICENSE"):
                    shutil.copyfile(packaged_fonts / name, source / "fonts" / name)
        command = [
            "env", "--chdir=" + str(source), "PYTHON_CPU_COUNT=" + str(jobs),
            "GOMAXPROCS=" + str(jobs), "GOFLAGS=-p=" + str(jobs),
            "PYTHONDONTWRITEBYTECODE=1", "KITTY_NO_LTO=1",
            "/usr/bin/python3", "setup.py", *recipe["build_flags"],
        ]
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        build_log = cache / ("build-" + stamp + ".out")
        print("Building isolated Kitty; log: " + str(build_log), flush=True)
        execute(command, build_log, 1200)
        executable = source / "kitty/launcher/kitty"
        version = execute([str(executable), "--version"], cache / "version.out", 10)
        if "kitty " + PIN["version"] not in version:
            raise RuntimeError("Unexpected isolated Kitty version: " + version)
        record = {
            "recipe": recipe, "jobs": jobs, "version": version.strip(),
            "build_command": command, "files": runtime_files(source),
            "build_log": build_log.name, "build_log_sha256": digest(build_log),
            "patch_log_sha256": digest(cache / "patch.out"),
            "compiler": subprocess.check_output(["cc", "--version"], text=True).splitlines()[0],
            "go": subprocess.check_output(["go", "version"], text=True).strip(),
            "build_environment": {name: os.environ[name] for name in
                                  ("CC", "CFLAGS", "CPPFLAGS", "LDFLAGS", "PKG_CONFIG_PATH")
                                  if name in os.environ},
        }
        pending = cache / "build.pending.json"
        pending.write_text(json.dumps(record, indent=2) + "\n")
        pending.replace(cache / "build.json")
        return verify(cache, recipe)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, help="optional local copy of the pinned archive")
    parser.add_argument("--jobs", type=int, default=12)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    host, record = ensure_host(args.archive, args.jobs, args.verify_only)
    print(str(host))
    print("Verified Kitty runtime files: " + str(len(record["files"])))
