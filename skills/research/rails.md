# Research — Rails Framework Instructions

## Full Mode Sub-Agent Prompt

Provide these instructions to the Step 2 sub-agent (fill in the scope):

> You are exploring a Rails codebase for the FLOW research phase.
> Research scope: <user's description from Step 1 — paste verbatim>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Never use Bash for file existence checks,
> directory listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Systematically read all code relevant to this feature:
>
> **Models** — Find all related models. For each, read the full class
> hierarchy (model + parent + ApplicationRecord). Look for: before_save,
> after_create, before_destroy callbacks. Check for default_scope (soft
> deletes), self.inheritance_column (no STI), belongs_to/has_many with
> dependent: options. Note the Base/Create split pattern.
>
> **Controllers** — Find affected controllers. Note subdomain, BaseController
> inheritance, params pattern (options OpenStruct), response helpers.
>
> **Workers** — Find affected Sidekiq workers. Read pre_perform!/perform!/
> post_perform! structure. Check config/sidekiq.yml for queue names.
>
> **Routes** — Read config/routes/ files relevant to this feature. Note
> scope with module:, as:, controller:, action: pattern.
>
> **Schema** — Read data/release.sql for all relevant tables. Note column
> types, constraints, indexes, foreign keys.
>
> **Tests** — Search test/support/ for existing create_*! helpers for
> affected models. Note existing test patterns.
>
> **Git history** — Run git log --oneline -10 on key files. Use git blame
> on anything non-obvious.
>
> Return your findings as a structured summary:
>
> - Affected files (full paths)
> - Per-model: class hierarchy, callbacks, associations, soft deletes
> - Per-controller: subdomain, BaseController, params pattern
> - Per-worker: queue name, halt conditions
> - Routes: file and pattern
> - Schema: table structure
> - Test helpers: existing create_*! helpers found
> - Risks: anything that could cause problems (callback chains, soft
>   deletes, Current attribute dependencies)

## Light Mode Sub-Agent Prompt

Provide these instructions to the Light Step 2 sub-agent (fill in the description):

> You are investigating a bug or small change in a Rails codebase.
> Description: <user's description from Light Step 1 — paste verbatim>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Never use Bash for file existence checks,
> directory listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> **Start with recent changes:**
>
> 1. Run `git log --oneline -20` to see recent commits
> 2. Look for commits related to the described issue
> 3. If a recent commit looks relevant, run `git show <sha>` to see the diff
>
> **Then read affected files:**
>
> 1. Read only the files directly related to the issue
> 2. For each model, read the full class hierarchy
> 3. Check `test/support/` for relevant create_*! helpers
>
> Do NOT explore the entire codebase. Stay focused on the files
> directly related to the bug or change.
>
> Return a structured summary: recent relevant commits, affected files
> (full paths), root cause or change needed, per-model class hierarchy
> and callbacks (only for affected models), schema changes needed (if
> any), risks, and existing create_*! helpers found.

## Light Mode Design Object Template

Use this template for `state["design"]` in Light Step 3:

```json
{
  "feature_description": "<user's bug description from Light Step 1>",
  "chosen_approach": "<the fix or change identified during investigation>",
  "rationale": "Identified during light-mode research",
  "schema_changes": [],
  "model_changes": [],
  "controller_changes": [],
  "worker_changes": [],
  "route_changes": [],
  "risks": [],
  "approved_at": "<current UTC timestamp>"
}
```

Populate the change arrays and risks from the investigation findings. Leave
arrays empty where not applicable.

## Framework-Specific Hard Rules

- Always read the full class hierarchy for every affected model — never just the model file
- Always check `test/support/` for existing helpers before noting that tests will be needed
