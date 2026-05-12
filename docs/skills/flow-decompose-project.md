---
title: /flow-decompose-project
nav_order: 17
parent: Skills
---

# /flow-decompose-project

**Phase:** Any (standalone)

**Usage:**

```text
/flow:flow-decompose-project <project description>
/flow:flow-decompose-project --step N --id <id>
```

Decomposes a large project into many GitHub issues with native sub-issue relationships, blocked-by dependencies, and phase labels. Produces a fully linked issue graph ready for autonomous execution via `/flow-start` or `/flow-orchestrate`.

---

## What It Does

Each step is enforced via self-invocation — the skill re-invokes itself with `--step N --id <id>` after each gate, forcing the model to re-read the full skill instructions at every step boundary. The `<id>` is a short UUID generated in Step 1 that scopes all file paths to prevent concurrent session collisions.

| Step | Name | Gate |
|------|------|------|
| 1 | Describe and Decompose | Automatic |
| 2 | Review Issue List | Automatic |
| 3 | Create Epic | Automatic (auto-fix loop on validator failure) |
| 4 | Create Child Issues | Automatic (auto-fix loop with skip-on-cap per child) |
| 5 | Link Relationships | Automatic |
| 6 | Report | Summary + cleanup |

1. **Step 1 — Describe and Decompose:** Invokes `decompose:decompose` with deep codebase exploration. Presents the DAG synthesis and proceeds directly to Step 2 — the user's invocation is the single authorization for the decompose-and-file pipeline.
2. **Step 2 — Review Issue List:** Builds a complete issue list from the DAG with titles, bodies, phase labels, and dependencies. After each child body is composed, two mechanical backstops run alongside each other: a **Backwards-Reasoning Scan** subsection scans every child for forbidden phrasings (`"PR #<N> decided"`, `"kept for backward compatibility"`, `"older plugin versions"`, `"as PR #<N> chose to"`) that ground the decomposition in historical artifacts rather than current code merits — see `.claude/rules/no-backwards-reasoning.md`. An **Include-Bias Scan** subsection then scans each child for defensive scope-shrinkage phrasings (`"Out of scope"` lowercase and title case, `"Non-goals"`, `"would expand scope"`, `"separate code surface"`) that exclude adjacent concerns without naming a concrete blocker — see `.claude/rules/include-bias-in-issues.md`. The issue list is written to state and Step 3 fires directly.
3. **Step 3 — Create Epic:** Validates the epic body via `bin/flow validate-issue-body` and files the parent epic issue. On validator failure, a bounded auto-fix loop (max 5 attempts) corrects the body mechanically or halts the entire flow with `validator_max_retries` (the epic is the parent of every child, so the failure cascades).
4. **Step 4 — Create Child Issues:** Creates all child issues in topological order (leaves first) so dependency numbers exist when referenced. Each issue gets the "Decomposed" label and an auto-derived phase label. The validator auto-fix loop fires per-child; after 5 failed attempts for one child, the skill records the skip and continues to the next child — the Step 6 report surfaces partial coverage.
5. **Step 5 — Link Relationships:** Sets sub-issue relationships (children to epic) and blocked-by dependencies (between children per DAG) via GitHub REST API. Best-effort throughout.
6. **Step 6 — Report:** Presents a summary table of everything created, then cleans up session files.

---

## Issue Format

The parent epic AND every child issue follow the same Body Shape Contract — `flow-decompose-project` is the single source of truth for body shape, and Steps 3 and 4 just write the bytes that Step 2 produces.

Each body has these five sections in this order:

- **Problem** — grounded in codebase evidence (file paths, line numbers, user impact)
- **Acceptance Criteria** — binary pass/fail checklist
- **Implementation Plan** — wrapped in the FLOW-PLAN sentinel pair (`<!-- FLOW-PLAN-BEGIN -->` ... `<!-- FLOW-PLAN-END -->`) and containing seven `###` subsections: Context, Exploration, Risks, Approach, Dependency Graph, Tasks, and Acceptance Criteria. Tasks use `#### Task N:` headers (not numbered list items) so `bin/flow plan-from-issue`'s `count_tasks` populates `code_tasks_total` at flow-start.
- **Files to Investigate** — verified paths
- **Context** — business reason and constraints

The FLOW-PLAN sentinel pair delimits the bytes that `bin/flow plan-from-issue` extracts verbatim and writes to `.flow-states/<branch>/plan.md` when the issue is later picked up via `/flow:flow-start #N`. Without the sentinel pair, `plan-from-issue` rejects the issue with `plan_markers_missing` and the flow halts.

### Pre-Filing Validation

Step 3 (epic) and Step 4 (per-child) invoke `bin/flow validate-issue-body` BEFORE `bin/flow issue`. The validator runs the same sentinel-extraction logic that `bin/flow plan-from-issue` applies at flow-start, so any body that fails this gate is unconsumable downstream and never reaches GitHub. On validator error, the skill applies a mechanical fix and re-runs the validator (max 5 attempts); after 5 failures Step 3 halts the whole flow with `validator_max_retries`, and Step 4 records the skip for that single child and continues with the next child.

The **Dependencies** between children are tracked via native GitHub blocked-by API relationships (created in Step 5).

---

## GitHub Relationships

The skill creates two types of GitHub relationships:

- **Sub-issues** — each child issue is linked as a sub-issue of the epic
- **Blocked-by** — dependency relationships between child issues per the DAG

All relationship creation is best-effort. Native blocked-by relationships are the sole dependency mechanism — `analyze-issues` queries them via GraphQL to detect blocked issues.

---

## Gates

- The user's invocation of `/flow:flow-decompose-project` is the single signal — the pipeline runs unattended through every step.
- Validator gates fire in Steps 3 and 4 — bodies that fail `bin/flow validate-issue-body` enter a 5-attempt auto-fix loop. Step 3 halt-on-cap fails the whole flow; Step 4 skip-on-cap fails a single child and continues.
- Self-invocation enforcement prevents step skipping.
- Session state in `.flow-states/decompose-project-<id>.json` enables resume.
