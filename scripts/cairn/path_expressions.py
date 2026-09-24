"""The repository paths a Python script builds, read without running it (BAR-007).

The file is parsed and each path expression whose base is known is worked
out to a repository path. The bases are the root (ROOT, REPO, root and names
like them, or `Path(__file__).resolve().parents[N]` from the file's place),
the file's own directory (`Path(__file__).parent`, os.path.dirname), and the
working directory, which is the root where these scripts run (`Path("x")`,
`open("x")`, `Path.cwd()`). The chain after the base is followed: `/`,
joinpath, os.path.join, parent, parents[N], with_name, with_suffix, str()
and f-strings that start with a known path. A name holds the value last
assigned to it in its scope, and `self.x` the value any method assigned it.

What is reported, each with its line and source text:

- the whole of each path chain that includes a string segment, and
  Path("dir/name"); Path("name") alone is a value a test may make up;
- a file open() reads (a file opened to write may not exist yet);
- in a command list, the script its interpreter runs (after any `env`
  prefix), the file of each --config-style option and each cargo
  --example, --test, --bench or --bin target;
- the text of every string, for the shell commands it may hold.

A relative path inside a call given `cwd=` is read from that directory, or
not at all when the directory is unknown, as is a command run with
`env --chdir`. A path is not reported when the code only asks whether it
exists (`.exists()`, `.is_file()`, `.is_dir()`, os.path.exists and the
like, on the chain or on the name that holds it), when it leaves the
repository, or when a segment is a pattern or an unknown value. A name the
code binds to an unknown value (a parameter, a loop or `with` variable)
does not fall back to meaning the root, so `root` in a test that makes a
temporary tree is not the repository.
"""

from __future__ import annotations

import ast
import posixpath
import re
from dataclasses import dataclass

ROOT_NAMES = {"ROOT", "REPO", "REPO_ROOT", "REPOSITORY", "ROOT_DIR", "PROJECT_ROOT", "repo_root", "root", "repo"}
PATH_TYPES = {"Path", "PurePath", "PosixPath", "PurePosixPath", "WindowsPath", "PureWindowsPath"}
INTERPRETERS = {"python", "python3", "bash", "sh", "zsh", "node", "pwsh", "ruby", "perl"}
OPTION_FILES = ("--config", "--manifest-path", "--file", "--config-file")
CARGO_TARGET_OPTIONS = {"--example": "example", "--test": "test", "--bench": "bench", "--bin": "bin"}
PROBES = {"exists", "is_file", "is_dir", "is_symlink", "lexists", "isfile", "isdir", "islink"}
WRITERS = {"write_text", "write_bytes", "mkdir", "touch"}
PATTERN = set("*?[]{}<>$")
TRY = (ast.Try, ast.TryStar) if hasattr(ast, "TryStar") else (ast.Try,)
# A file an interpreter or an option is given: a name with an extension.
FILE_NAME = re.compile(r"(?:\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.[A-Za-z0-9]+")


@dataclass(frozen=True)
class Place:
    """A repository path: `path` is relative to the root ("." is the root);
    `base` is "root", "file" or "cwd", how the expression reached it."""
    path: str
    base: str
    literal: bool = False


@dataclass(frozen=True)
class Text:
    value: str


@dataclass(frozen=True)
class Interpreter:
    pass


# A name the code bound to something this reader cannot know: a parameter, a
# loop or `with` variable, a value it could not work out. Unlike a name it
# never bound (an import), it does not fall back to meaning the root.
UNKNOWN = object()


@dataclass(frozen=True)
class Reference:
    line: int
    source: str
    kind: str  # "path", "target" (a cargo target name) or "command" (text to read for shell commands)
    place: Place | None = None
    name: str = ""


def dotted(node: ast.AST) -> str:
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return ""


def join(place: Place, segment: str) -> Place | None:
    """`place` / `segment`, or None when the segment is absolute, a pattern
    or leaves the repository."""
    if segment.startswith("/") or PATTERN & set(segment) or "\\" in segment:
        return None
    if not segment:
        return place
    path = posixpath.normpath(posixpath.join(place.path, segment))
    if path == ".." or path.startswith("../"):
        return None
    return Place(path, place.base, True)


def parent(place: Place, levels: int = 1) -> Place | None:
    path = place.path
    for _ in range(levels):
        if path in ("", "."):
            return None
        path = posixpath.dirname(path) or "."
    return Place(path, place.base, place.literal)


def relative_text(value: str, literal: bool = True) -> Place | None:
    """A string as a path from the working directory, if it can be one."""
    if not value or value.startswith(("/", "~")) or PATTERN & set(value) or "\\" in value or ":" in value:
        return None
    place = join(Place(".", "cwd"), value)
    return Place(place.path, place.base, literal) if place else None


class Reader:
    def __init__(self, name: str, source: str):
        self.name = name
        self.source = source
        self.found: list[Reference] = []
        self.pending: list[tuple[str, Reference]] = []  # (name, reference) for an assigned chain
        self.probed: set[str] = set()
        self.cwd: list = []  # the working directory of the calls being read

    # -- values ---------------------------------------------------------------

    def value(self, node: ast.AST, env: dict):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return Text(node.value)
        if isinstance(node, ast.Name):
            if node.id == "__file__":
                return Place(self.name, "file")
            if node.id in env:
                return None if env[node.id] is UNKNOWN else env[node.id]
            return Place(".", "root") if node.id in ROOT_NAMES else None
        if isinstance(node, ast.Attribute):
            if dotted(node) == "sys.executable":
                return Interpreter()
            if isinstance(node.value, ast.Name) and node.value.id == "self":
                value = env.get("self." + node.attr)
                return None if value is UNKNOWN else value
            if node.attr == "parent":
                base = self.value(node.value, env)
                return parent(base) if isinstance(base, Place) else None
            return None
        if isinstance(node, ast.Subscript):
            if (isinstance(node.value, ast.Attribute) and node.value.attr == "parents"
                    and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, int)):
                base = self.value(node.value.value, env)
                return parent(base, node.slice.value + 1) if isinstance(base, Place) else None
            return None
        if isinstance(node, ast.BinOp):
            left, right = self.value(node.left, env), self.value(node.right, env)
            if isinstance(node.op, ast.Div) and isinstance(left, Place) and isinstance(right, Text):
                return join(left, right.value)
            if isinstance(node.op, ast.Add):
                if isinstance(left, Text) and isinstance(right, Text):
                    return Text(left.value + right.value)
                if isinstance(left, Place) and isinstance(right, Text) and right.value.startswith("/"):
                    return join(left, right.value.lstrip("/"))
            return None
        if isinstance(node, ast.JoinedStr):
            return self.formatted(node, env)
        if isinstance(node, ast.Call):
            return self.call(node, env)
        return None

    def formatted(self, node: ast.JoinedStr, env: dict):
        parts = node.values
        constants = [p.value for p in parts if isinstance(p, ast.Constant) and isinstance(p.value, str)]
        if len(constants) == len(parts):
            return Text("".join(constants))
        first = parts[0] if parts else None
        if not isinstance(first, ast.FormattedValue):
            return None
        base = self.value(first.value, env)
        second = parts[1] if len(parts) > 1 else None
        if not isinstance(base, Place) or not isinstance(second, ast.Constant) or not isinstance(second.value, str):
            return None
        text = second.value
        if not text.startswith("/"):
            return None
        if len(parts) > 2:
            # A later placeholder: only the whole segments before it are known.
            text = text[: text.rfind("/") + 1]
        return join(base, text.strip("/")) if text.strip("/") else base

    def call(self, node: ast.Call, env: dict):
        name = dotted(node.func)
        args = [self.value(a, env) for a in node.args]
        short = name.rsplit(".", 1)[-1]
        if short in PATH_TYPES and name in (short, "pathlib." + short):
            if not args:
                return Place(".", "cwd")
            first = args[0]
            # Path("name") alone is a value a test may make up; with a
            # directory, or joined to more, it names a repository path.
            place = (first if isinstance(first, Place) else
                     relative_text(first.value, "/" in first.value) if isinstance(first, Text) else None)
            return self.extend(place, args[1:])
        if name in ("Path.cwd", "pathlib.Path.cwd", "os.getcwd"):
            return Place(".", "cwd")
        if name == "os.path.join" and args:
            first = args[0]
            place = first if isinstance(first, Place) else relative_text(first.value) if isinstance(first, Text) else None
            return self.extend(place, args[1:])
        if name == "os.path.dirname" and args and isinstance(args[0], Place):
            return parent(args[0])
        if name in ("os.path.abspath", "os.path.realpath", "os.path.normpath", "str", "os.fspath") and args:
            return args[0] if isinstance(args[0], (Place, Text)) else None
        if isinstance(node.func, ast.Attribute):
            receiver = self.value(node.func.value, env)
            if not isinstance(receiver, Place):
                return None
            attr = node.func.attr
            if attr in ("resolve", "absolute", "expanduser"):
                return receiver
            if attr == "joinpath":
                return self.extend(receiver, args)
            if attr == "with_name" and args and isinstance(args[0], Text):
                folder = parent(receiver)
                return join(folder, args[0].value) if folder else None
            if attr == "with_suffix" and args and isinstance(args[0], Text):
                stem, _ = posixpath.splitext(receiver.path)
                return Place(stem + args[0].value, receiver.base, True)
        return None

    @staticmethod
    def extend(place, rest):
        for segment in rest:
            if not isinstance(place, Place) or not isinstance(segment, Text):
                return None
            place = join(place, segment.value)
        return place

    # -- statements -------------------------------------------------------------

    def module(self, tree: ast.Module) -> None:
        self.block(tree.body, {}, docstring=True)
        for name, ref in self.pending:
            if name not in self.probed:
                self.found.append(ref)

    def block(self, statements: list[ast.stmt], env: dict, docstring: bool = False) -> None:
        functions = []
        for index, s in enumerate(statements):
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)):
                functions.append(s)
                env[s.name] = UNKNOWN
            elif isinstance(s, ast.ClassDef):
                self.klass(s, env)
                env[s.name] = UNKNOWN
            elif isinstance(s, (ast.Assign, ast.AnnAssign)):
                targets = s.targets if isinstance(s, ast.Assign) else [s.target]
                if s.value is None:
                    continue
                named = [dotted(t) for t in targets if dotted(t)]
                self.scan(s.value, env, holder=named[0] if len(named) == 1 else "")
                value = self.value(s.value, env)
                for t in targets:
                    self.bind(t, value, env)
            elif isinstance(s, ast.AugAssign):
                self.scan(s.value, env)
                self.bind(s.target, None, env)
            elif isinstance(s, (ast.For, ast.AsyncFor)):
                self.scan(s.iter, env)
                self.bind(s.target, None, env)
                self.block(s.body, env)
                self.block(s.orelse, env)
            elif isinstance(s, ast.While):
                self.scan(s.test, env)
                self.block(s.body, env)
                self.block(s.orelse, env)
            elif isinstance(s, ast.If):
                self.scan(s.test, env)
                self.block(s.body, env)
                self.block(s.orelse, env)
            elif isinstance(s, (ast.With, ast.AsyncWith)):
                for item in s.items:
                    self.scan(item.context_expr, env)
                    if item.optional_vars is not None:
                        self.bind(item.optional_vars, None, env)
                self.block(s.body, env)
            elif isinstance(s, TRY):
                self.block(s.body, env)
                for handler in s.handlers:
                    if handler.type is not None:
                        self.scan(handler.type, env)
                    if handler.name:
                        env[handler.name] = UNKNOWN
                    self.block(handler.body, env)
                self.block(s.orelse, env)
                self.block(s.finalbody, env)
            elif isinstance(s, ast.Match):
                self.scan(s.subject, env)
                for case in s.cases:
                    self.block(case.body, env)
            elif isinstance(s, ast.Expr):
                if docstring and index == 0 and isinstance(s.value, ast.Constant) and isinstance(s.value.value, str):
                    continue
                self.scan(s.value, env)
            else:
                for child in ast.iter_child_nodes(s):
                    if isinstance(child, ast.expr):
                        self.scan(child, env)
        for f in functions:
            self.function(f, dict(env))

    def function(self, node, env: dict) -> None:
        for decorator in node.decorator_list:
            self.scan(decorator, env)
        for default in [*node.args.defaults, *[d for d in node.args.kw_defaults if d is not None]]:
            self.scan(default, env)
        a = node.args
        for arg in [*a.posonlyargs, *a.args, *a.kwonlyargs, *[x for x in (a.vararg, a.kwarg) if x]]:
            env[arg.arg] = UNKNOWN
        self.block(node.body, env, docstring=True)

    def klass(self, node: ast.ClassDef, env: dict) -> None:
        for expr in [*node.decorator_list, *node.bases]:
            self.scan(expr, env)
        body = dict(env)
        methods = [s for s in node.body if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef))]
        self.block([s for s in node.body if s not in methods], body, docstring=True)
        shared: dict = {}
        for method in sorted(methods, key=lambda m: m.name != "__init__"):
            local = {**env, **shared}
            self.function(method, local)
            shared.update({k: v for k, v in local.items() if k.startswith("self.")})

    def bind(self, target: ast.AST, value, env: dict) -> None:
        key = dotted(target)
        if key and (isinstance(target, ast.Name) or key.startswith("self.")):
            env[key] = UNKNOWN if value is None else value
        elif isinstance(target, (ast.Tuple, ast.List)):
            for element in target.elts:
                self.bind(element, None, env)
        elif isinstance(target, ast.Starred):
            self.bind(target.value, None, env)

    # -- expressions ------------------------------------------------------------

    def text(self, node: ast.expr) -> str:
        segment = ast.get_source_segment(self.source, node) or ""
        return " ".join(segment.split())[:160]

    def report(self, node: ast.expr, place: Place, holder: str = "") -> None:
        if place.base == "cwd" and self.cwd:
            # Inside a call run with cwd=: a relative path is read from there.
            where = self.cwd[-1]
            if where is UNKNOWN:
                return
            rebased = join(where, place.path)
            if rebased is None:
                return
            place = Place(rebased.path, where.base, True)
        ref = Reference(node.lineno, self.text(node), "path", place)
        if holder:
            self.pending.append((holder, ref))
        else:
            self.found.append(ref)

    def builds(self, node: ast.AST) -> bool:
        """Whether `node` builds a path from a base, so its operands are part of it."""
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Div, ast.Add)):
            return True
        if isinstance(node, ast.JoinedStr):
            return True
        if isinstance(node, ast.Call):
            name = dotted(node.func)
            short = name.rsplit(".", 1)[-1]
            if short in PATH_TYPES or name in ("os.path.join", "os.path.dirname", "os.path.abspath",
                                               "os.path.realpath", "os.path.normpath", "str", "os.fspath"):
                return True
            return isinstance(node.func, ast.Attribute) and node.func.attr in (
                "joinpath", "with_name", "with_suffix", "resolve", "absolute", "expanduser")
        if isinstance(node, ast.Attribute):
            return node.attr == "parent"
        return isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute) and node.value.attr == "parents"

    def scan(self, node: ast.AST, env: dict, holder: str = "", inner: bool = False, probe: bool = False) -> None:
        if isinstance(node, (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp)):
            local = dict(env)
            for comp in node.generators:
                self.scan(comp.iter, local)
                self.bind(comp.target, None, local)
                for condition in comp.ifs:
                    self.scan(condition, local)
            for part in ([node.key, node.value] if isinstance(node, ast.DictComp) else [node.elt]):
                self.scan(part, local)
            return
        if isinstance(node, ast.Lambda):
            local = dict(env)
            for arg in node.args.args:
                local[arg.arg] = UNKNOWN
            self.scan(node.body, local)
            return
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            self.found.append(Reference(node.lineno, self.text(node), "command", name=node.value))
            return
        if isinstance(node, ast.Call):
            cwd = next((k.value for k in node.keywords if k.arg == "cwd"), None)
            if cwd is not None:
                self.scan(cwd, env)
                where = self.value(cwd, env)
                self.cwd.append(where if isinstance(where, Place) else UNKNOWN)
                try:
                    self.scan(ast.Call(func=node.func, args=node.args,
                                       keywords=[k for k in node.keywords if k.arg != "cwd"]), env, holder, inner, probe)
                finally:
                    self.cwd.pop()
                return
            self.opened(node, env)
        if isinstance(node, (ast.List, ast.Tuple)):
            self.command(node, env)
        building = self.builds(node)
        value = self.value(node, env) if building else None
        if isinstance(value, Place) and not inner and not probe and value.literal:
            self.report(node, value, holder)
        # The operands of a chain whose value is known are part of it and
        # are not reported again; an unknown chain's operands are read alone.
        quiet = building and isinstance(value, Place)
        for child in ast.iter_child_nodes(node):
            if not isinstance(child, ast.expr):
                continue
            if (isinstance(node, ast.Call) and child is node.func and isinstance(node.func, ast.Attribute)
                    and node.func.attr in PROBES):
                self.probe(node.func.value)
                self.scan(node.func.value, env, probe=True)
                continue
            if isinstance(node, ast.Call) and child in node.args and self.probing(node):
                self.probe(child)
                self.scan(child, env, probe=True)
                continue
            self.scan(child, env, inner=quiet, probe=probe and quiet)

    def probing(self, node: ast.Call) -> bool:
        name = dotted(node.func)
        return name.startswith("os.path.") and name.rsplit(".", 1)[-1] in PROBES or name == "os.access"

    def probe(self, node: ast.AST) -> None:
        if dotted(node):
            self.probed.add(dotted(node))

    def opened(self, node: ast.Call, env: dict) -> None:
        """open("name") reads a file from the working directory."""
        if dotted(node.func) not in ("open", "io.open", "builtins.open") or not node.args:
            return
        arg = node.args[0]
        value = self.value(arg, env)
        mode = node.args[1] if len(node.args) > 1 else next((k.value for k in node.keywords if k.arg == "mode"), None)
        mode_text = mode.value if isinstance(mode, ast.Constant) and isinstance(mode.value, str) else "r"
        # A file opened to write may not exist yet.
        if isinstance(value, Text) and not set("wax") & set(mode_text):
            place = relative_text(value.value)
            if place:
                self.report(arg, place)

    def command(self, node: ast.List | ast.Tuple, env: dict) -> None:
        """A command given as a list: its interpreter's script, the file of
        each --config-style option and each cargo target it names."""
        elements = node.elts
        values = [self.value(e, env) for e in elements]
        # "" stands for an element that is not a string.
        texts = [v.value if isinstance(v, Text) else "" for v in values]
        program, elsewhere = 0, False
        # `env NAME=value ... program`: the program comes after; --chdir
        # runs it in a directory this reader does not follow.
        if texts and posixpath.basename(texts[0]) == "env":
            program = 1
            while program < len(texts) and (texts[program].startswith("-") or "=" in texts[program]):
                elsewhere |= texts[program].startswith(("--chdir", "-C"))
                program += 1
        if not elsewhere and program < len(values) and (
                isinstance(values[program], Interpreter) or posixpath.basename(texts[program]) in INTERPRETERS):
            j = program + 1
            while j < len(texts) and texts[j].startswith("-"):
                if texts[j] in ("-c", "-m", "-e", "-lc", "-ic"):
                    j = len(texts)
                    break
                j += 2 if texts[j] in ("-X", "-W") else 1
            if j < len(texts) and FILE_NAME.fullmatch(texts[j]):
                self.path_text(elements[j], texts[j])
        for i, text in enumerate(texts):
            following = texts[i + 1] if i + 1 < len(texts) else ""
            for option in OPTION_FILES:
                if elsewhere:
                    break
                if text == option and FILE_NAME.fullmatch(following):
                    self.path_text(elements[i + 1], following)
                elif text.startswith(option + "=") and FILE_NAME.fullmatch(text.split("=", 1)[1]):
                    self.path_text(elements[i], text.split("=", 1)[1])
        # A cargo run in an unknown directory builds some other package.
        if "cargo" not in [posixpath.basename(t) for t in texts] or (self.cwd and self.cwd[-1] is UNKNOWN):
            return
        for i, text in enumerate(texts):
            following = texts[i + 1] if i + 1 < len(texts) else ""
            option, _, target = text.partition("=")
            if option not in CARGO_TARGET_OPTIONS:
                continue
            if not target and following and not following.startswith("-"):
                target = following
            if target and not PATTERN & set(target):
                self.found.append(Reference(elements[i].lineno, self.text(node), "target",
                                            name=f"{CARGO_TARGET_OPTIONS[option]} {target}"))

    def path_text(self, node: ast.expr, text: str) -> None:
        place = relative_text(text)
        if place and place.path != ".":
            self.report(node, place)


def references(name: str, source: str) -> list[Reference] | None:
    """The references a Python file makes, or None when it does not parse."""
    try:
        tree = ast.parse(source)
    except (SyntaxError, ValueError):
        return None
    reader = Reader(name, source)
    reader.module(tree)
    return reader.found
