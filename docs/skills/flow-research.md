---
title: /flow:research
nav_order: 4
parent: Skills
---

# /flow:research

**Phase:** 2 — Research

**Usage:** `/flow:research`

Explores the codebase before any design or implementation begins. Reads all affected code, discovers risks specific to Rails conventions, asks clarifying questions via tabbed UI, and documents findings in `.flow-states/<branch>.json`.

---

## What It Does

1. Reads feature context from `.flow-states/<branch>.json`
2. Explores all affected models (full class hierarchy), controllers, workers, routes, and schema
3. Formulates clarifying questions based on what was found
4. Presents questions in batches of up to 4 using the tabbed `AskUserQuestion` UI — navigate freely with ← → arrows
5. Documents all findings into `.flow-states/<branch>.json["research"]`
6. Presents a clean findings summary
7. Gates on user approval before proceeding to Design

---

## Rails-Specific Checks

Every Research run checks for:

| Concern | Why it matters |
|---------|---------------|
| Callback hierarchy | `before_save` in parent classes silently overwrite values passed to `update!` |
| Soft deletes | `default_scope { where(active: true) }` hides deleted records — use `.unscoped` when needed |
| Base/Create split | Models have separate classes for reading vs creating — understand both |
| Test helpers | `test/support/` contains `create_*!` helpers — never use `Model::Create.create!` directly |
| Worker queues | Check `config/sidekiq.yml` for correct queue names before adding new workers |
| Schema | `data/release.sql` is the source of truth — not migrations |

---

## Findings Stored In State

Research writes to `.flow-states/<branch>.json["research"]`:

- `clarifications` — every Q&A pair from the session
- `affected_files` — all files that will need to change
- `risks` — Rails-specific gotchas discovered
- `open_questions` — anything still unresolved
- `summary` — plain English description of what exists

If Research is revisited, prior findings are extended — never discarded.

---

## Light Mode Behavior

When `state["mode"] == "light"` (set by `/flow:start --light`), Research uses a
"recent changes first" protocol:

1. Asks the user to describe the bug or change
2. Checks `git log` for recent relevant commits before exploring deeply
3. Launches a focused sub-agent on only the affected files (not the full codebase)
4. Writes both `state["research"]` and a simplified `state["design"]` object
5. Transitions directly to Phase 4: Plan (Design was already marked skipped)

The simplified design object contains the factual description of what needs to
change — not design alternatives. Plan and Review read it unchanged.

---

## Gates

- Never proposes solutions — that is Design's job (in light mode, the design object is factual)
- Never writes or modifies any application code
- Always reads full class hierarchy for every affected model
- Requires user approval before proceeding to Phase 3: Design (or Phase 4: Plan in light mode)

---

## See Also

- [FLOW State Schema](../reference/flow-state-schema.md)
