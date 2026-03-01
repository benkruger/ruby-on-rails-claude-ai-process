---
name: security
description: "Phase 7: Security — scan for security issues in the feature diff. In-flow: diff-only after Review. Standalone: full repo, report-only, no state file required."
model: opus
---

# FLOW Security — Phase 7: Security

<HARD-GATE>
Run this phase entry check as your very first action. If any check fails,
stop immediately and show the error to the user.

1. Run both commands in parallel (two Bash calls in one response):
   - `git worktree list --porcelain` — note the path on the first `worktree` line (this is the project root).
   - `git branch --show-current` — this is the current branch.
2. Use the Read tool to read `<project_root>/.flow-states/<branch>.json`.
   - If the file does not exist: STOP. "BLOCKED: No FLOW feature in progress.
     Run /flow:start first."
3. Check `phases.6.status` in the JSON.
   - If not `"complete"`: STOP. "BLOCKED: Phase 6: Review must be
     complete. Run /flow:review first."
</HARD-GATE>

Keep the project root, branch, and state data from the gate in context —
all subsequent steps use them directly. Do not re-read the state file or
re-run git commands to gather the same information.

## Announce

At the very start, print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````markdown
```text
============================================
  FLOW v0.12.0 — Phase 7: Security — STARTING
============================================
```
````

## Update State

Using the state data from the gate, cd into the worktree and update Phase 7:
- `status` → `in_progress`
- `started_at` → current UTC timestamp (only if null — never overwrite)
- `session_started_at` → current UTC timestamp
- `visit_count` → increment by 1
- `current_phase` → `7`

## Logging

After every Bash command completes, log it to `.flow-states/<branch>.log`.

Run the command directly — do not append any suffix:

```bash
COMMAND
```

Then Read `.flow-states/<branch>.log` (empty string if it does not
exist yet) and Write it back with this line appended:

```text
YYYY-MM-DDTHH:MM:SSZ [Phase 7] Step X — desc (exit EC)
```

Get `<branch>` from the state file.

## Framework Fragment

Read the framework-specific instructions from
`${CLAUDE_PLUGIN_ROOT}/skills/security/<framework>.md`
where `<framework>` is the `framework` field from the state file
(`.flow-states/<branch>.json`).

The fragment provides the security analysis sub-agent prompt referenced below.

---

## Step 1 — Launch security analysis sub-agent

Read the following from the state file (small, structured — keep in main context):
- `state["design"]` — what was approved to be built
- `state["research"]["risks"]` — risks identified during Research

Then launch a mandatory sub-agent to analyze the feature diff for security
issues. Use the Task tool:

- `subagent_type`: `"Explore"`
- `description`: `"Security analysis"`

Provide the sub-agent with the **Security Analysis Sub-Agent Prompt** from the
framework fragment (fill in the feature name, design, and risks).

Wait for the sub-agent to return before proceeding.

---

## Step 2 — Confirm findings and record in state

Read the sub-agent's findings. For each reported issue:

1. Read the cited file and line to confirm the issue exists (sub-agents may
   have false positives)
2. Drop any finding that is a false positive — explain why it was dropped

Write all confirmed findings and clean checks to the state file:

```json
"security": {
  "findings": [
    {
      "id": 1,
      "check": "authorization_gaps",
      "description": "PaymentController#show has no before_action auth check",
      "file": "app/controllers/payment_controller.rb",
      "line": 15,
      "status": "pending"
    }
  ],
  "clean_checks": ["sql_injection", "csrf_bypass", "open_redirects"],
  "scanned_at": "2026-02-20T15:00:00Z"
}
```

Number each finding with a sequential `id`. Set `status` to `"pending"` for
every confirmed finding. `scanned_at` is the current UTC timestamp.

If there are no confirmed findings, set `findings` to an empty array, list
all 10 checks in `clean_checks`, and skip to Step 4.

---

## Step 3 — Fix findings

Fix each confirmed finding one at a time, in order:

1. Fix the issue in code
2. Run `bin/ci`
3. Invoke `/flow:commit` for the fix
4. Update the finding's `status` to `"fixed"` in the state file
5. Move to the next finding

<HARD-GATE>
bin/ci must be green after every fix. Do not move to the next finding
until the current fix passes bin/ci and is committed.
</HARD-GATE>

Repeat until all findings have `status: "fixed"`.

---

## Step 4 — Present security summary

Show a summary of what was found and fixed inside a fenced code block:

````markdown
```text
============================================
  FLOW — Phase 7: Security — SUMMARY
============================================

  Checks run       : 10
  Findings         : N
  Fixed            : N
  Clean checks     : N

  Findings
  --------
  - [FIXED] authorization_gaps: PaymentController#show has no auth check
  - [FIXED] rubocop_disables: # rubocop:disable in payment_controller.rb

  Clean Checks
  ------------
  sql_injection, csrf_bypass, open_redirects, ...

  bin/ci           : ✓ green

============================================
```
````

---

## Done — Update state and complete phase

Update Phase 7 in state:
1. `cumulative_seconds` += `current_time - session_started_at`. Do not print the calculation.
2. `status` → `complete`
3. `completed_at` → current UTC timestamp
4. `session_started_at` → `null`
5. `current_phase` → `8`

Format `cumulative_seconds` as `<formatted_time>`: `Xh Ym` if ≥ 3600, `Xm` if ≥ 60, `<1m` if < 60.

Print inside a fenced code block:

````markdown
```text
============================================
  FLOW v0.12.0 — Phase 7: Security — COMPLETE (<formatted_time>)
============================================
```
````

Invoke `flow:status`, then use AskUserQuestion:

> "Phase 7: Security is complete. Ready to begin Phase 8: Reflect?"
>
> - **Yes, start Phase 8 now** — invoke `flow:reflect`
> - **Not yet** — print paused banner
> - **I have a correction or learning to capture**

**If "I have a correction or learning to capture":**
1. Ask the user what they want to capture
2. Invoke `/flow:note` with their message
3. Re-ask with only "Yes, start Phase 8 now" and "Not yet"

**If Yes** — invoke `flow:reflect` using the Skill tool.

**If Not yet**, print inside a fenced code block:

````markdown
```text
============================================
  FLOW — Paused
  Run /flow:continue when ready to continue.
============================================
```
````

---

## Hard Rules

- Always run `bin/ci` after any fix made during Security
- Never transition to Reflect unless bin/ci is green
- Read the full diff before starting — no partial reviews
