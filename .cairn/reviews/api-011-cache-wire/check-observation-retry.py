"""Exercise the actual Orca observation retry with a controlled clock."""
import ast
from pathlib import Path
from types import SimpleNamespace

source = ast.parse(Path("tests/api_widget_behavior/orca.py").read_text())
inside = next(node for node in source.body if isinstance(node, ast.FunctionDef) and node.name == "inside")
functions = [node for node in inside.body if isinstance(node, ast.FunctionDef) and node.name in ("wait", "application_disappeared")]
assert len(functions) == 2
class RemoteError(Exception):
    def __init__(self, message="The application no longer exists", domain="atspi_error", code=0):
        super().__init__(message)
        self.message, self.domain, self.code = message, domain, code
class Clock:
    now = 0.0
    def monotonic(self): return self.now
    def sleep(self, delay): self.now += delay
clock = Clock()
context = SimpleNamespace(pending=lambda: False)
namespace = {"time": clock, "GLib": SimpleNamespace(Error=RemoteError, MainContext=SimpleNamespace(default=lambda: context))}
exec(compile(ast.Module(body=functions, type_ignores=[]), "orca-observation-retry", "exec"), namespace)
wait = namespace["wait"]
calls = 0
def transient():
    global calls
    calls += 1
    if calls < 3: raise RemoteError()
    return "observed replacement"
assert wait("replacement absent", transient, timeout=0.1) == "observed replacement"
assert calls == 3
print("PASS removed-node observation retries until the required replacement appears")
start = clock.now
def permanent(): raise RemoteError()
try:
    wait("replacement never appeared", permanent, timeout=0.1)
except AssertionError as error:
    assert str(error) == "replacement never appeared"
else: raise AssertionError("permanent disappearance was accepted")
assert 0.1 <= clock.now - start < 0.14
print("PASS permanent disappearance still fails within the unchanged deadline")
for error in [RemoteError("different failure"), RemoteError(domain="different-domain"), RemoteError(code=1)]:
    def unexpected(): raise error
    start = clock.now
    try: wait("unexpected failure", unexpected, timeout=0.1)
    except RemoteError as observed: assert observed is error
    else: raise AssertionError("unrelated error was swallowed")
    assert clock.now == start
print("PASS unrelated remote errors propagate immediately")
