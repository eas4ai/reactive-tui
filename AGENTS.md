# Working agreement

This repository runs under Cairn. `docs/spec/` is the contract, the roadmap names the current commitment, and `cairn` reads the repository and names the next action. This file states the move for each verdict and action. The kernel does not parse it; it is protected and changes only between commitments, by developer authorization.

## The agent

Run `cairn wake` first, every session, and act on the verdict only. With hooks the verdict is printed before every turn; this agreement holds without them.

- Resolvable: do the one action named until its predicate holds, leave the required trace (branch commit, snapshot or log record), then run `cairn wake` again.
- Waiting: an escalation is unanswered and wake printed its five fields verbatim. Add nothing and stop; the developer answers.
- Done: a done record exists and nothing waits. Report it and stop. Backlog waiting: wake names `promote` instead.

Before changing a declared input: `cairn begin <action> <target>` (`--touch <path>` declares a new file); it prints the lease sha. After the commit: `cairn end --lease <sha>` with that sha, so a stale end never closes another session's lease. Commit before `cairn check`; an uncommitted declared input makes wake name `commit` or `record` before anything else. Push with `cairn push`: it pushes the branch and both durable refs atomically where the remote allows and in the safe order otherwise. Never push `refs/cairn/*` with plain `git push`.

The move for each action wake can name:

- `repair PATH`: make the hand-written file read under its grammar; change no unrelated byte.
- `recover TRANSACTION`: run `cairn recover <transaction>`.
- `reconcile ACTION`: finish the leased action and `cairn end`, or abandon it with `cairn end --abandon`; a lease left by a dead session needs no `--lease`.
- `scope PATH`: restore the path to its allowed base and run `cairn scope <breach> restore`, or ask the developer to keep it with `cairn escalate` and, after `ok`, `cairn scope <breach> keep`.
- `fix ITEM`: write a test that fails, make it pass, commit, check, then `cairn fix <item>`.
- `record PATH` and `commit PATH`: put the change under a lease with `cairn begin`, commit it, or revert it.
- `declare REQ`: `cairn declare` a mechanism naming REQ; show it fail on a violating example before trusting it.
- `run REQ`: `cairn check REQ`.
- `implement REQ`: read the latest receipt and its output, change the code under a lease, commit, `cairn end`, `cairn check REQ`.
- `escalate REQ`: three attempts failed; `cairn escalate` with the five fields before any fourth attempt.
- `review mechanism REQ`: `cairn review mechanism REQ <fail-receipt>` after checking the failure was the stated violation.
- `capture ITEM`: `cairn outside <item> --reason "<why it is not this commitment's work>"`, or escalate.
- `review SLUG`: `cairn review SLUG --file <path>` naming a file that answers Q1 to Q6 for every target with observed commands, paths or outputs.
- `report SLUG`: `cairn brief SLUG`; start one adversary with none of your context on the brief and projection only; wait; `cairn report SLUG --file <its report>`.
- `resolve SLUG N`: fix finding N as its own work, commit, then `cairn resolve SLUG N "<how>"`; or dispute it with `cairn escalate`.
- `accept SLUG`: give the adversary the report, the resolutions and the cumulative delta; `cairn accept SLUG --file <its acceptance>`.
- `build DECISION`: build what the decision says, commit, then `cairn realize <id> --subject "<what was built>"`.
- `done SLUG`: `cairn done SLUG`.
- `promote`: choose one backlog item by judgment; `cairn promote <item>`. Promotion never Agrees text.
- `reply SLUG`: `cairn reply SLUG "<explanation>"`; an `ask` answer authorizes an explanation only.

Out of scope is captured, never built: `cairn item --backlog`, `--next-feature`, or `--defect --from <REQ>`. A defect against this commitment's requirement is worked here, not captured.

Decide by level: Routine and Judged leave no record; Blocking is `cairn escalate` and stops.

A Consequential decision -- one with real options and a recommendation, tied to this commitment's requirements -- takes one more step first: `cairn measure` with the same fields `cairn decide`/`cairn escalate` would take (`--commitment`, `--concern`, `--question`, `--recommendation`, `--because`, `--if-wrong`, `--instead`, `--option`, `--path`, `--decision`). It prints five scored dimensions (evidence, reach, contract fit, new surface, ambiguity), a composite, and `suggested: agent` or `suggested: developer`. The suggestion is information, not consent: read it and the five numbers, then either `cairn decide --consequential --commitment ...` (the same flags, continuing) or `cairn escalate --consequential --commitment ...` (the same flags, stopping) -- your own judgment, whatever the suggestion says. Two things bypass your judgment entirely and are always `cairn escalate --consequential`: the measurement's own floor (a draft that would change an Agreed requirement's text or falsifier, the working agreement, or data that cannot be regenerated) and its veto (an option that reaches too far, changes the contract, or opens too much new surface) -- `cairn decide --consequential` rejects either one and names the measurement that caught it. Put your real evidence in `--because`: a command, a file, quoted output, or the failing test and the falsifier it maps to raise the evidence score and lower the composite.

## The developer

Answer an escalation with `cairn answer <slug> ok | instead <text> | ask <text>`. Read the queue with `cairn decisions` and mark each with `cairn decisions --read <id>`. Run `cairn authorize` after changing `docs/spec/`, `AGENTS.md` or `.cairn/settings.json` between commitments. After Done, open the next work with `/next-feature`.

## Writing for the developer

Write so the developer can understand the choice and its consequences
after one reading. Apply this to questions, specifications, records,
and progress reports.

- Name who does what and what changes for the user or system. Use
  familiar words, concrete examples, and short sentences.
- Match the explanation to the developer's knowledge. Explain an
  unfamiliar technical term when it matters to the decision. Keep
  technical detail that changes the answer; remove jargon that only
  makes the sentence sound authoritative.
- When asking for a decision, state the actual choice, your
  recommendation, why it helps, and what the alternative changes.
  Explain costs or risks in terms of what could happen. For an
  escalation, put this information in the existing fields.
- With a decision or agreement prompt, say: "If this isn't clear, ask
  me to explain it another way before you decide." For an escalation,
  put the invitation after the options on the existing Reply line.
- Distinguish what you observed from what you assume or do not know.
  Keep important limits visible when shortening an explanation.
- Before sending, ask whether the developer can tell what their answer
  would authorize without decoding internal names or abstract labels.
  Rewrite any sentence that hides that choice.
- If a reply shows a misunderstanding, explain the choice again before
  treating the reply as agreement. Silence alone is not confirmation.

For example: "Should the app save unfinished drafts so users can reopen
them later?" names the behavior the developer is deciding about.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **reactive-tui** (22846 symbols, 58671 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## When Debugging

1. `gitnexus_query({query: "<error or symptom>"})` — find execution flows related to the issue
2. `gitnexus_context({name: "<suspect function>"})` — see all callers, callees, and process participation
3. `READ gitnexus://repo/reactive-tui/process/{processName}` — trace the full execution flow step by step
4. For regressions: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})` — see what your branch changed

## When Refactoring

- **Renaming**: MUST use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` first. Review the preview — graph edits are safe, text_search edits need manual review. Then run with `dry_run: false`.
- **Extracting/Splitting**: MUST run `gitnexus_context({name: "target"})` to see all incoming/outgoing refs, then `gitnexus_impact({target: "target", direction: "upstream"})` to find all external callers before moving code.
- After any refactor: run `gitnexus_detect_changes({scope: "all"})` to verify only expected files changed.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Tools Quick Reference

| Tool | When to use | Command |
|------|-------------|---------|
| `query` | Find code by concept | `gitnexus_query({query: "auth validation"})` |
| `context` | 360-degree view of one symbol | `gitnexus_context({name: "validateUser"})` |
| `impact` | Blast radius before editing | `gitnexus_impact({target: "X", direction: "upstream"})` |
| `detect_changes` | Pre-commit scope check | `gitnexus_detect_changes({scope: "staged"})` |
| `rename` | Safe multi-file rename | `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` |
| `cypher` | Custom graph queries | `gitnexus_cypher({query: "MATCH ..."})` |

## Impact Risk Levels

| Depth | Meaning | Action |
|-------|---------|--------|
| d=1 | WILL BREAK — direct callers/importers | MUST update these |
| d=2 | LIKELY AFFECTED — indirect deps | Should test |
| d=3 | MAY NEED TESTING — transitive | Test if critical path |

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/reactive-tui/context` | Codebase overview, check index freshness |
| `gitnexus://repo/reactive-tui/clusters` | All functional areas |
| `gitnexus://repo/reactive-tui/processes` | All execution flows |
| `gitnexus://repo/reactive-tui/process/{name}` | Step-by-step execution trace |

## Self-Check Before Finishing

Before completing any code modification task, verify:
1. `gitnexus_impact` was run for all modified symbols
2. No HIGH/CRITICAL risk warnings were ignored
3. `gitnexus_detect_changes()` confirms changes match expected scope
4. All d=1 (WILL BREAK) dependents were updated

## Keeping the Index Fresh

After committing code changes, the GitNexus index becomes stale. Re-run analyze to update it:

```bash
npx gitnexus analyze
```

If the index previously included embeddings, preserve them by adding `--embeddings`:

```bash
npx gitnexus analyze --embeddings
```

To check whether embeddings exist, inspect `.gitnexus/meta.json` — the `stats.embeddings` field shows the count (0 means no embeddings). **Running analyze without `--embeddings` will delete any previously generated embeddings.**

> Claude Code users: A PostToolUse hook handles this automatically after `git commit` and `git merge`.

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

## Commands

- `cargo test --locked --no-fail-fast` - run the full test suite
- `cargo fmt --all -- --check` - run the format task
- `cargo clippy --locked --all-targets -- -D warnings` - run the lint task

## Code Map

- `src` - application source
- `tests` - automated tests
- `docs` - project documentation
- `.github` - project configuration

## Conventions

- Declare child modules with `pub mod` and re-export with `pub use`.
- Import first-party code with `use crate::...`.
- Use `.rs` for Rust source files.

<!-- gitnexus:end -->
