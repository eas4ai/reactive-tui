#!/usr/bin/env python3
"""Compile public examples, check rustdoc visibility, and validate the API matrix."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import runpy

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
MATRIX = ROOT / "manual/supported-api.md"
SECTIONS = ("props", "rustdoc", "examples", "inventory")


def fences(text):
    for match in re.finditer(r"^```([^\n]*)\n(.*?)^```[ \t]*$", text, re.M | re.S):
        flags = [flag.strip() for flag in match[1].split(",")]
        yield text.count("\n", 0, match.start()) + 1, flags, match[2]


def rust_comments(path):
    """Keep separate doc-comment blocks separate, with original line numbers."""
    block = []
    start = 0
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        match = re.match(r"\s*//[/!] ?(.*)$", line)
        if match:
            if not block:
                start = line_number
            block.append(match[1])
        elif block:
            yield start, "\n".join(block) + "\n"
            block = []
    if block:
        yield start, "\n".join(block) + "\n"


def matrix_modules(text):
    found = set()
    for line in text.splitlines():
        if not line.startswith("| `"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 5 or any(not cell for cell in cells):
            raise AssertionError("API matrix rows need module, contract, checks, limits and status")
        names = re.findall(r"`([a-z_]+)`", cells[0])
        if not names or found.intersection(names):
            raise AssertionError("Missing or duplicate API matrix module")
        found.update(names)
        if not re.search(r"(?:API|ABI|RND|EMB|WAK|REG|CCH|GPU)-\d{3}", cells[2]):
            raise AssertionError("Every matrix row needs named behavior evidence")
        if cells[4] not in ("Verified", "Verified with limits", "Covered by named checks", "API-019 review pending"):
            raise AssertionError("Matrix status must distinguish acceptance from pending review")
    return found


def require_markdown(index, directory):
    if 'href="markdown/index.html"' not in index:
        raise AssertionError("Public Markdown module is absent from the rustdoc module index")
    for name in ("index.html", "renderer/index.html", "converter/index.html", "ast_walker/index.html"):
        if not (directory / "markdown" / name).is_file():
            raise AssertionError("Missing public Markdown documentation: " + name)


class Check:
    def __init__(self):
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        self.output = ROOT / "target/evidence/api-documentation" / stamp
        self.output.mkdir(parents=True)
        self.scratch = TARGET / "api-documentation" / stamp
        self.scratch.mkdir(parents=True)
        self.execute = runpy.run_path(str(ROOT / "scripts/check-widget-platforms.py"))["execute"]
        self.steps = []

    def run(self, name, command, timeout=900, expected_tests=None):
        log = self.output / (name + ".out")
        item = {"name": name, "command": list(map(str, command)), "log": str(log.relative_to(ROOT))}
        try:
            text = self.execute(item["command"], log, timeout)
            if expected_tests is not None:
                summaries = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
                passed = sum(int(row[0]) for row in summaries)
                if passed < expected_tests or any(int(row[1]) for row in summaries):
                    raise AssertionError(f"Expected at least {expected_tests} passing tests, observed {passed}")
                item["tests_passed"] = passed
        except Exception as error:
            item.update(result="fail", error=str(error))
            print("FAIL", name, error, flush=True)
            text = None
        else:
            item["result"] = "pass"
            print("PASS", name, flush=True)
        if log.exists():
            item["sha256"] = hashlib.sha256(log.read_bytes()).hexdigest()
        self.steps.append(item)
        return text

    def inspect(self, name, function):
        before = len(self.steps)
        try:
            function()
            if any(step["result"] != "pass" for step in self.steps[before:]):
                raise AssertionError("A required nested check failed; see the retained outputs")
        except Exception as error:
            self.steps.append({"name": name, "result": "fail", "error": str(error)})
            print("FAIL", name, error, flush=True)
        else:
            self.steps.append({"name": name, "result": "pass"})
            print("PASS", name, flush=True)

    def library(self, name):
        output = self.run(name, ["cargo", "build", "--locked", "--lib",
                                               "--message-format=json-render-diagnostics"])
        if output is None:
            raise RuntimeError("Could not build the facade used by documentation consumers")
        for line in output.splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get("reason") == "compiler-artifact"
                    and message.get("target", {}).get("name") == "reactive_tui"):
                return next(path for path in message["filenames"] if path.endswith(".rlib"))
        raise RuntimeError("Cargo did not identify the documentation consumer library")

    def props(self):
        self.run("documentation-macro-behavior", ["cargo", "test", "--locked", "--test", "api_documentation_contract"], expected_tests=4)
        self.run("documentation-screen-behavior", ["cargo", "test", "--locked", "--lib", "responsive_"], expected_tests=2)
        self.run("css-unit-behavior", ["cargo", "test", "--locked", "--lib", "layout::css::css_in_rust::tests"], expected_tests=5)
        self.run("css-consumer-behavior", ["cargo", "test", "--locked", "--test", "css_in_rust_simple_test"], expected_tests=14)
        self.run("props-behavior", ["cargo", "test", "--locked", "--test", "api_props_contract",
                                     "--test", "props_derive_test", "--test", "simple_props_test"], expected_tests=17)
        self.run("props-diagnostics", ["cargo", "test", "--locked", "-p", "reactive-tui-macros", "--lib"], expected_tests=2)
        library = self.library("props-consumer-library")
        binary = self.scratch / "props-defaults"
        source = ROOT / "verification/api-documentation/props-defaults.rs"
        command = ["rustc", "--edition=2021", "--crate-name", "api018_defaults", str(source),
                   "--extern", "reactive_tui=" + library, "-L", "dependency=" + str(TARGET / "debug/deps"),
                   "-C", "codegen-units=12", "-C", "link-arg=-Wl,--threads=12", "-o", str(binary)]
        if self.run("props-defaults-compile", command) is not None:
            self.run("props-defaults-run", [str(binary)], 10)

    def rustdoc(self):
        for name, toolchain, flags in (
            ("default", [], []), ("minimal", [], ["--no-default-features"]),
            ("all-features", ["+nightly"], ["--all-features"]),
        ):
            result = self.run("rustdoc-" + name, ["cargo", *toolchain, "doc", "--locked", "--lib",
                                                 "--no-deps", *flags])
            if result is not None:
                directory = TARGET / "doc/reactive_tui"
                self.inspect("markdown-visible-" + name,
                             lambda: require_markdown((directory / "index.html").read_text(), directory))
                for page in (directory / "index.html", directory / "markdown/index.html"):
                    if page.is_file():
                        destination = self.output / (name + "-" + page.parent.name + "-index.html")
                        destination.write_bytes(page.read_bytes())
        for name, toolchain, flags in (("default", [], []), ("minimal", [], ["--no-default-features"]),
                                       ("all-features", ["+nightly"], ["--all-features"])):
            self.run("crate-doctests-" + name, ["cargo", *toolchain, "test", "--locked", "--doc", *flags,
                                                  "--", "--test-threads=8"], expected_tests=1)
        self.run("cargo-examples", ["cargo", "check", "--locked", "--examples"])

    def rust_examples(self, library, sources):
        for index, (name, blocks) in enumerate(sources):
            path = self.output / f"rust-{index}.md"
            chunks = []
            for line, flags, code in blocks:
                # Ignored source examples still compile in this facade-only consumer.
                # Host/clipboard examples are not executed on the developer's desktop.
                flags = ["no_run" if flag == "ignore" else flag for flag in flags]
                chunks.append(f"Example from {name}:{line}\n\n```{','.join(flags)}\n{code}```\n")
            path.write_text("\n".join(chunks))
            self.run(f"rust-examples-{index}", ["rustdoc", "--test", str(path), "--edition=2021",
                     "--extern", "reactive_tui=" + library, "-L", "dependency=" + str(TARGET / "debug/deps"),
                       "--test-args=--test-threads=8"], 300, expected_tests=len(blocks))

    def collect_examples(self):
        guides = [ROOT / "README.md", *sorted((ROOT / "docs").glob("*.md")),
                  *sorted((ROOT / "manual").rglob("*.md")),
                  ROOT / "include/README.md", *sorted((ROOT / "bindings/typescript").glob("*.md"))]
        rust, c, typescript, python = [], [], [], []
        inventory = []
        for path in guides:
            blocks = list(fences(path.read_text()))
            selected = [block for block in blocks if block[1][0] == "rust"]
            if selected:
                rust.append((str(path.relative_to(ROOT)), selected))
            for line, flags, code in blocks:
                if flags[0] in ("c", "typescript", "ts"):
                    (c if flags[0] == "c" else typescript).append((str(path.relative_to(ROOT)), line, code))
                if flags[0] == "python":
                    python.append((str(path.relative_to(ROOT)), line, code))
                if flags[0] in ("rust", "c", "typescript", "ts", "python"):
                    inventory.append({"file": str(path.relative_to(ROOT)), "line": line,
                                      "language": flags[0], "sha256": hashlib.sha256(code.encode()).hexdigest()})
        paths = sorted((ROOT / "src").rglob("*.rs")) + [ROOT / "crates/reactive-tui-macros/src/lib.rs"]
        for path in paths:
            # Dependency crates are not modules of the public Reactive-TUI facade.
            # SuprTUI and Crossterm behavior have their own renderer/input checks.
            if any(path.is_relative_to(ROOT / directory) for directory in
                   ("crates/reactive-tui-suprtui", "crates/reactive-tui-crossterm")):
                continue
            selected = []
            for start, comment in rust_comments(path):
                for line, flags, code in fences(comment):
                    if "ignore" in flags or "no_run" in flags or path.parent.parent.name == "reactive-tui-macros":
                        selected.append((start + line - 1, flags, code))
            if selected:
                rust.append((str(path.relative_to(ROOT)), selected))
                inventory.extend({"file": str(path.relative_to(ROOT)), "line": line,
                                  "language": "rust", "sha256": hashlib.sha256(code.encode()).hexdigest()}
                                 for line, _, code in selected)
        (self.output / "examples.json").write_text(json.dumps(inventory, indent=2) + "\n")
        if not rust or not c or not typescript:
            raise AssertionError("Missing Rust, C or TypeScript public examples")
        return rust, c, typescript, python

    def c_examples(self, examples):
        self.run("ffi-library", ["cargo", "build", "--locked", "--features", "ffi"])
        for index, (source, line, code) in enumerate(examples):
            path = self.output / f"c-{index}.c"
            if not re.search(r"\bmain\s*\(", code) and not code.lstrip().startswith("#include"):
                code = "#include <reactive_tui.h>\nvoid documentation_example(void) {\n" + code + "\n}\n"
            path.write_text(code)
            command = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I" + str(ROOT / "include"), str(path)]
            if re.search(r"\bmain\s*\(", code):
                command.extend(["-L" + str(TARGET / "debug"), "-lreactive_tui", "-o", str(self.scratch / f"c-{index}")])
            else:
                command.append("-fsyntax-only")
            self.run(f"c-example-{index}", command, 30)
    def typescript_examples(self, examples):
        ts_files = []
        for index, (source, line, code) in enumerate(examples):
            path = self.output / f"typescript-{index}.ts"
            path.write_text(code + "\nexport {};\n")
            ts_files.append(str(path))
        package = ROOT / "bindings/typescript"
        self.run("typescript-dependencies", ["npm", "--prefix", str(package), "ci", "--ignore-scripts"])
        config = {"extends": str(package / "tsconfig.json"), "compilerOptions": {
            "noEmit": True, "rootDir": str(ROOT), "baseUrl": str(ROOT),
            "typeRoots": [str(package / "node_modules/@types")],
            "paths": {"@reactive-tui/core": [str(package / "src" / "index.ts")]},
        }, "files": ts_files, "include": [], "exclude": []}
        path = self.scratch / "tsconfig.json"
        path.write_text(json.dumps(config, indent=2) + "\n")
        self.run("typescript-examples", ["node", str(package / "node_modules/typescript/bin/tsc"),
                                         "--project", str(path)], 120)

    def examples(self):
        rust, c, typescript, python = self.collect_examples()
        self.rust_examples(self.library("examples-consumer-library"), rust)
        self.c_examples(c)
        self.typescript_examples(typescript)
        for index, (source, line, code) in enumerate(python):
            path = self.output / f"python-{index}.py"
            path.write_text(code)
            self.run(f"python-example-{index}", ["env", "RTUI_LIBRARY_PATH=" + str(TARGET / "debug/libreactive_tui.so"),
                                               "python3", "-B", str(path)], 30)

    def inventory(self):
        text = MATRIX.read_text()
        expected = set(re.findall(r"^pub mod ([a-z_]+)", (ROOT / "src/lib.rs").read_text(), re.M)) | {"macros"}
        actual = matrix_modules(text)
        if actual != expected:
            raise AssertionError(f"API matrix differs from public modules: missing={expected-actual}, extra={actual-expected}")
        for link in re.findall(r"\]\(([^)#]+)(?:#[^)]*)?\)", text):
            if "://" in link:
                continue
            path = (MATRIX.parent / link).resolve()
            if not path.is_relative_to(ROOT):
                raise AssertionError("Matrix link escapes the repository: " + link)
            if not path.exists():
                raise AssertionError("Broken API matrix evidence link: " + link)
        for claim in ("Orca", "GNOME Terminal", "iTerm2 3.7", "Kitty", "patch", "validate", "Option<T>"):
            if claim not in text:
                raise AssertionError("Matrix omits an approved limit: " + claim)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only", choices=SECTIONS)
    args = parser.parse_args()
    os.chdir(ROOT)
    os.environ.update(CARGO_INCREMENTAL="0", RUST_TEST_THREADS="8", RAYON_NUM_THREADS="8",
                      CARGO_BUILD_JOBS="8", LP_NUM_THREADS="8", PYTHON_CPU_COUNT="8", GOMAXPROCS="8", GOFLAGS="-p=8")
    os.environ["RUSTDOCFLAGS"] = os.environ.get("RUSTDOCFLAGS", "") + " -D rustdoc::broken_intra_doc_links"
    check = Check()
    check.run("checker-controls", ["python3", "-B", str(ROOT / "verification/api-documentation/checker-controls.py")], 30)
    for section in ((args.only,) if args.only else SECTIONS):
        check.inspect(section, getattr(check, section))
    (check.output / "results.json").write_text(json.dumps(check.steps, indent=2) + "\n")
    failed = [step["name"] for step in check.steps if step["result"] != "pass"]
    print("Documentation outputs:", check.output.relative_to(ROOT), flush=True)
    if failed:
        raise SystemExit("API-018 failed: " + ", ".join(failed))
    if args.only:
        print(f"PASS development section: {args.only}; full API-018 acceptance was not run", flush=True)
    else:
        print("PASS API-018: Props, public rustdoc, compiled examples and supported-API matrix", flush=True)


if __name__ == "__main__":
    main()
