#!/usr/bin/env python3
"""Inventory native declarations without invoking potentially mismatched functions."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

from abi.compiler_probe import run_probe
from abi.typescript_schema import schema_from_ast, typescript_names
from abi.loader_probe import run_loader_probe

BASELINES = Path("scripts/abi/baselines")


def main():
    migration_text = Path("bindings/typescript/MIGRATION.md").read_text()
    required_policy = (
        "The compiled Rust exports are the compatibility baseline.",
        "Never reuse a consumed pointer",
        "library retains the allocation metadata",
        "another thread may call `rtui_app_quit`",
        "Optional root and effect-cleanup callbacks may be NULL",
    )
    missing_policy = [text for text in required_policy if text not in migration_text]
    if missing_policy:
        raise RuntimeError("Packaged ABI ownership policy is incomplete: " + repr(missing_policy))
    subprocess.run(["cargo", "build", "--locked", "--features", "ffi"], check=True, timeout=300)
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    library = target / "debug/libreactive_tui.so"
    symbols = subprocess.check_output(["nm", "-D", "--defined-only", "--format=posix", str(library)], text=True)
    exports = {line.split()[0] for line in symbols.splitlines() if line.split()[1] in {"T", "W"}}
    baseline = json.loads((BASELINES / "binding-abi-baseline.json").read_text())
    missing_original = set(baseline["rust_exports"]) - exports
    if missing_original:
        raise RuntimeError("Existing Rust exports removed: " + repr(missing_original))
    headers = sorted(Path("include/reactive_tui").glob("*.h"))
    with tempfile.TemporaryDirectory(prefix="abi-inventory-", dir=target) as scratch:
        scratch = Path(scratch)
        native_ast = subprocess.run([
            "clang", "-x", "c", "-std=c11", "-pedantic-errors", "-Xclang",
            "-ast-dump=json", "-fsyntax-only", "include/reactive_tui/native.h",
        ], capture_output=True, text=True, check=True, timeout=60)
        audited = run_probe(json.loads(native_ast.stdout), target, scratch)
        source = Path(scratch) / "headers.c"
        source.write_text("".join(f'#include "reactive_tui/{p.name}"\n' for p in headers)
                          + "".join(f'typedef {name} ABI_inventory_{name};\n'
                                    for name in baseline["c_type_aliases"]))
        compatibility = json.loads(Path("scripts/abi/compatibility.json").read_text())
        assertions = ['#ifdef __cplusplus', '#define ABI_ASSERT static_assert', '#else',
                      '#define ABI_ASSERT _Static_assert', '#endif']
        assertions.extend(f'ABI_ASSERT({old} == {new}, "{old}");'
                          for old, new in compatibility["enum_aliases"].items())
        for name in compatibility["retired_enum_constants"]:
            assertions.extend([f'#ifdef {name}', '#error Invalid legacy enum exposed', '#endif'])
        source.write_text(source.read_text() + '\n'.join(assertions) + '\n')
        result = subprocess.run([
            "clang", "-Iinclude", "-Xclang", "-ast-dump=json", "-fsyntax-only", str(source),
        ], capture_output=True, text=True, timeout=60)
        declarations = {}
        current_file = ""
        for node in json.loads(result.stdout)["inner"]:
            current_file = node.get("loc", {}).get("file", current_file)
            if (node.get("kind") == "FunctionDecl" and node.get("name")
                    and "include/reactive_tui/" in current_file):
                declarations[node["name"]] = node.get("type", {}).get("qualType", "")
        if set(declarations) != set(audited["signatures"]):
            raise RuntimeError("Header callable inventory differs from the compiler-audited API: "
                               + repr(set(declarations) ^ set(audited["signatures"])))
        subprocess.run([
            "clang++", "-x", "c++", "-std=c++17", "-pedantic-errors", "-Iinclude",
            "-fsyntax-only", str(source),
        ], check=True, timeout=60)
        schema = json.loads(Path("bindings/typescript/src/native-api.json").read_text())
        if schema != schema_from_ast(json.loads(native_ast.stdout)):
            raise RuntimeError("TypeScript native schema differs from compiler-audited declarations")
        if Path('bindings/typescript/src/native-types.ts').read_text() != typescript_names(schema):
            raise RuntimeError('Public native function type names differ from the audited schema')
        native_baseline = json.loads((BASELINES / "binding-native-baseline.json").read_text())
        for section in ("functions", "records", "callbacks"):
            for name, definition in native_baseline["schema"][section].items():
                if schema[section].get(name) != definition:
                    raise RuntimeError("Existing native ABI changed: " + name)
        for name, value in native_baseline["enum_values"].items():
            if f"enum {name} {value}" not in audited["layouts"]:
                raise RuntimeError("Existing native enum changed: " + name)
        run_loader_probe(schema, audited, target, scratch)
        subprocess.run(['node', 'scripts/abi/typescript-public-api.cjs'], check=True, timeout=60)
        migration = json.loads((BASELINES / "binding-abi-migration.json").read_text())
        previous_loader = json.loads(
            (BASELINES / "binding-typescript-native-baseline.json").read_text()
        )
        if set(previous_loader['functions']) != set(baseline['typescript_symbols']):
            raise RuntimeError('Original TypeScript signature inventory is incomplete')
        for name, signature in previous_loader['functions'].items():
            if migration['typescript_functions'].get(name, {}).get('previous') != signature:
                raise RuntimeError('Original TypeScript declaration was not retained in migration data: ' + name)
        for section, original in (("c_functions", baseline["c_declarations"]),
                                  ("typescript_functions", baseline["typescript_symbols"]),
                                  ("c_types", baseline["c_type_aliases"])):
            if set(migration[section]) != set(original):
                raise RuntimeError("Incomplete migration inventory: " + section)
            for name, item in migration[section].items():
                if not item.get("guidance"):
                    raise RuntimeError("Missing migration guidance: " + name)
                if section != "c_types":
                    current = schema["functions"].get(name)
                    if item["current"] != current or item["status"] != ("reconciled" if current else "retired"):
                        raise RuntimeError("Stale migration inventory: " + name)
        typescript = sorted(schema["functions"])
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
