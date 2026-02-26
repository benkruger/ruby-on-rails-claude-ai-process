---
title: /flow:review
nav_order: 8
parent: Skills
---

# /flow:review

**Phase:** 6 — Review

**Usage:** `/flow:review`

Systematic review against design, research risks, and Rails
anti-patterns. Fixes findings, runs bin/ci after every fix, then
transitions to Security.

---

## What It Checks

| Area | What |
|------|------|
| Design alignment | Implementation matches `state["design"]` |
| Research risks | Every risk in `state["research"]["risks"]` accounted for |
| Associations | `inverse_of:`, `dependent:`, `class_name:` all explicit |
| Queries | No N+1, no queries in views, no arbitrary `.first`/`.last` |
| Callbacks | `Current` used correctly, no `update_column` |
| Soft deletes | `.unscoped` used only where appropriate |
| Tests | `create_*!` helpers, both branches covered, assertions present |
| Clarity | Descriptive names, no inline comments, no over-engineering |

---

## Fixing Findings

- Minor → fix directly, commit, re-run bin/ci
- Significant → AskUserQuestion: fix here or go back to Code/Plan/Design/Research

---

## Gates

- bin/ci must be green after every fix
- bin/ci must be green before transitioning to Security
- Full diff must be read before review begins
- Can return to Code, Plan, Design, or Research
