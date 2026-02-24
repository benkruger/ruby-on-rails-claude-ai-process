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

With the evidence gathered in Step 1, follow the reflection process below.

When Step E says to commit, use `/commit`.

### Step A — Synthesize findings

Before asking the user anything, organize all gathered evidence into four
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

### Step B — Present findings

Present the synthesis to the user in a banner:

````
```
============================================
  Reflect — Findings
============================================

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

============================================
```
````

Then use AskUserQuestion:

> "Does this capture what went wrong? Anything I missed or got wrong?"
> - **Yes, this is accurate** — proceed to proposals
> - **Needs corrections** — describe what to change

If "Needs corrections", revise and re-present until accurate.

### Step C — Propose CLAUDE.md additions

For each item in "Missing rules", propose a specific addition to CLAUDE.md.

**Writing rules for CLAUDE.md:**
- Write for Claude, not for humans — the audience is a future Claude session
- Be direct, specific, and actionable — describe the exact situation and the
  exact required behavior
- One to three sentences maximum
- Generic and reusable — not tied to the specific feature or session
- Placed in the correct section of the target CLAUDE.md

Present each proposal individually using AskUserQuestion:

> "Proposed CLAUDE.md addition:
> '[proposed text]'
> Section: [target section]"
> - **Yes, add it**
> - **Yes, but rephrase** — describe how
> - **No, skip this one**

For "Yes, but rephrase" — revise and confirm before collecting.

Collect all approved additions. Do not apply yet.

### Step D — Strengthen violated rules

For each item in "Process violations", evaluate whether the existing rule's
language was clear enough. If the violation happened because the rule was
ambiguous or easy to overlook, propose a rewording.

Present each rewording proposal individually using AskUserQuestion (same
three options as Step C).

Collect all approved rewordings. Do not apply yet.

### Step E — Apply approved changes

Read the target CLAUDE.md. Apply all approved additions and rewordings.
Do not duplicate existing content.

Then commit via `/commit`.

Only CLAUDE.md and `.claude/` files are committed — never application code.

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
- Follow the reflection process (Steps A through E) exactly — do not skip or reorder steps