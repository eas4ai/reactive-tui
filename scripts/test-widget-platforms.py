#!/usr/bin/env python3
"""Controlled native-record and launch fixtures; never native acceptance."""
from contextlib import ExitStack
import hashlib
import importlib.util
import inspect
import io
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("widget_platforms", ROOT / "scripts/check-widget-platforms.py")
CHECK = importlib.util.module_from_spec(spec)
spec.loader.exec_module(CHECK)


class NativeWidgetControls(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.probe = str(self.root / "private target/debug/examples/image_host_probe")
        self.test_directory = self.root / "private target/debug/deps"
        self.commands = []
        guards = ExitStack()
        self.addCleanup(guards.close)
        guards.enter_context(patch.object(CHECK, "ROOT", self.root))
        guards.enter_context(patch.object(CHECK, "RECORDS", self.root / "records"))
        guards.enter_context(patch.object(CHECK, "CASES", {"case": ["--lib", "fixture"]}))
        guards.enter_context(patch.object(CHECK, "digest", return_value="controlled-digest"))
        guards.enter_context(patch.object(CHECK.subprocess, "check_output", return_value="controlled-commit"))
        guards.enter_context(patch("sys.stdout", new=io.StringIO()))

    def execute(self, command, output, timeout):
        self.commands.append(command)
        text = "test result: ok. 1 passed; 0 failed;"
        if command[0] == "cargo" and "--no-run" in command:
            text = "\n".join(json.dumps({"reason": "compiler-artifact", "profile": {"test": True},
                "target": {"name": name}, "executable": str(self.test_directory / name)})
                for name in ("reactive_tui", "api_widget_behavior"))
        elif "-c" in command:
            text = "Controlled compiler diagnostic\n" + json.dumps(self.probe)
        elif "scripts/check-dialog-http.py" in command:
            text = "Dialog HTTPS trusted: passed\nDialog HTTPS untrusted: passed"
        output.write_text(text)
        return text

    def records(self, *, zero=False):
        for system in ("Darwin", "Windows"):
            directory = CHECK.RECORDS / system.lower()
            directory.mkdir(parents=True)
            text = "test result: ok. " + ("0" if zero else "1") + " passed; 0 failed;"
            outputs = {"case.out": text, "build.out": "Controlled build", "curl.out": "Controlled curl",
                "https.out": "Dialog HTTPS trusted: passed\nDialog HTTPS untrusted: passed"}
            if system == "Windows":
                outputs["runtime-install.out"] = "Controlled runtime installation"
            else:
                outputs["iterm-build.out"] = "Controlled image build"
                outputs["iterm-host.out"] = "\n".join(
                    f"iTerm2 {mode}: image presence, update, movement and removal passed"
                    for mode in ("app-iterm", "app-auto", "surface-iterm", "iterm"))
                outputs["iterm-host.out"] += "\niTerm2: ASCII violating image case rejected"
                outputs["iterm-host/host.json"] = "Controlled hash fixture, not native capture"
                for mode in ("app-iterm", "app-auto", "surface-iterm", "iterm", "app-ascii"):
                    for name in ("pixels.json", "geometry-pixels.json", "fixture-exit.txt",
                                 "stage-0.png", "stage-1.png", "stage-2.png"):
                        outputs[f"iterm-host/{mode}/{name}"] = "Controlled hash fixture"
            for name, data in outputs.items():
                path = directory / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(data)
            record = {"system": system, "platform": system + "-controlled-fixture", "result": "pass",
                "inputs_digest": "controlled-digest", "commit": "controlled-commit",
                "outputs": {name: hashlib.sha256((directory / name).read_bytes()).hexdigest()
                            for name in outputs}}
            (directory / "record.json").write_text(json.dumps(record))

    def test_managed_record_directory(self):
        self.assertEqual(str(spec.loader.path).replace("scripts/check-widget-platforms.py", ""), str(ROOT) + "/")
        actual = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(actual)
        self.assertEqual(actual.RECORDS, ROOT / ".cairn/reviews/widget-platforms")

    def test_digest_declares_workspace_and_runner_dependencies(self):
        self.assertTrue({"crates", "tests", "scripts/check-widget-platforms.py",
                         ".cairn/api-closure/check.py"}.issubset(CHECK.INPUTS))

    def test_windows_runtime_uses_actual_test_artifact_directory(self):
        with patch.object(CHECK.platform, "system", return_value="Windows"), \
                patch.object(CHECK, "execute", side_effect=self.execute):
            CHECK.run(True)
        install = [command for command in self.commands if "scripts/install-conpty-runtime.py" in command]
        self.assertEqual(len(install), 1)
        self.assertEqual(install[0][3], str(self.test_directory))

    def test_missing_test_artifacts_do_not_install_runtime(self):
        with patch.object(CHECK, "execute", side_effect=self.execute):
            with self.assertRaisesRegex(RuntimeError, "Missing or ambiguous"):
                CHECK.install_runtime("{}\n", self.root)
        self.assertEqual(self.commands, [])

    def test_ambiguous_test_artifact_directories_do_not_install_runtime(self):
        build = "\n".join(json.dumps({"reason": "compiler-artifact", "profile": {"test": True},
            "target": {"name": name}, "executable": str(self.root / name / "test.exe")})
            for name in ("reactive_tui", "api_widget_behavior"))
        with patch.object(CHECK, "execute", side_effect=self.execute):
            with self.assertRaisesRegex(RuntimeError, "Missing or ambiguous"):
                CHECK.install_runtime(build, self.root)
        self.assertEqual(self.commands, [])

    def test_darwin_forwards_exact_selected_probe(self):
        with patch.object(CHECK.platform, "system", return_value="Darwin"), \
                patch.object(CHECK, "execute", side_effect=self.execute):
            CHECK.run(True)
        captures = [command for command in self.commands if "scripts/check-iterm-host.py" in command]
        self.assertEqual(len(captures), 1)
        self.assertIn("--executable", captures[0])
        self.assertEqual(captures[0][captures[0].index("--executable") + 1], self.probe)

    def test_probe_build_uses_outer_process_group_deadline(self):
        with patch.object(CHECK, "execute", side_effect=self.execute) as execute:
            self.assertEqual(CHECK.build_probe(self.root / "build.out"), self.probe)
        self.assertLess(execute.call_args.args[2], 600)
        command = execute.call_args.args[0]
        self.assertIn("-c", command)
        self.assertIn(".cairn/api-closure/check.py", command[-1])

    def test_zero_case_rejected_even_when_hash_matches(self):
        self.records(zero=True)
        with self.assertRaisesRegex(RuntimeError, "execute tests"):
            CHECK.verify()

    def test_valid_controlled_record_fixture(self):
        self.records()
        CHECK.verify()

    def test_damaged_output_rejected(self):
        self.records()
        (CHECK.RECORDS / "darwin/case.out").write_text("Damaged controlled output")
        with self.assertRaisesRegex(RuntimeError, "Damaged"):
            CHECK.verify()

    def test_iterm_capture_requires_explicit_selected_executable(self):
        spec = importlib.util.spec_from_file_location("iterm_host", ROOT / "scripts/check-iterm-host.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertIn("executable", inspect.signature(module.capture).parameters)


class NativeDigestControls(unittest.TestCase):
    @unittest.skipIf(os.name == "nt", "POSIX process-group control; Windows uses taskkill")
    def test_owned_descendant_stops_on_outer_timeout_and_output_survives(self):
        with tempfile.TemporaryDirectory() as scratch:
            log = Path(scratch) / "timeout.out"
            program = ("import subprocess,sys,time; "
                "child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); "
                "print(child.pid,flush=True); time.sleep(30)")
            pid = None
            try:
                with self.assertRaises(subprocess.TimeoutExpired):
                    CHECK.execute([sys.executable, "-c", program], log, .5)
                pid = int(log.read_text().strip())
                status = subprocess.run(["ps", "-p", str(pid), "-o", "stat="],
                    capture_output=True, text=True, check=False)
                self.assertTrue(status.returncode != 0 or status.stdout.strip().startswith("Z"),
                                "Owned descendant survived the outer process-group timeout")
            finally:
                if pid is not None:
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_workspace_and_runner_changes_invalidate_real_git_digest(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            def git(*arguments, check=True):
                return subprocess.run(["git", "-c", "user.name=Controlled fixture",
                    "-c", "user.email=fixture@example.invalid", *arguments], cwd=root,
                    check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            git("init")
            paths = ("Cargo.toml", "crates/member/Cargo.toml", "scripts/check-widget-platforms.py")
            for name in paths:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("Controlled digest fixture\n")
            git("add", ".")
            git("commit", "-m", "Controlled input baseline")
            with patch.object(CHECK, "ROOT", root):
                baseline = CHECK.digest()
                for name in paths[1:]:
                    path = root / name
                    original = path.read_bytes()
                    path.write_text("Controlled changed input\n")
                    with self.assertRaises(subprocess.CalledProcessError):
                        CHECK.digest()
                    git("add", name)
                    git("commit", "-m", "Controlled dependency change")
                    self.assertNotEqual(CHECK.digest(), baseline)
                    path.write_bytes(original)
                    git("add", name)
                    git("commit", "-m", "Controlled dependency restoration")
                    self.assertEqual(CHECK.digest(), baseline)
                (root / "crates/member/untracked.rs").write_text("Controlled new source\n")
                with self.assertRaisesRegex(RuntimeError, "Commit native widget inputs"):
                    CHECK.digest()


if __name__ == "__main__":
    unittest.main()
