---
name: reflect
description: "Reflect on session mistakes. Reviews conversation against CLAUDE.md rules, proposes targeted improvements."
---

# Reflect

Review what went wrong in this session and apply CLAUDE.md improvements.

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

With the evidence gathered in Step 1, follow the reflection process below.

### Step A — Synthesize findings

Before doing anything else, organize all gathered evidence into four
categories:

**Process violations** — existing rules in CLAUDE.md that were broken or
nearly broken during the session. Quote the specific rule.

**Claude mistakes** — things Claude got wrong that the user had to correct.
Be specific and honest. Name the mistake clearly — do not soften or hedge.

For each mistake, state:
1. What Claude did wrong (the actual behavior, not a euphemism)
2. What the user said or did to correct it (quote or paraphrase)
3. How many rounds of correction it took before Claude got it right

If you cannot answer all three, you are probably softening the mistake.

**Missing rules** — situations where Claude did the wrong thing but no
existing rule covered it. These are gaps in CLAUDE.md.

**Process gaps** — places where the development process itself (tools,
skills, workflows) should be improved. These are not CLAUDE.md rules —
they are process changes.

### Step B — Route and apply

This step is fully autonomous — decide destinations and apply all changes
without asking the user.

**Writing rules for CLAUDE.md:**
- Write for Claude, not for humans — the audience is a future Claude session
- Be direct, specific, and actionable — describe the exact situation and the
  exact required behavior
- One to three sentences maximum
- Generic and reusable — not tied to the specific feature or session
- Placed in the correct section of the target CLAUDE.md

For each item in "Missing rules":
1. Compose a specific CLAUDE.md addition following the writing rules above
2. Read the target file, apply the addition. Do not duplicate existing content.

For each item in "Process violations":
1. Evaluate whether the existing rule's language was clear enough
2. If the violation happened because the rule was ambiguous or easy to
   overlook, reword the rule
3. Read the target file, apply the rewording. Do not duplicate existing content.

Only CLAUDE.md and `.claude/` files are modified — never application code.

### Step C — Commit

Commit all changes via `/commit --auto`.

### Step D — Present report

Present the full report to the user:

````
```
============================================
  Reflect — Report
============================================

  Findings
  --------

  Process violations
  ------------------
  - CLAUDE.md says "never use guard clauses" but Claude
    added an early return in the worker
  - ...

  Claude mistakes
  ---------------
  - Suggested git rebase (forbidden — corrected immediately)
  - ...

  Missing rules
  -------------
  - No rule about checking eager-loaded associations
    before using pluck
  - ...

  Process gaps
  ------------
  - /flow:commit should warn when branch is behind
  - ...

  Changes applied
  ---------------
  CLAUDE.md: 2 additions, 1 rewording

============================================
```
````

Omit "Changes applied" if no changes were made.

---

## Done

Print:

```
============================================
  Reflect — COMPLETE
============================================
```

## Hard Rules

- Always read CLAUDE.md before synthesizing findings — never work from memory
- Always read the full conversation context before synthesizing findings
- Follow the reflection process (Steps A through D) exactly — do not skip or reorder steps
- Decisions on wording are autonomous — do not ask the user for approval mid-process
- The report in Step D is the user's review point — make it comprehensive
