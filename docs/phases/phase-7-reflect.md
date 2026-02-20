---
title: "Phase 7: Reflect"
nav_order: 9
---

# Phase 7: Reflect

**Command:** `/flow:reflect`

Runs before the PR is merged. Reviews what went wrong across all phases,
proposes CLAUDE.md improvements, and notes plugin gaps. The only commits
are CLAUDE.md and `.claude/` changes — application code is never touched.

---

## Three Sources

Reflect synthesises from all three before asking the user anything:

1. **State file data** — visit counts, timing, captured `/flow:note` entries, research risks, open questions
2. **Captured notes** — corrections logged automatically by `/flow:note` throughout the session
3. **Conversation context** — what Claude can still see of the session's back-and-forth

Sources 1 and 2 survive compaction. Context is a bonus if available.

---

## What Gets Captured

**CLAUDE.md additions** — committed to the feature branch:
- Generic, reusable Rails patterns discovered during this feature
- Each approved individually before being written

**Plugin improvement notes** — presented only, never committed:
- Places where the FLOW process itself should improve
- User decides whether to open issues on the plugin repo

---

## What Makes a Good CLAUDE.md Entry

**Good:** Generic pattern that prevents the same mistake in any future feature
> "Never assume branch-behind is unlikely in a multi-session workflow"

**Bad:** Feature-specific note that only applies here
> "Payment::Base uses the critical Sidekiq queue"

---

## What Comes Next

Merge the PR manually (which now includes CLAUDE.md improvements),
then run Phase 8: Cleanup (`/flow:cleanup`).
