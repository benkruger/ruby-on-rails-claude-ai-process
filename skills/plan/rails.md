# Plan — Rails Framework Instructions

## Plan Sections Initialization

Initialise `state["plan"]` if it does not exist:

```json
{
  "sections": {
    "schema":      { "status": "pending", "tasks": [] },
    "models":      { "status": "pending", "tasks": [] },
    "workers":     { "status": "pending", "tasks": [] },
    "controllers": { "status": "pending", "tasks": [] },
    "integration": { "status": "pending", "tasks": [] }
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
> 2. **Test helpers** — Use Grep to search test/support/ for create_*!
>    helpers. If not found, flag that a helper creation task is needed.
> 3. **Route context** — Use Read to check route files. What routes already
>    exist in the target file? What patterns are used?
> 4. **Schema context** — Use Read to check data/release.sql for related tables.
>
> Return per-task:
>
> - File paths: verified / corrected
> - Available helpers (if test task)
> - Route context (if route/controller task)
> - Schema context (if schema task)
> - Any corrections needed

## Section Definitions

Work through each section below in order:

### Section 1 — Schema

*Skip if `design["schema_changes"]` is empty.*

Generate tasks for all `data/release.sql` changes:

```text
Task 1 — Schema
  Add <table_name> table to data/release.sql
  Files: data/release.sql
  Note: Column types, constraints, indexes, foreign keys
```

One task per table or significant column change. Be specific — include
column names, types, and any constraints.

**No back navigation on first section.**

### Section 2 — Models and Tests

Generate tasks following strict TDD order — test before implementation:

```text
Task N — Test (failing)
  Write failing test for <Model>::Base — <what it tests>
  Files: test/models/<model>/base_test.rb
  Helper: test/support/<model>_helpers.rb (create_<model>! if needed)
  TDD: write test first, run it, confirm it fails

Task N+1 — Implementation
  Implement <Model>::Base
  Files: app/models/<model>/base.rb
  Note: table_name, soft delete, associations, callbacks

Task N+2 — Test (failing)
  Write failing test for <Model>::Create
  Files: test/models/<model>/create_test.rb

Task N+3 — Implementation
  Implement <Model>::Create
  Files: app/models/<model>/create.rb
```

Pair every implementation task with a test task that comes before it.
Check `test/support/` — if a `create_*!` helper is missing, add a task
to create it before the test task that needs it.

### Section 3 — Workers

*Skip if `design["worker_changes"]` is empty.*

Generate tasks following TDD order:

```text
Task N — Test (failing)
  Write failing test for <Worker>
  Files: test/workers/<worker>_test.rb
  Note: Test pre_perform!, perform!, post_perform! separately
        Test halt! conditions

Task N+1 — Implementation
  Implement <Worker>
  Files: app/workers/<worker>.rb
  Note: Queue from config/sidekiq.yml, pre_perform!/perform!/post_perform! structure
```

### Section 4 — Controllers and Routes

Generate tasks following TDD order:

```text
Task N — Route
  Add route to config/routes/<file>.rb
  Files: config/routes/<file>.rb
  Note: scope with module:, as:, controller:, action: explicitly

Task N+1 — Test (failing)
  Write failing controller test
  Files: test/controllers/<path>_test.rb
  Note: authenticate_admin! or authenticate_user! as needed

Task N+2 — Implementation
  Implement <Controller>#<action>
  Files: app/controllers/<path>_controller.rb
  Note: options OpenStruct params, render_ok/render_error responses
```

### Section 5 — Integration Tests

Generate tasks for any cross-cutting test coverage:

```text
Task N — Integration test
  Write integration test for <end-to-end flow>
  Files: test/integration/<name>_test.rb
  Note: Full lifecycle — create, read, edge cases
```

## Plan Save Schema

Write to `.flow-states/<branch>.json` under `plan`:

```json
{
  "sections": {
    "schema":      { "status": "approved", "tasks": [1] },
    "models":      { "status": "approved", "tasks": [2,3,4,5] },
    "workers":     { "status": "approved", "tasks": [6,7] },
    "controllers": { "status": "approved", "tasks": [8,9,10] },
    "integration": { "status": "approved", "tasks": [11,12] }
  },
  "current_section": null,
  "tasks": [
    {
      "id": 1,
      "section": "schema",
      "type": "schema",
      "description": "Add payments table to data/release.sql",
      "files": ["data/release.sql"],
      "tdd": false,
      "status": "pending"
    }
  ],
  "approved_at": "<current UTC timestamp>"
}
```
