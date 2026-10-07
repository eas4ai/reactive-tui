#!/usr/bin/env python3
"""Which Rust toolchain each command of a GitHub Actions workflow selects,
read from the workflow file alone.

rustup picks a toolchain in this order: the `+<toolchain>` argument of the
cargo call, then the RUSTUP_TOOLCHAIN variable, then the runner's default. A
workflow spells the first two through its environment (`TOOLCHAIN: ${{
matrix.toolchain }}` on the job, `cargo "+$TOOLCHAIN"` in a step), and a
step's own `env` overrides the job's, which overrides the workflow's. So what
a step runs at is a question of the environment in force at that step for
the matrix entry in hand. This module answers it for the two readers of
ci.yml: scripts/cairn/ci_workflow.py (BAR-012) and
scripts/check-pre-release-ci.py (RID-001). Nothing here runs a command, and
a value the reader cannot turn into a literal, an expression other than a
matrix reference or an unset variable, resolves to None, which both readers
refuse.
"""
from __future__ import annotations

import re

MATRIX_REF = re.compile(r"\$\{\{\s*matrix\.([A-Za-z_][A-Za-z0-9_-]*)\s*\}\}")
EXPRESSION = re.compile(r"\$\{\{")
SHELL_VAR = re.compile(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$")
# `cargo <first argument> <rest of the line>`; `cargo-deny` is not a call.
CARGO_CALL = re.compile(r"(?<![\w/.-])cargo[ \t]+(\S+)([^\n]*)")
INSTALL = re.compile(r"rustup[ \t]+toolchain[ \t]+install[ \t]+(\S+)")
OS_CONDITION = re.compile(r"^\s*runner\.os\s*(==|!=)\s*'([A-Za-z]+)'\s*$")
RUNNER_OS = {"ubuntu-": "Linux", "macos-": "macOS", "windows-": "Windows"}
# The variables that choose a toolchain; a step may not override them.
TOOLCHAIN_VARIABLES = ("TOOLCHAIN", "RUSTUP_TOOLCHAIN")
# A stable release named in full, with a target triple when the job names one.
RELEASE = re.compile(r"^(\d+)\.(\d+)\.(\d+)(-[A-Za-z0-9_.-]+)?$")


def runner_os(os_name: str) -> str | None:
    """`runner.os` of a runner label: ubuntu-24.04 is Linux, macos-14 is macOS,
    windows-2022 is Windows; None for a label this reader does not know."""
    for prefix, label in RUNNER_OS.items():
        if os_name.startswith(prefix):
            return label
    return None


def release_pair(text: str | None) -> tuple[int, int] | None:
    """major and minor of a release or a rust-version, None otherwise."""
    match = re.match(r"^(\d+)\.(\d+)", text or "")
    return (int(match.group(1)), int(match.group(2))) if match else None


def substitute(text: str, entry: dict | None) -> str:
    """`${{ matrix.<key> }}` becomes the entry's value; other expressions stay."""
    if not entry:
        return text
    return MATRIX_REF.sub(lambda m: str(entry[m.group(1)]) if m.group(1) in entry else m.group(0), text)


def effective_env(*layers: dict | None, entry: dict | None = None) -> dict[str, str]:
    """The environment a step sees: the workflow's, the job's and the step's
    `env` in that order, each overriding the one before, with matrix
    references substituted for the entry."""
    env: dict[str, str] = {}
    for layer in layers:
        if isinstance(layer, dict):
            for key, value in layer.items():
                env[str(key)] = substitute(str(value), entry)
    return env


def resolve(value: str | None, env: dict[str, str], depth: int = 0) -> str | None:
    """The literal a value stands for: quotes drop, a shell variable reads the
    environment; None for an expression, an unset variable or nothing."""
    if value is None or depth > 4:
        return None
    text = value.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        text = text[1:-1]
    if EXPRESSION.search(text):
        return None
    match = SHELL_VAR.match(text)
    if match:
        return resolve(env.get(match.group(1)), env, depth + 1)
    return text or None


def step_runs_on(step: dict, os_label: str | None) -> bool | None:
    """Whether the step runs on that runner OS: True or False when the step's
    `if` is `runner.os == '...'` or `!=` (or absent), None for a condition
    this reader does not understand."""
    condition = str(step.get("if", "") or "").strip()
    if not condition:
        return True
    match = OS_CONDITION.match(condition)
    if not match or os_label is None:
        return None
    equal = match.group(2).lower() == os_label.lower()
    return equal if match.group(1) == "==" else not equal


def cargo_calls(run: str, env: dict[str, str]) -> list[dict]:
    """Each cargo call in a step's text: `first`, its first argument as
    written; `explicit`, whether that argument is a `+<toolchain>` pin;
    `command`, the call as `cargo <arguments>` with the pin removed; and
    `selected`, the toolchain the call runs at: the pin, else
    RUSTUP_TOOLCHAIN from the environment, else None (the runner's default,
    or a value the reader cannot resolve)."""
    calls = []
    for match in CARGO_CALL.finditer(run):
        first, rest = match.group(1), match.group(2)
        bare = first.strip("\"'")
        if bare.startswith("+"):
            selected = resolve(bare[1:], env)
            command = ("cargo " + rest.strip()).strip()
            explicit = True
        else:
            selected = resolve(env.get("RUSTUP_TOOLCHAIN"), env) if "RUSTUP_TOOLCHAIN" in env else None
            command = f"cargo {first}{rest}".rstrip()
            explicit = False
        calls.append({"first": first, "explicit": explicit, "command": command, "selected": selected})
    return calls


def installs(run: str, env: dict[str, str]) -> list[str | None]:
    """The toolchains `rustup toolchain install` installs in a step's text,
    None for one the reader cannot resolve."""
    return [resolve(match.group(1), env) for match in INSTALL.finditer(run)]
