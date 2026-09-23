"""Compare C declarations with types inferred independently by rustc."""
import json
from pathlib import Path
import re
import subprocess

RENAMES = {"ReactiveError": "RTuiError", "ReactiveTerminal": "RTuiTerminal", "Capabilities": "RTuiCapabilities"}
PRIMITIVES = {"uint8_t": "u8", "uint16_t": "u16", "uint32_t": "u32", "uint64_t": "u64",
              "int8_t": "i8", "int16_t": "i16", "int32_t": "i32", "int64_t": "i64",
              "size_t": "usize", "float": "f32", "double": "f64", "_Bool": "bool",
              "char": "c_char", "void": "c_void", "int": "i32", "unsigned int": "u32"}


def split_args(value):
    result, start, depth = [], 0, 0
    for index, char in enumerate(value):
        if char in "(<[": depth += 1
        elif char in ")>]" and not (char == ">" and index and value[index - 1] == "-"): depth -= 1
        elif char == "," and depth == 0:
            result.append(value[start:index].strip())
            start = index + 1
    if value[start:].strip(): result.append(value[start:].strip())
    return result


def c_type(value, aliases):
    value = re.sub(r"\b(struct|enum|union) ", "", value.strip())
    if value in aliases:
        return c_type(aliases[value], aliases)
    callback = re.fullmatch(r"(.+?)\s*\(\*\)\((.*)\)", value)
    if callback:
        return "callback(" + ",".join(c_type(a, aliases) for a in split_args(callback[2]) if a != "void") + ")->" + ("()" if callback[1] == "void" else c_type(callback[1], aliases))
    if value.endswith("*"):
        inner = value[:-1].strip()
        const = inner.endswith("const") or (inner.startswith("const ") and not inner.endswith("*"))
        return ("*const " if const else "*mut ") + c_type(inner.removesuffix("const").strip() if inner.endswith("const") else inner.removeprefix("const ") if const else inner, aliases)
    return PRIMITIVES.get(value, value)


def rust_type(value):
    value = re.sub(r"(?:[A-Za-z_][A-Za-z_0-9]*::)+([A-Za-z_][A-Za-z_0-9]*)", r"\1", value)
    for old, new in RENAMES.items(): value = re.sub(r"\b" + old + r"\b", new, value)
    value = re.sub(r"(\*const |\*mut )i8\b", r"\1c_char", value)
    if value.startswith("Option<") and value.endswith(">"):
        value = value[7:-1]
    match = re.fullmatch(r'(?:unsafe )?extern "C" fn\((.*)\)(?: -> (.*))?', value)
    if match:
        return "callback(" + ",".join(rust_type(a) for a in split_args(match[1])) + ")->" + rust_type(match[2] or "()")
    return value


def run_probe(ast, target, scratch):
    nodes = [n for n in ast["inner"] if not n.get("loc", {}).get("includedFrom")]
    functions = [n for n in nodes if n.get("kind") == "FunctionDecl"]
    records = [n for n in nodes if n.get("kind") == "RecordDecl" and n.get("completeDefinition") and n.get("name", "").startswith("RTui")]
    aliases = {n["name"]: n["type"]["qualType"] for n in nodes if n.get("kind") == "TypedefDecl" and "(*)(" in n.get("type", {}).get("qualType", "")}
    rust = ['use reactive_tui::ffi::*;', 'fn main() {']
    c = ['#include "reactive_tui/native.h"', '#include <stdio.h>', 'int main(void) {']
    for n in functions:
        args = [x for x in n.get("inner", []) if x.get("kind") == "ParmVarDecl"]
        rust.append('let f: unsafe extern "C" fn(' + ','.join('_' for _ in args) + ') -> _ = ' + n['name'] + ';')
        rust.append('println!("fn ' + n['name'] + ' {}", std::any::type_name_of_val(&f));')
    for n in records:
        name = n['name']
        rust_name = next((k for k, v in RENAMES.items() if v == name), name)
        rust.append(f'println!("layout {name} {{}} {{}}", size_of::<{rust_name}>(), align_of::<{rust_name}>());')
        c.append(f'printf("layout {name} %zu %zu\\n", sizeof({name}), _Alignof({name}));')
        for field in n.get('inner', []):
            if field.get('kind') != 'FieldDecl': continue
            key = field['name']
            rust.append(f'println!("field {name}.{key} {{}}", std::mem::offset_of!({rust_name}, {key}));')
            rust.append(f'let value = std::mem::MaybeUninit::<{rust_name}>::uninit();')
            rust.append(f'let field = unsafe {{ std::ptr::addr_of!((*value.as_ptr()).{key}) }};')
            rust.append(f'println!("type {name}.{key} {{}}", std::any::type_name_of_val(&field));')
            c.append(f'printf("field {name}.{key} %zu\\n", offsetof({name}, {key}));')
    enums = [n for n in nodes if n.get('kind') == 'EnumDecl' and n.get('name', '').startswith('RTui')]
    for n in enums:
        name = n['name']
        rust_name = next((k for k, v in RENAMES.items() if v == name), name)
        rust.append(f'println!("layout {name} {{}} {{}}", size_of::<{rust_name}>(), align_of::<{rust_name}>());')
        c.append(f'printf("layout {name} %zu %zu\\n", sizeof({name}), _Alignof({name}));')
        prefix = re.sub(r'(?<!^)(?=[A-Z])', '_', name).upper() + '_'
        for variant in n.get('inner', []):
            if variant.get('kind') != 'EnumConstantDecl': continue
            key = variant['name']
            rust_variant = ''.join(part.title() for part in key.removeprefix(prefix).split('_'))
            rust.append(f'println!("enum {key} {{}}", {rust_name}::{rust_variant} as i64);')
            c.append(f'printf("enum {key} %lld\\n", (long long){key});')
    rust.append('}')
    c.append('return 0; }')
    rs = scratch / 'probe.rs'; rs.write_text('\n'.join(rust) + '\n')
    cs = scratch / 'probe.c'; cs.write_text('\n'.join(c) + '\n')
    subprocess.run(['rustc', '--edition=2021', str(rs), '--extern', f'reactive_tui={target}/debug/libreactive_tui.rlib', '-L', f'dependency={target}/debug/deps', '-o', str(scratch/'rust-probe')], check=True, timeout=120)
    subprocess.run(['clang', '-std=c11', '-pedantic-errors', '-Iinclude', str(cs), '-o', str(scratch/'c-probe')], check=True, timeout=60)
    rust_lines = subprocess.check_output([str(scratch/'rust-probe')], text=True, timeout=30).splitlines()
    c_lines = subprocess.check_output([str(scratch/'c-probe')], text=True, timeout=30).splitlines()
    rust_layout = [x for x in rust_lines if not x.startswith(('fn ', 'type '))]
    if rust_layout != c_lines:
        raise RuntimeError('Rust/C layout disagreement: ' + repr((rust_layout, c_lines)))
    signatures = {x.split(' ', 2)[1]: x.split(' ', 2)[2] for x in rust_lines if x.startswith('fn ')}
    for n in functions:
        args = [c_type(x['type']['qualType'], aliases) for x in n.get('inner', []) if x.get('kind') == 'ParmVarDecl']
        ret = n['type']['qualType'].split('(', 1)[0].strip()
        expected = 'callback(' + ','.join(args) + ')->' + ('()' if ret == 'void' else c_type(ret, aliases))
        actual = rust_type(signatures[n['name']])
        if expected != actual:
            raise RuntimeError(f"{n['name']}: C {expected} != Rust {actual}")
    field_types = {x.split(' ', 2)[1]: x.split(' ', 2)[2] for x in rust_lines if x.startswith('type ')}
    for record in records:
        for field in record.get('inner', []):
            if field.get('kind') != 'FieldDecl': continue
            key = record['name'] + '.' + field['name']
            actual = rust_type(field_types[key].removeprefix('*const '))
            expected = c_type(field['type']['qualType'], aliases)
            if actual != expected:
                raise RuntimeError(f'{key}: C {expected} != Rust {actual}')
    print(f'Independent compilers agree: {len(functions)} signatures, {len(records)} record layouts')
    return {'signatures': signatures, 'layouts': c_lines}
