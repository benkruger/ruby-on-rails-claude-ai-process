# Plan — Python Framework Instructions

## Plan Sections Initialization

Initialise `state["plan"]` if it does not exist:

```json
{
  "sections": {
    "modules": { "status": "pending", "tasks": [] },
    "scripts": { "status": "pending", "tasks": [] },
    "tests":   { "status": "pending", "tasks": [] }
  },
  "current_section": null,
  "tasks": [],
  "approved_at": null
}
```

## Section Verification Sub-Agent Prompt

Provide these instructions to the section verification sub-agent
(fill in the details):

> You are verifying plan tasks for the FLOW plan phase.
> Feature: <feature name from state>
> Section: <current section name>
>
> Design decisions: <paste relevant state["design"] fields>
> Research findings: <paste relevant state["research"] fields>
>
> Tasks to verify:
> <paste the draft tasks for this section>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Never use Bash for file existence checks (`test -f`, `ls`, `stat`, etc.).
>
> For each task, check the codebase:
>
> 1. **File paths** — Use Glob to verify files exist. For new files,
>    use Glob on the parent directory to confirm it exists.
> 2. **Test fixtures** — Use Grep to search conftest.py for existing
>    fixtures. If not found, flag that a fixture creation task is needed.
> 3. **Module context** — Use Read to check imports and dependencies.
>    What modules already exist? What patterns are used?
>
> Return per-task:
>
> - File paths: verified / corrected
> - Available fixtures (if test task)
> - Module context (if module task)
> - Any corrections needed

## Section Definitions

Work through each section below in order:

### Section 1 — Modules

Generate tasks following strict TDD order — test before implementation:

```text
Task N — Test (failing)
  Write failing test for <module>
  Files: tests/test_<module>.py
  Fixture: conftest.py (add fixture if needed)
  TDD: write test first, run it, confirm it fails

Task N+1 — Implementation
  Implement <module>
  Files: lib/<module>.py (or appropriate path)
  Note: imports, function signatures, error handling
```

Pair every implementation task with a test task that comes before it.
Check `conftest.py` — if a fixture is missing, add a task to create it
before the test task that needs it.

**No back navigation on first section.**

### Section 2 — Scripts

*Skip if `design["script_changes"]` is empty.*

Generate tasks following TDD order:

```text
Task N — Test (failing)
  Write failing test for <script>
  Files: tests/test_<script>.py
  Note: Test argument parsing, main flow, error handling

Task N+1 — Implementation
  Implement <script>
  Files: bin/<script> or lib/<script>.py
  Note: argparse, main flow, exit codes
```

### Section 3 — Integration Tests

Generate tasks for any cross-cutting test coverage:

```text
Task N — Integration test
  Write integration test for <end-to-end flow>
  Files: tests/test_<name>_integration.py
  Note: Full lifecycle — setup, execute, verify
```

## Plan Save Schema

Write to `.flow-states/<branch>.json` under `plan`:

```json
{
  "sections": {
    "modules": { "status": "approved", "tasks": [1,2,3,4] },
    "scripts": { "status": "approved", "tasks": [5,6] },
    "tests":   { "status": "approved", "tasks": [7] }
  },
  "current_section": null,
  "tasks": [
    {
      "id": 1,
      "section": "modules",
      "type": "test",
      "description": "Write failing test for flow_utils",
      "files": ["tests/test_flow_utils.py"],
      "tdd": true,
      "status": "pending"
    }
  ],
  "approved_at": "<current UTC timestamp>"
}
```
