"""Exercise discovery against a remote application disappearing mid-traversal."""
import ast
from pathlib import Path
from types import SimpleNamespace

from gi.repository import GLib


class RemoteNode:
    def __init__(self, name, children=(), failure=None, operation="count"):
        self.name, self.children = name, children
        self.failure, self.operation = failure, operation

    def clear_cache(self):
        pass

    def get_name(self):
        if self.failure and self.operation == "name":
            raise self.failure
        return self.name

    def get_child_count(self):
        if self.failure and self.operation == "count":
            raise self.failure
        return len(self.children)

    def get_child_at_index(self, index):
        return self.children[index]


def discover(source, failure, operation="count"):
    tree = ast.parse(Path(source).read_text())
    inside = next(node for node in tree.body
                  if isinstance(node, ast.FunctionDef) and node.name == "inside")
    names = {"descendants", "application_disappeared", "applications",
             "find_window", "window"}
    selected = [node for node in inside.body
                if isinstance(node, ast.FunctionDef) and node.name in names]
    replacement = RemoteNode("Reactive TUI App accessibility round 2")
    desktop = RemoteNode("desktop", [
        RemoteNode("accessibility_probe", failure=failure, operation=operation),
        RemoteNode("accessibility_probe", [replacement]),
    ])
    namespace = {"GLib": GLib, "Atspi": SimpleNamespace(get_desktop=lambda _: desktop)}
    exec(compile(ast.Module(body=selected, type_ignores=[]), source, "exec"), namespace)
    assert namespace["window"](2) is replacement
    assert namespace["window"](1) is None


baseline = ".cairn/reviews/api-011-orca-before-disappeared-app.py"
corrected = "tests/api_widget_behavior/orca.py"
disappeared = GLib.Error("The application no longer exists", "atspi_error", 0)
for operation in ("name", "count"):
    try:
        discover(baseline, disappeared, operation)
    except GLib.Error as error:
        assert error is disappeared
        print(f"PASS baseline rejects disappearing App during {operation}")
    else:
        raise AssertionError("baseline did not reproduce the remote traversal failure")
    discover(corrected, disappeared, operation)
    print(f"PASS corrected discovery finds replacement after {operation} failure")

for failure in (
    GLib.Error("connection refused", "atspi_error", 0),
    GLib.Error("The application no longer exists", "different-domain", 0),
    GLib.Error("The application no longer exists", "atspi_error", 1),
):
    try:
        discover(corrected, failure)
    except GLib.Error as error:
        assert error is failure
        print(f"PASS unrelated error propagates: {failure.domain}/{failure.code}/{failure.message}")
    else:
        raise AssertionError("unrelated remote error was swallowed")
