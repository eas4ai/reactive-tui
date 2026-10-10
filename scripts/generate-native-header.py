#!/usr/bin/env python3
"""Generate C declarations from Rust source; the ABI gate verifies compiled types."""
import os
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
from abi.typescript_schema import schema_from_ast, typescript_names


def main():
    tool = Path(os.environ.get("RTUI_CBINDGEN", "target/abi-tools/bin/cbindgen")).resolve()
    if not tool.is_file():
        raise SystemExit("Install the header tool: cargo install cbindgen --version 0.29.4 --locked --root target/abi-tools")
    version = subprocess.check_output([str(tool), "--version"], text=True).strip()
    if version != "cbindgen 0.29.4":
        raise SystemExit(f"Expected cbindgen 0.29.4, got {version}")
    if sys.argv[1:] not in ([], ["--verify"]):
        raise SystemExit("Usage: generate-native-header.py [--verify]")
    command = [str(tool), "--quiet", "--config", "scripts/abi/cbindgen.toml", "--crate", "reactive-tui", "--output", "include/reactive_tui/native.h"]
    command.extend(sys.argv[1:])
    env = os.environ.copy()
    env.setdefault("RUSTC_BOOTSTRAP", "1")
    try:
        env["CARGO_BUILD_JOBS"] = str(min(8, max(1, int(env.get("CARGO_BUILD_JOBS", "8")))))
    except ValueError:
        env["CARGO_BUILD_JOBS"] = "8"
    result = subprocess.call(command, env=env, timeout=300)
    if result:
        return result
    ast = subprocess.check_output([
        'clang', '-x', 'c', '-Xclang', '-ast-dump=json', '-fsyntax-only',
        'include/reactive_tui/native.h',
    ], text=True, timeout=60)
    schema = schema_from_ast(json.loads(ast))
    outputs = {
        'bindings/typescript/src/native-api.json': json.dumps(schema, indent=2) + '\n',
        'bindings/typescript/src/native-types.ts': typescript_names(schema),
    }
    for filename, content in outputs.items():
        path = Path(filename)
        if '--verify' in sys.argv:
            if not path.exists() or path.read_text() != content:
                raise SystemExit('Generated declaration differs: ' + filename)
        else:
            path.write_text(content)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
