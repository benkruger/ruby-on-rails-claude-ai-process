# Review — Python Framework Instructions

## Diff Analysis Sub-Agent Prompt

Provide these instructions to the Step 1 sub-agent (fill in the details):

> You are analyzing a feature diff for the FLOW review phase.
> Feature: <feature name from state>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Only use Bash for git commands (git diff,
> git log, git blame). Never use Bash for file existence checks, directory
> listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Approved design:
> <paste state["design"] — chosen_approach, module_changes, test_changes,
> script_changes>
>
> Research risks:
> <paste state["research"]["risks"]>
>
> Plan tasks:
> <paste state["plan"]["tasks"] summaries>
>
> First, get the full diff:
>
> ```bash
> git diff origin/main...HEAD
> ```
>
> Read every changed file completely. Then check:
>
> **Design alignment:**
>
> - Do module changes match design["module_changes"]?
> - Do test changes match design["test_changes"]?
> - Do script changes match design["script_changes"]?
> - Flag any deviation — minor drift or major mismatch.
>
> **Research risk coverage:**
>
> - For each risk in the list, confirm it was handled in the diff.
> - Flag any risk not addressed.
>
> **Python anti-pattern check:**
>
> Imports: no circular imports, no wildcard imports (`from x import *`).
> Mutable defaults: no mutable default arguments (`def f(x=[])`).
> Error handling: no bare `except:`, no broad `except Exception`
> without re-raise.
> Type safety: consistent use of type hints if the project uses them.
> Tests: fixtures used where appropriate, both branches tested,
> assertions present.
> Lint: scan diff for noqa/type:ignore comments.
> Code clarity: descriptive names, no inline comments, no over-engineering.
>
> Return structured findings in three categories:
>
> 1. Design alignment issues (with file:line references)
> 2. Uncovered research risks (with which risk and why)
> 3. Anti-pattern violations (with file:line and what to fix)
>
> If a category has no findings, say so explicitly.

## Framework-Specific Hard Rules

- Any `# noqa` or `# type: ignore` comment in the diff is a finding — remove it and fix the code
- Any modification to lint configuration in the diff is a finding — revert it and fix the code
