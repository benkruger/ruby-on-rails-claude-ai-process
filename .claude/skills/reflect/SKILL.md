---
name: reflect
description: "Reflect on session mistakes. Reviews conversation against CLAUDE.md rules, proposes targeted improvements."
---

# Reflect

Review what went wrong in this session and propose CLAUDE.md improvements.

## Announce

Print:

```
============================================
  Reflect — STARTING
============================================
```

## Step 1 — Gather evidence

Read and synthesize from two sources:

### Source A — CLAUDE.md rules

Read this repo's `CLAUDE.md`. These are the rules that should have been
followed. Note every rule, convention, and lesson learned entry.

### Source B — Conversation context

Review the current conversation for:
- Moments where the user corrected Claude
- Responses where Claude was overruled or pushed back
- Misunderstandings that required clarification
- Suggestions Claude made that were rejected
- Places where Claude violated a CLAUDE.md rule

Note: context may have been compacted. Use what is available.

---

## Step 2 — Follow the reflection process

With the evidence gathered in Step 1, follow the shared reflection
process in `docs/reflection-process.md` (Steps A through E).

When Step E says to commit, use `/commit`.

---

## Done

Print:

```
============================================
  Reflect — COMPLETE
============================================
```

## Hard Rules

- Always read CLAUDE.md before presenting findings — never work from memory
- Always read the full conversation context before presenting findings
- Follow `docs/reflection-process.md` exactly — do not skip or reorder steps