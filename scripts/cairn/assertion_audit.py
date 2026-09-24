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

A test attribute is `#[test]`, a path ending in `test` (`#[tokio::test]`),
or `test` among the attributes of a `#[cfg_attr(..)]`, so a test compiled
only on another platform is audited too.

A test skips when it returns early (`return;`, `return,`, `return }` or
`return Ok(())`) from a block that has not asserted by then: an `if` that
finds no GPU, an `Err(_)` arm for a missing terminal, a `let ... else`, or
a macro whose body returns that way (a `require_gpu!()` returns from the
test that calls it). A test also skips when its only assertions sit in some
branches of an `if` chain and it can finish through another branch,
explicit or the missing `else`, with no assertion anywhere else in it; an
`if` inside a loop body is taken in some iterations, not instead of them. The
skipping branch must print SKIP, whatever the test is named: a print macro
(eprintln!, println!, eprint!, print!, write!, writeln!) whose string holds
the word SKIP, or a call of a helper that prints one. A name such as
SKIP_GPU_TESTS is not the word. No list of hardware words decides it,
because a gate can name its cause in any words or in none.

`--fixture PATH` audits one file only (used to demonstrate the failing case).
"""

import re
import sys
from pathlib import Path

from _common import ROOT, enclosing_block, mask, matching, rust_sources, statement_start

TEST_ATTR = re.compile(r"#\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test\s*[\](]")
ITEM_FN = re.compile(
    r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]*\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]"
)
FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
MACRO = re.compile(r"\bmacro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)\s*\{")
MARKERS = re.compile(r"\bassert(?:_eq|_ne)?!|\bpanic!|\bunreachable!|\bdebug_assert")
EARLY_RETURN = re.compile(r"\breturn\s*(?:[;,}]|Ok\s*\(\s*\(\s*\)\s*\))")
NO_HARNESS = {"dqc_003_captured_diagnostics"}
PRINT = re.compile(r"\b(?:e?print(?:ln)?|write(?:ln)?)!\s*\(")
SKIP_WORD = re.compile(r'"[^"\n]*\bSKIP\b')
CFG_ATTR = re.compile(r"#\[\s*cfg_attr\s*\(")
TEST_NAME = re.compile(r"\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test\s*(?:\(.*\))?\s*", re.S)


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def is_test_attribute(attr: str) -> bool:
    """`#[test]`, `#[tokio::test]`, or `#[cfg_attr(pred, .., test, ..)]`."""
    if TEST_ATTR.match(attr):
        return True
    m = CFG_ATTR.match(attr)
    if not m:
        return False
    inner = attr[m.end():matching(attr, m.end() - 1) - 1]
    parts, depth, start = [], 0, 0
    for i, c in enumerate(inner):
        depth += c in "([{"
        depth -= c in ")]}"
        if c == "," and depth == 0:
            parts.append(inner[start:i])
            start = i + 1
    parts.append(inner[start:])
    return any(TEST_NAME.fullmatch(part) for part in parts[1:])


def prints_skip(code: str, prose: str, start: int = 0, end: int | None = None) -> bool:
    """Whether code[start:end] has a print macro whose string holds the word
    SKIP. `code` and `prose` share offsets (see mask)."""
    end = len(code) if end is None else end
    for m in PRINT.finditer(code, start, end):
        close = matching(code, m.end() - 1)
        if SKIP_WORD.search(prose, m.end(), close):
            return True
    return False


def silent_returns(code: str, prose: str, skippers: set[str] = frozenset()) -> list[int]:
    """Offsets of the early returns in `code` whose statement asserts nothing
    and prints no SKIP before them."""
    found = []
    for ret in EARLY_RETURN.finditer(code):
        start = statement_start(code, enclosing_block(code, ret.start()))
        before = code[start:ret.start()]
        if MARKERS.search(before) or prints_skip(code, prose, start, ret.start()) or calls_helper(before, skippers):
            continue
        found.append(ret.start())
    return found


def if_chains(code: str):
    """Yield (start, end, branches, has_else) for every `if` chain in `code`,
    each branch a (start, end) block span."""
    for m in re.finditer(r"\bif\b", code):
        if re.search(r"\belse\s*$", code[:m.start()]):
            continue
        branches, at, has_else = [], m.end(), False
        while True:
            depth, i = 0, at
            while i < len(code) and not (code[i] == "{" and depth == 0):
                depth += code[i] in "(["
                depth -= code[i] in ")]"
                if code[i] == ";" and depth == 0:
                    break
                i += 1
            if i >= len(code) or code[i] != "{":
                break
            close = matching(code, i)
            branches.append((i, close))
            rest = re.match(r"\s*else\s*(if\b)?\s*", code[close:])
            if not rest:
                break
            if rest.group(1):
                at = close + rest.end()
                continue
            j = close + rest.end()
            if j < len(code) and code[j] == "{":
                branches.append((j, matching(code, j)))
                has_else = True
            break
        if branches:
            yield m.start(), branches[-1][1], branches, has_else


def in_loop(code: str, at: int) -> bool:
    """Whether offset `at` of `code` is inside the body of a for, while or
    loop: a branch there is taken in some iterations, not instead of them."""
    block = enclosing_block(code, at)
    while block > 0:
        if re.match(r"\s*(?:'\w+\s*:\s*)?(?:for|while|loop)\b", code[statement_start(code, block):block]):
            return True
        block = enclosing_block(code, block)
    return False


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


def returning_macros(files: list[Path]) -> dict[str, str]:
    """Macros defined in `files` whose body returns early without printing
    SKIP first, by name, with the file that defines each."""
    found = {}
    for path in files:
        code, prose = mask(path.read_text(errors="replace"))
        for m in MACRO.finditer(code):
            at, body = body_after(code, m.end() - 1)
            if silent_returns(body, prose[at:at + len(body)]):
                found.setdefault(m.group(1), rel(path))
    return found


def audit(path: Path, macros: dict[str, str] | None = None) -> list[str]:
    if path.stem in NO_HARNESS:
        return []
    code, prose = mask(path.read_text(errors="replace"))
    bodies = helper_bodies(code, prose)
    helpers = helpers_where(bodies, lambda body, _: MARKERS.search(body) is not None)
    skippers = helpers_where(bodies, lambda body, told: prints_skip(body, told))
    macros = returning_macros([path]) if macros is None else macros
    bad = []
    for attrs, end in attribute_runs(code):
        if not any(is_test_attribute(a) for a in attrs):
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
        quiet = skippers - {name}
        asserts = lambda start, stop: MARKERS.search(body, start, stop) or calls_helper(body[start:stop], others)
        skips = lambda start, stop: prints_skip(body, told, start, stop) or calls_helper(body[start:stop], quiet)
        problem = None
        for ret in EARLY_RETURN.finditer(body):
            start = statement_start(body, enclosing_block(body, ret.start()))
            if not (asserts(start, ret.start()) or skips(start, ret.start())):
                problem = f"returns at line {code.count(chr(10), 0, open_at + ret.start()) + 1} before asserting, without printing SKIP"
                break
        for call in re.finditer(r"(?<![.:\w])([A-Za-z_]\w*)\s*!\s*[(\[{]", body) if problem is None else ():
            if call.group(1) not in macros:
                continue
            start = statement_start(body, enclosing_block(body, call.start()))
            if not (asserts(start, call.start()) or skips(start, call.start())):
                problem = (f"{call.group(1)}! at line {code.count(chr(10), 0, open_at + call.start()) + 1} "
                           f"returns before asserting, without printing SKIP ({macros[call.group(1)]})")
                break
        for chain_start, chain_end, branches, has_else in if_chains(body) if problem is None else ():
            if asserts(0, chain_start) or asserts(chain_end, len(body)) or in_loop(body, chain_start):
                continue
            quiet_branch = [b for b in branches if not asserts(*b) and not skips(*b) and not EARLY_RETURN.search(body, *b)]
            if not any(asserts(*b) for b in branches) or not (quiet_branch or not has_else):
                continue
            where = code.count(chr(10), 0, open_at + chain_start) + 1
            problem = (f"asserts only in some branches of the if at line {where} and can finish through "
                       f"{'a branch' if quiet_branch else 'the missing else'} without printing SKIP")
            break
        if problem:
            bad.append(f"{rel(path)}::{name} ({problem})")
    return bad


def main() -> int:
    if len(sys.argv) > 2 and sys.argv[1] == "--fixture":
        files = [Path(sys.argv[2]).resolve()]
    else:
        files = rust_sources("tests", "src", "crates", "benches", "examples")
    macros = returning_macros(files)
    bad = [b for f in files for b in audit(f, macros)]
    if bad:
        print("BAR-002 violated: tests without an assertion and not named smoke_, or that skip without printing SKIP:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-002 holds: {len(files)} files audited")
    return 0


if __name__ == "__main__":
    sys.exit(main())
