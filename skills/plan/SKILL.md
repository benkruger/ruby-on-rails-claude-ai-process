---
name: plan
description: "Phase 4: Plan — break the approved design into ordered, executable tasks section by section. Each section is approved individually. Supports back navigation within the plan and to Design or Research."
model: sonnet
---

# FLOW Plan — Phase 4: Plan

<HARD-GATE>
Run this phase entry check as your very first action. If any check fails,
stop immediately and show the error to the user.

1. Run both commands in parallel (two Bash calls in one response):
   - `git worktree list --porcelain` — note the path on the first `worktree` line (this is the project root).
   - `git branch --show-current` — this is the current branch.
2. Use the Read tool to read `<project_root>/.flow-states/<branch>.json`.
   - If the file does not exist: STOP. "BLOCKED: No FLOW feature in progress.
     Run /flow:start first."
3. Check `phases.3.status` in the JSON.
   - If not `"complete"`: STOP. "BLOCKED: Phase 3: Design must be
     complete. Run /flow:design first."
</HARD-GATE>

Keep the project root, branch, and state data from the gate in context —
all subsequent steps use them directly. Do not re-read the state file or
re-run git commands to gather the same information.

## Announce

At the very start, print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````text
```
============================================
  FLOW v0.12.0 — Phase 4: Plan — STARTING
============================================
```
````

## Update State

Using the state data from the gate, cd into the worktree and update Phase 4:
- `status` → `in_progress`
- `started_at` → current UTC timestamp (only if null — never overwrite)
- `session_started_at` → current UTC timestamp
- `visit_count` → increment by 1
- `current_phase` → `4`

## Framework Fragment

Read the framework-specific instructions from
`${CLAUDE_PLUGIN_ROOT}/skills/plan/<framework>.md`
where `<framework>` is the `framework` field from the state file
(`.flow-states/<branch>.json`).

The fragment provides plan section initialization, section verification
sub-agent prompt, section definitions, and plan save schema referenced below.

Initialise `state["plan"]` using the **Plan Sections Initialization** from
the framework fragment.

## Logging

No logging for this phase. Plan runs no Bash commands beyond the entry
gate — the work is AskUserQuestion calls, sub-agent verifications, and
state file writes.

---

## Resuming Mid-Plan

If `state["plan"]["current_section"]` is already set, this is a resume.

Show what is already approved. Print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````text
```
============================================
  FLOW — Plan in progress
============================================

  Approved sections:
  [x] Schema
  [x] Models

  Resuming at: Workers

============================================
```
````

Then use AskUserQuestion:

> "Ready to continue from the Workers section?"
>
> - **Yes, continue** — jump to that section
> - **Go back to an approved section** — show picker of approved sections

---

## Step 1 — Quick check-in

Use AskUserQuestion:

> "Ready to plan? Anything to add before we start?"
>
> - **Ready — generate tasks from the design**
> - **I want to add something first** — describe it in Other

If the user adds something, note it and incorporate it into the plan.

---

## Step 2 — Review the design

Review the design data already in context from the gate:
- `feature_description`
- `schema_changes`
- `model_changes`
- `controller_changes`
- `worker_changes`
- `route_changes`
- `risks`

Skip sections with no changes (e.g., if `worker_changes` is empty,
skip the workers section and note it was skipped).

---

## Section Structure

Work through each section below in order. For each section:

1. Generate tasks for that section
2. Verify tasks via mandatory sub-agent
3. Adjust tasks based on sub-agent findings
4. Present them clearly
5. Ask the section approval question
6. Handle the response

### Section Verification via Sub-Agent

After generating tasks for a section, launch a mandatory sub-agent to verify
them. Use the Task tool:

- `subagent_type`: `"Explore"`
- `description`: `"Plan task verification — <section name>"`

Provide the sub-agent with the **Section Verification Sub-Agent Prompt**
from the framework fragment (fill in the feature name, section, design
decisions, research findings, and tasks).

Adjust tasks based on the sub-agent's findings before presenting the
section to the user.

### Section Approval Question

At the end of every section, use AskUserQuestion:

> "Does the [Section Name] plan look right?"
>
> - **Yes, looks good** — mark section approved, move to next
> - **Needs changes** — describe in Other, revise and re-present
> - **Go back to [previous section]** — only shown if one section back
> - **Go back further** — only shown if two or more sections back

**"Go back further"** triggers a second AskUserQuestion listing all
approved sections as options. User picks one. Mark that section and
all sections after it as `pending`, re-open the chosen section.

When going back: explain clearly which sections were invalidated and
why (because earlier decisions affect later ones).

---

Follow the **Section Definitions** from the framework fragment. Work through
each section in order. Skip sections with no corresponding design changes.

---

## Step 3 — Final Full Plan Review

Once all sections are approved, show the complete ordered task list. Print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````text
```
============================================
  FLOW — Phase 4: Plan — FULL PLAN
============================================

  Feature: <feature>

  [ ] Task 1  — Schema:  Add payments table to data/release.sql
  [ ] Task 2  — Test:    Failing test for Payment::Base
  [ ] Task 3  — Model:   Implement Payment::Base
  [ ] Task 4  — Test:    Failing test for Payment::Create
  [ ] Task 5  — Model:   Implement Payment::Create
  [ ] Task 6  — Test:    Failing test for PaymentWebhookWorker
  [ ] Task 7  — Worker:  Implement PaymentWebhookWorker
  [ ] Task 8  — Route:   Add POST /api/webhooks/payment
  [ ] Task 9  — Test:    Controller test for webhooks#payment
  [ ] Task 10 — Impl:    Implement WebhooksController#payment
  [ ] Task 11 — Test:    Integration test for full webhook flow
  [ ] Task 12 — CI:      bin/ci green

============================================
```
````

Then use AskUserQuestion:

> "Does the full plan look right?"
>
> - **Approve — ready to code**
> - **Needs changes** — describe which tasks to revise
> - **Go back to a plan section** — show section picker
> - **Go back to Design** — approach needs rethinking
> - **Go back to Research** — something was missed

**"Go back to Design"** — update Phase 4 to `pending`, Phase 3 to
`in_progress`, then invoke `flow:design`.

**"Go back to Research"** — update Phase 4 to `pending`, Phase 3 to
`pending`, Phase 2 to `in_progress`, then invoke `flow:research`.

---

## Step 4 — Save plan to state

Write the plan to `.flow-states/<branch>.json` under `plan` using the
**Plan Save Schema** from the framework fragment.

---

## Done — Update state and complete phase

Update Phase 4 in state:
1. `cumulative_seconds` += `current_time - session_started_at`. Do not print the calculation.
2. `status` → `complete`
3. `completed_at` → current UTC timestamp
4. `session_started_at` → `null`
5. `current_phase` → `5`

Format `cumulative_seconds` as `<formatted_time>`: `Xh Ym` if ≥ 3600, `Xm` if ≥ 60, `<1m` if < 60.

Print inside a fenced code block:

````text
```
============================================
  FLOW v0.12.0 — Phase 4: Plan — COMPLETE (<formatted_time>)
============================================
```
````

Invoke `flow:status`, then use AskUserQuestion:

> "Phase 4: Plan is complete. Ready to begin Phase 5: Code?"
>
> - **Yes, start Phase 5 now** — invoke `flow:code`
> - **Not yet** — print paused banner
> - **I have a correction or learning to capture**

**If "I have a correction or learning to capture":**
1. Ask the user what they want to capture
2. Invoke `/flow:note` with their message
3. Re-ask with only "Yes, start Phase 5 now" and "Not yet"

**If Yes** — invoke `flow:code` using the Skill tool.

**If Not yet**, print inside a fenced code block:

````text
```
============================================
  FLOW — Paused
  Run /flow:continue when ready to continue.
============================================
```
````

---

## Hard Rules

- Always TDD order — test task before every implementation task
- Always check `test/support/` for existing helpers before creating new ones
- Never skip sections silently — always note when a section is skipped and why
- When going back invalidates sections, explain clearly which sections need re-approval
- Never write implementation code during Plan — task descriptions only
