# Review — Rails Framework Instructions

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
> <paste state["design"] — chosen_approach, schema_changes, model_changes,
> controller_changes, worker_changes, route_changes>
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
> - Do schema changes match design["schema_changes"]?
> - Do model decisions match design["model_changes"]?
> - Do controller/route changes match design?
> - Do worker changes match design?
> - Flag any deviation — minor drift or major mismatch.
>
> **Research risk coverage:**
>
> - For each risk in the list, confirm it was handled in the diff.
> - Flag any risk not addressed.
>
> **Rails anti-pattern check:**
>
> - Associations: every belongs_to/has_many has inverse_of:, dependent:,
>   class_name: explicit
> - Queries: no N+1, no DB queries in views, no .first/.last for defaults
> - Callbacks: Current attribute usage correct, no update_column
> - Models: self.table_name in namespaced Base, no STI
> - Soft deletes: .unscoped usage correct
> - Workers: halt! in pre_perform!, queue matches sidekiq.yml
> - Tests: create_*! helpers used, both branches tested, assertions present
> - RuboCop: scan diff for rubocop:disable comments, check .rubocop.yml changes
> - Code clarity: descriptive names, no inline comments, no over-engineering
>
> Return structured findings in three categories:
>
> 1. Design alignment issues (with file:line references)
> 2. Uncovered research risks (with which risk and why)
> 3. Anti-pattern violations (with file:line and what to fix)
>
> If a category has no findings, say so explicitly.

## Framework-Specific Hard Rules

- Any `# rubocop:disable` comment in the diff is an automatic finding — remove it and fix the code
- Any modification to `.rubocop.yml` in the diff is an automatic finding — revert it and fix the code
