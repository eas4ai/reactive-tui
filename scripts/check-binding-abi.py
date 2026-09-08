#!/usr/bin/env python3
"""Inventory native declarations without invoking potentially mismatched functions."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


def main():
    subprocess.run(["cargo", "build", "--locked", "--features", "ffi"], check=True, timeout=300)
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    library = target / "debug/libreactive_tui.so"
    symbols = subprocess.check_output(["nm", "-D", "--defined-only", "--format=posix", str(library)], text=True)
    exports = {line.split()[0] for line in symbols.splitlines() if line.split()[1] in {"T", "W"}}
    headers = sorted(Path("include/reactive_tui").glob("*.h"))
    with tempfile.TemporaryDirectory(prefix="abi-inventory-", dir=target) as scratch:
        source = Path(scratch) / "headers.c"
        source.write_text("".join(f'#include "reactive_tui/{p.name}"\n' for p in headers))
        result = subprocess.run([
            "clang", "-Iinclude", "-Xclang", "-ast-dump=json", "-fsyntax-only", str(source),
        ], capture_output=True, text=True, timeout=60)
        declarations = {}
        def visit(node):
            if node.get("kind") == "FunctionDecl" and node.get("name"):
                declarations[node["name"]] = node.get("type", {}).get("qualType", "")
            for child in node.get("inner", []):
                visit(child)
        visit(json.loads(result.stdout))
        loader = Path("bindings/typescript/src/ffi.ts").read_text()
        typescript = sorted(set(re.findall(r"['\"](rtui_\w+)['\"]\s*:", loader)))
        missing_c = sorted(set(declarations) - exports)
        missing_ts = sorted(set(typescript) - exports)
        print(json.dumps({"c_declarations": declarations, "typescript_symbols": typescript,
                          "rust_exports": sorted(exports), "missing_c": missing_c,
                          "missing_typescript": missing_ts}, indent=2))
        if result.returncode:
            print(result.stderr)
        if missing_c or missing_ts or result.returncode:
            print("ABI declaration inventory failed; no native functions were invoked")
            return 1
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
