#!/usr/bin/env python3
"""The workspace-gates mechanism's own tests: which failures are the
toolchain's (a signal, an internal compiler error) and so leave no verdict,
and which are the code's and fail BAR-001."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import workspace_gates as gates  # noqa: E402


ICE = """\
   Compiling reactive-tui v1.0.0 (/work/reactive-tui)
error: internal compiler error: /rustc-dev/59807616e1fa2540724bfbac14d7976d7e4a3860/compiler/rustc_codegen_ssa/src/mir/operand.rs:163:21: from_const: invalid ByVal layout: TyAndLayout {
thread 'rustc' (4110861) panicked at /rustc-dev/59807616e1fa2540724bfbac14d7976d7e4a3860/compiler/rustc_codegen_ssa/src/mir/operand.rs:163:21:
Box<dyn Any>
error: could not compile `reactive-tui` (test "api_mouse_hook_routing")

Caused by:
  process didn't exit successfully: `/home/user/.rustup/toolchains/1.95-x86_64-unknown-linux-gnu/bin/rustc --crate-name api_mouse_hook_routing --edition=2021 tests/api_mouse_hook_routing.rs --test` (exit status: 101)
warning: build failed, waiting for other jobs to finish...
"""

SEGFAULT = """\
   Compiling reactive-tui v1.0.0 (/work/reactive-tui)
error: rustc interrupted by SIGSEGV, printing backtrace

error: could not compile `reactive-tui` (lib)
"""

COMPILE_ERROR = """\
   Compiling reactive-tui v1.0.0 (/work/reactive-tui)
error[E0425]: cannot find value `nothing` in this scope
 --> src/lib.rs:3:5
error: could not compile `reactive-tui` (lib) due to 1 previous error
"""

FAILED_TEST = """\
     Running tests/api_mouse_hook_routing.rs (target/debug/deps/api_mouse_hook_routing-1)

running 1 tests
test routes ... FAILED

failures:

---- routes stdout ----
thread 'routes' panicked at tests/api_mouse_hook_routing.rs:9:5:
assertion failed

failures:
    routes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test api_mouse_hook_routing`
"""


class ToolchainCrashes(unittest.TestCase):
    def test_an_internal_compiler_error_is_the_toolchains(self):
        crashes = gates.toolchain_crashes(ICE)
        self.assertEqual(len(crashes), 1, crashes)
        self.assertIn("rustc internal compiler error: /rustc-dev/", crashes[0])

    def test_a_signal_is_the_toolchains(self):
        self.assertEqual(gates.toolchain_crashes(SEGFAULT), ["rustc killed by SIGSEGV"])

    def test_a_compile_error_is_the_codes(self):
        self.assertEqual(gates.toolchain_crashes(COMPILE_ERROR), [])

    def test_a_failed_test_is_the_codes(self):
        self.assertEqual(gates.toolchain_crashes(FAILED_TEST), [])

    def test_a_compile_error_beside_an_ice_is_still_the_codes(self):
        self.assertEqual(gates.toolchain_crashes(ICE + COMPILE_ERROR), [])

    def test_a_failed_test_beside_an_ice_is_still_the_codes(self):
        self.assertEqual(gates.toolchain_crashes(ICE + FAILED_TEST), [])


if __name__ == "__main__":
    unittest.main()
