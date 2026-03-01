# Research — Python Framework Instructions

## Full Mode Sub-Agent Prompt

Provide these instructions to the Step 2 sub-agent (fill in the scope):

> You are exploring a Python codebase for the FLOW research phase.
> Research scope: <user's description from Step 1 — paste verbatim>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Never use Bash for file existence checks,
> directory listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Systematically read all code relevant to this feature:
>
> **Modules** — Find all related Python modules. Read each module fully.
> Look for: imports, class definitions, function signatures, module-level
> state, `__init__.py` exports.
>
> **Scripts** — Find affected CLI scripts or entry points (in `bin/`,
> `scripts/`, or package `__main__.py`). Read argument parsing, main
> flow, and error handling.
>
> **Configuration** — Read relevant config files: `pyproject.toml`,
> `setup.cfg`, `conftest.py`, CI config, `.yml` files.
>
> **Tests** — Find existing test files for affected modules. Read
> `conftest.py` for shared fixtures. Note existing test patterns and
> helper functions.
>
> **Git history** — Run git log --oneline -10 on key files. Use git blame
> on anything non-obvious.
>
> Return your findings as a structured summary:
>
> - Affected files (full paths)
> - Per-module: imports, classes, key functions, dependencies
> - Per-script: arguments, main flow, error handling
> - Test fixtures: existing conftest.py fixtures and helpers found
> - Risks: anything that could cause problems (circular imports,
>   global state, missing error handling)

## Light Mode Sub-Agent Prompt

Provide these instructions to the Light Step 2 sub-agent (fill in the description):

> You are investigating a bug or small change in a Python codebase.
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
> 2. For each module, read its imports and dependencies
> 3. Check `conftest.py` for relevant fixtures
>
> Do NOT explore the entire codebase. Stay focused on the files
> directly related to the bug or change.
>
> Return a structured summary: recent relevant commits, affected files
> (full paths), root cause or change needed, module dependencies,
> risks, and existing test fixtures found.

## Light Mode Design Object Template

Use this template for `state["design"]` in Light Step 3:

```json
{
  "feature_description": "<user's bug description from Light Step 1>",
  "chosen_approach": "<the fix or change identified during investigation>",
  "rationale": "Identified during light-mode research",
  "module_changes": [],
  "test_changes": [],
  "script_changes": [],
  "risks": [],
  "approved_at": "<current UTC timestamp>"
}
```

Populate the change arrays and risks from the investigation findings. Leave
arrays empty where not applicable.

## Framework-Specific Hard Rules

- Always read module imports and dependencies before modifying
- Always check `conftest.py` for existing fixtures before noting that tests will be needed
