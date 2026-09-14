# Working agreement

This repository runs under Cairn. The specification in docs/spec/ is
the contract, the roadmap names the current commitment, and the
referee, `cairn`, reads the repository and names the next action. This
file says what each party does when it is their turn.

Cairn is a discipline tool, not a security boundary. It checks recorded
results and freshness; it cannot judge whether a review is thorough or a
mechanism proves its requirements. A command that always succeeds can
produce passes without testing anything. The developer must challenge
unsound mechanisms, and the agent must demonstrate what makes them fail.

## The agent

Wake. Before anything else, run `cairn wake`. Read the glossary, the
keystone, the roadmap, the current commitment, and the decision records
for what it names. Nothing you remember from an earlier session counts; the
repository does.

Act on the verdict, and only on it.

- Resolvable: do the one action named. Then run `cairn wake` again.
  Do not stop while the verdict is Resolvable.
- Escalate: present the escalation file to the developer, in its own
  words, and stop. Do nothing else until it is answered.
- Done: the commitment is complete. Report it, and stop. The next
  commitment is the developer's to name.

When wake says `reply <slug>`, read the developer's question and append
your explanation with `cairn answer <slug> "<explanation>"`. Run wake again
to return the decision to the developer. An `ask` answer authorizes only
an explanation; it does not authorize implementation.

Before you change code, write `.cairn/in-progress`:

    action: implement | build-decision | run-mechanism | review
    target: <requirement, decision slug, or commitment slug>
    base: <commit identifier>
    started: <iso timestamp>

Remove it when the change is committed. On wake, an existing record is
reconciled before any new work: finish or abandon the action it names.

Check execution has a separate working-tree lock. When wake names
`cairn-check.lock`, wait for a live owner. If the owner is dead or the
record is incomplete, inspect the command and any surviving children;
remove the named lock only after execution has stopped. Keep the agent's
in-progress record until its own action is finished or abandoned.

Commit before you check. `cairn check` records evidence only against a
committed tree and refuses a dirty declared input. Your own test runs
while editing are how you work; they are not evidence. Commit the new
evidence receipts and their output files after each check. Evidence
history is tracked; only .cairn/in-progress stays ignored.

Keep the candidate stable throughout a check. Cairn compares it with the
commit before execution and validates it again before recording evidence.
A changed candidate keeps its diagnostic output but gets no receipt.
Missing or damaged captured output needs a new check, never an edited
historical receipt.

When wake says `run <REQ>`, use `cairn check <REQ>`; Cairn selects its
mechanism for you. Declare every file that can affect the result, including
shared dependencies. Broad directories are easier to maintain but rerun
checks for unrelated edits. Narrow paths reduce reruns but need updating
when dependencies change. Do not omit a dependency just to shorten a run.

When wake says `review mechanism <REQ>`, compare the check with the
revised requirement and falsifier without changing code. Record what you
examined and any mismatch in the existing commitment review. Fix a
mismatch as a separate implementation action. Try a safe violating example
and the corrected case; record the results, or why that demonstration is
impractical. Add the exact `REQ sha256:...` entry wake prints to
the declaration's `reviewed:` list only after that review. Commit before
check. Copying the digest alone does not establish that the check works.
Apply the same failure demonstration when building a new check.

Decide by level. Routine: decide, no record. Judged and above: record
it with `cairn decide` before you build it. Blocking: `cairn escalate`,
then stop. Three attempts at a requirement without new passing evidence
make the next decision about it Blocking. An attempt is one distinct
digest of the mechanism's declared inputs among the failing checks
since the last pass; reruns, documentation changes, and a return to a
digest already tried add nothing, and the first check of a
requirement is its baseline, never an attempt. A failure no change
inside the footprint can address is not an attempt at all: it is an
escalation, and the wake names DEC-019 when it sees three runs at one
digest. A failing requirement every commitment inherits is repaired
under the current commitment; its mechanism's inputs are already in
the footprint, and a fix outside them means the declaration was
incomplete: declare, then fix.

Out of scope is captured, never built. An idea outside the current
commitment goes to `cairn backlog --title ... --body ... --from <REQ>`.
It enters a commitment only when the developer writes it into the
specification and names it there.

Review before Done. When every requirement passes, examine the work for
what the mechanisms would miss, record what you attacked and what you
found in `.cairn/reviews/<slug>.md`, and change no code while you look.
A finding is resolved as its own work, after the review is recorded.

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

## The developer

An escalation awaits you in `.cairn/escalations/<slug>.md`. Answer it:

    cairn answer <slug> ok | instead <what> | ask <question>

Use `ask <question>` whenever the explanation is unclear. The agent will
reply in the same file, then the decision returns to you. Answer `ok`,
`instead <what>`, or ask another question. Only `ok` or `instead` closes it.

A queued decision awaits you in `.cairn/queue/<slug>`; its record is in
docs/decisions/. The agent did not wait for you. Reading it and
removing the queue entry in a commit is the review; the commit's author
and date are the mark. To reverse it, have the agent supersede the
record with the cause named.

The next commitment is yours. Write the requirement into the
specification with its falsifier, name it in a commitment file, and
move the roadmap's Current: line.

Merge other branches with `git merge --no-ff` so their commits stay off
this loop's first-parent history. Cairn checks each of this loop's own
commits; reverting a change does not erase a footprint breach. Declare a
missing input when it belongs to the commitment. Otherwise capture the
work in the backlog and ask the developer to resolve its scope.

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

<!-- gitnexus:end -->
