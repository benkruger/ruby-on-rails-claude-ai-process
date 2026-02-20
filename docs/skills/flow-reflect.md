---
title: /flow:reflect
nav_order: 9
parent: Skills
---

# /flow:reflect

**Phase:** 7 — Reflect

**Usage:** `/flow:reflect`

Synthesises what went wrong from three sources, proposes CLAUDE.md
improvements, and notes plugin gaps. Runs before the PR merges.

---

## Sources

| Source | What | Survives compaction? |
|--------|------|---------------------|
| State file | Visit counts, timing, notes array | Yes |
| `/flow:note` captures | Corrections logged automatically | Yes |
| Conversation context | Session back-and-forth | Only if not compacted |

---

## Outputs

**CLAUDE.md** — committed to feature branch, merged with the feature:
- Generic reusable Rails patterns
- Each entry approved individually

**Plugin improvement notes** — presented only, never committed:
- Places where the FLOW process itself should improve

---

## Gates

- Phase 6: Review must be complete
- Only CLAUDE.md and `.claude/` files are committed — never application code
