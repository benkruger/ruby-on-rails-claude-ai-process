---
title: Skills
nav_order: 3
---

# Skills

Skills are the building blocks of the ROR workflow. Some are tied to a specific phase and invoked automatically as part of that phase. Others are utility skills available at any point.

All skills are namespaced under `ror:` and announce themselves clearly when they start and finish.

---

## Phase Skills

These skills correspond directly to a workflow phase. Each one starts and ends with a banner so you always know where you are.

| Skill | Phase | Description |
|-------|-------|-------------|
| [`/ror:start`](ror-start.md) | 0 — Start | Create the worktree, upgrade gems, open the PR, configure permissions |
| `/ror:research` | 1 — Research | *(coming soon)* |
| `/ror:design` | 2 — Design | *(coming soon)* |
| `/ror:plan` | 3 — Plan | *(coming soon)* |
| `/ror:implement` | 4 — Implement | *(coming soon)* |
| `/ror:test` | 5 — Test | *(coming soon)* |
| `/ror:review` | 6 — Review | *(coming soon)* |
| `/ror:ship` | 7 — Ship | *(coming soon)* |

---

## Utility Skills

These skills are available at any point in the workflow, regardless of phase.

| Skill | Description |
|-------|-------------|
| [`/ror:commit`](ror-commit.md) | Review the full diff, approve or deny, then git add + commit + push |
| [`/ror:status`](ror-status.md) | Show current phase, PR link, phase checklist, and what comes next |
