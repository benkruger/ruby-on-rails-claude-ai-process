---
title: /flow:reflect
nav_order: 9
parent: Skills
---

# /flow:reflect

**Phase:** 8 — Reflect

**Usage:** `/flow:reflect`

Synthesises what went wrong from four sources, routes each learning to
its correct permanent home, and notes plugin gaps. Runs before the PR
merges.

---

## Sources

| Source | What | Survives compaction? |
|--------|------|---------------------|
| State file | Visit counts, timing, notes array | Yes |
| `/flow:note` captures | Corrections logged automatically | Yes |
| Conversation context | Session back-and-forth | Only if not compacted |
| Worktree auto-memory | Patterns and observations from feature work | Yes |

---

## Outputs

Approved learnings are routed to one of 5 destinations:

| # | Destination | Path |
|---|-------------|------|
| 1 | Global CLAUDE.md | `~/.claude/CLAUDE.md` |
| 2 | Project CLAUDE.md | `CLAUDE.md` in worktree |
| 3 | Global rules | `~/.claude/rules/<topic>.md` |
| 4 | Project rules | `.claude/rules/<topic>.md` in worktree |
| 5 | Project memory | `~/.claude/projects/<repo-root>/memory/MEMORY.md` |

Destinations 1, 3, 5 are user-private (direct edits, not committed).
Destinations 2, 4 are committed to the feature branch via `/flow:commit`.

**Plugin improvement notes** — presented only, never committed:

- Places where the FLOW process itself should improve

---

## Gates

- Phase 7: Security must be complete
- Only CLAUDE.md and `.claude/` files are committed — never application code
