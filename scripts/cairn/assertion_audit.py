#!/usr/bin/env python3
"""BAR-002: every #[test] asserts an observable outcome or is named smoke_*,
and a test that skips prints SKIP.

Scans the Rust sources of every workspace member (src/, tests/, crates/)
plus benches/ and examples/. The audit reads code, not prose: comments are
removed and string and char literals are blanked before it looks for
attributes, assertions, calls and returns, so an assert! quoted in a comment
or a string does not count.

A test body counts as asserting when it contains an assert macro (assert!,
assert_eq!, assert_ne!, debug_assert*), panic! or unreachable!, calls a
function or macro defined in the same file whose body asserts (a check
helper), or the test carries #[should_panic]. An attribute belongs to the
item it is attached to, so a neighbour's #[should_panic] exempts nothing. A
bare matches! does not count; .expect() and .unwrap() do not count. A test
attribute with no fn after it is itself reported. Files with
`harness = false` in Cargo.toml are skipped.

A test skips when it returns early (`return;`, `return,`, `return }` or
`return Ok(())`) from a block that has not asserted by then: an `if` that
finds no GPU, an `Err(_)` arm for a missing terminal, a `let ... else`. The
skipping block must print SKIP before it returns, whatever the test is named;
no list of hardware words decides it, because a gate can name its cause in
any words or in none.

`--fixture PATH` audits one file only (used to demonstrate the failing case).
"""

import re
import sys
from pathlib import Path

from _common import ROOT, rust_sources

TEST_ATTR = re.compile(r"#\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test\s*[\](]")
ITEM_FN = re.compile(
    r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]*\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]"
)
FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
MACRO = re.compile(r"\bmacro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)\s*\{")
MARKERS = re.compile(r"\bassert(?:_eq|_ne)?!|\bpanic!|\bunreachable!|\bdebug_assert")
EARLY_RETURN = re.compile(r"\breturn\s*(?:[;,}]|Ok\s*\(\s*\(\s*\)\s*\))")
RAW_STRING = re.compile(r"[bc]?r(#*)\"")
CHAR = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|.)|[^\\'\n])'")
NO_HARNESS = {"dqc_003_captured_diagnostics"}


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def mask(text: str) -> tuple[str, str]:
    """Return (code, prose): `code` has comments removed and literal contents
    blanked; `prose` has comments removed and literals kept. Both keep every
    offset and newline of `text`."""
    code, prose = list(text), list(text)

    def blank(buf: list[str], start: int, end: int) -> None:
        for k in range(start, min(end, len(buf))):
            if buf[k] != "\n":
                buf[k] = " "

    def after_ident(i: int) -> bool:
        return i > 0 and (text[i - 1].isalnum() or text[i - 1] == "_")

    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            blank(code, i, end)
            blank(prose, i, end)
            i = end
            continue
        if text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(code, i, j)
            blank(prose, i, j)
            i = j
            continue
        raw = RAW_STRING.match(text, i)
        if raw and not after_ident(i):
            close = '"' + raw.group(1)
            end = text.find(close, raw.end())
            end = n if end < 0 else end
            blank(code, raw.end(), end)
            i = end + len(close)
            continue
        if text[i] == '"' or (text[i] in "bc" and text.startswith('"', i + 1) and not after_ident(i)):
            j = start = i + (1 if text[i] == '"' else 2)
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(code, start, j)
            i = j + 1
            continue
        if text[i] == "'":
            char = CHAR.match(text, i)
            if char:
                blank(code, i + 1, char.end() - 1)
                i = char.end()
                continue
        i += 1
    return "".join(code), "".join(prose)


def matching(code: str, i: int) -> int:
    """Index just past the bracket that closes the one at code[i]."""
    pairs = {"{": "}", "[": "]", "(": ")"}
    opening, closing = code[i], pairs[code[i]]
    depth, j = 1, i + 1
    while j < len(code) and depth:
        depth += code[j] == opening
        depth -= code[j] == closing
        j += 1
    return j


def body_after(code: str, start: int) -> tuple[int, str]:
    i = code.find("{", start)
    if i < 0:
        return start, ""
    return i, code[i:matching(code, i)]


def helper_bodies(code: str, prose: str) -> dict[str, tuple[str, str]]:
    """The (code, prose) body of each function and macro defined in the file."""
    bodies = {}
    for m in FN.finditer(code):
        at, body = body_after(code, m.end())
        bodies.setdefault(m.group(1), (body, prose[at:at + len(body)]))
    for m in MACRO.finditer(code):
        at, body = body_after(code, m.end() - 1)
        bodies.setdefault(m.group(1), (body, prose[at:at + len(body)]))
    return bodies


def helpers_where(bodies: dict[str, tuple[str, str]], direct) -> set[str]:
    """Functions and macros for which `direct(code, prose)` holds, or that
    call one of them."""
    helpers = {name for name, (body, told) in bodies.items() if direct(body, told)}
    while True:
        more = {name for name, (body, _) in bodies.items() if name not in helpers
                and calls_helper(body, helpers)}
        if not more:
            return helpers
        helpers |= more


def calls_helper(body: str, helpers: set[str]) -> bool:
    """Whether `body` calls one of `helpers`, as a function or a macro. A
    method call or a path call (`x.check(`, `Other::check(`) may name
    another type's function, so only a bare call counts."""
    return any(re.search(rf"(?<![.:\w]){re.escape(h)}\s*!?\s*[(\[{{]", body) for h in helpers)


def enclosing_block(body: str, at: int) -> int:
    """Offset of the `{` of the innermost block of `body` that contains `at`."""
    depth = 0
    for k in range(at - 1, -1, -1):
        if body[k] == "}":
            depth += 1
        elif body[k] == "{":
            if depth == 0:
                return k
            depth -= 1
    return 0


def statement_start(body: str, block: int) -> int:
    """Offset where the statement that owns the `{` at `block` begins: its
    `if`, `match` arm, `let ... else` or `else`, so a gate's condition is read
    with the block it guards."""
    depth = 0
    for k in range(block - 1, -1, -1):
        c = body[k]
        if depth == 0 and c in ";}{([":
            return k + 1
        if c in ")]}":
            depth += 1
        elif c in "([{":
            depth -= 1
    return 0


def attribute_runs(code: str):
    """Yield (attributes, end) for each run of outer attributes in `code`."""
    i = 0
    while (start := code.find("#[", i)) >= 0:
        attrs, j = [], start
        while code.startswith("#[", j):
            end = matching(code, j + 1)
            attrs.append(code[j:end])
            j = end
            while j < len(code) and code[j].isspace():
                j += 1
        yield attrs, j
        i = j


def audit(path: Path) -> list[str]:
    if path.stem in NO_HARNESS:
        return []
    code, prose = mask(path.read_text(errors="replace"))
    bodies = helper_bodies(code, prose)
    helpers = helpers_where(bodies, lambda body, _: MARKERS.search(body) is not None)
    skippers = helpers_where(bodies, lambda _, told: "SKIP" in told)
    bad = []
    for attrs, end in attribute_runs(code):
        if not any(TEST_ATTR.match(a) for a in attrs):
            continue
        fn = ITEM_FN.match(code, end)
        if not fn:
            bad.append(f"{rel(path)}: test attribute at offset {end} with no fn after it")
            continue
        name = fn.group(1)
        others = helpers - {name}
        should_panic = any(a.startswith("#[should_panic") for a in attrs)
        open_at, body = body_after(code, fn.end())
        if not (name.startswith("smoke_") or should_panic
                or MARKERS.search(body) or calls_helper(body, others)):
            bad.append(f"{rel(path)}::{name}")
            continue
        told = prose[open_at:open_at + len(body)]
        for ret in EARLY_RETURN.finditer(body):
            start = statement_start(body, enclosing_block(body, ret.start()))
            before = body[start:ret.start()]
            if MARKERS.search(before) or calls_helper(before, others):
                continue
            if "SKIP" in told[start:ret.start()] or calls_helper(before, skippers - {name}):
                continue
            line = code.count("\n", 0, open_at + ret.start()) + 1
            bad.append(f"{rel(path)}::{name} (returns at line {line} before asserting, without printing SKIP)")
            break
    return bad


def main() -> int:
    if len(sys.argv) > 2 and sys.argv[1] == "--fixture":
        files = [Path(sys.argv[2]).resolve()]
    else:
        files = rust_sources("tests", "src", "crates", "benches", "examples")
    bad = [b for f in files for b in audit(f)]
    if bad:
        print("BAR-002 violated: tests without an assertion and not named smoke_, or that skip without printing SKIP:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-002 holds: {len(files)} files audited")
    return 0


if __name__ == "__main__":
    sys.exit(main())
