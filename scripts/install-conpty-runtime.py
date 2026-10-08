#!/usr/bin/env python3
"""Install the pinned, offline ConPTY bundle beside a Windows executable.

The destination is an application directory, not the executable filename.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

SOURCE = Path(__file__).resolve().parents[1] / "src/terminal/pty/windows/runtime"


def install(application_directory, architecture):
    manifest = json.loads((SOURCE / "manifest.json").read_text())
    members = [(f"{architecture}/conpty.dll", "conpty.dll")]
    members.extend((f"{arch}/OpenConsole.exe", f"{arch}/OpenConsole.exe")
                   for arch in ["x86", "x64", "arm64"])
    # Validate the complete source before writing any destination files.
    for source, _ in members:
        if hashlib.sha256((SOURCE / source).read_bytes()).hexdigest() != manifest["files"][source]:
            raise RuntimeError("ConPTY source digest mismatch: " + source)
    destination = Path(application_directory).resolve() / "reactive-tui-conpty"
    for source, target in members:
        target = destination / target
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(SOURCE / source, target)
    for name in ["LICENSE", "manifest.json", "README.md"]:
        shutil.copyfile(SOURCE / name, destination / name)
    print(f"Installed Microsoft ConPTY {manifest['version']} ({architecture}) in {destination}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("application_directory", type=Path)
    parser.add_argument("--arch", choices=["x86", "x64", "arm64"], required=True,
                        help="architecture of the application executable")
    args = parser.parse_args()
    install(args.application_directory, args.arch)
