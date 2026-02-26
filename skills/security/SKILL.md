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
  FLOW v0.8.5 — Phase 7: Security — STARTING
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

Run the command with exit code capture:

```bash
COMMAND; EC=$?; exit $EC
```

Then Read `.flow-states/<branch>.log` (empty string if it does not
exist yet) and Write it back with this line appended:

```text
YYYY-MM-DDTHH:MM:SSZ [Phase 7] Step X — desc (exit EC)
```

Get `<branch>` from the state file.

---

## Step 1 — Launch security analysis sub-agent

Read the following from the state file (small, structured — keep in main context):
- `state["design"]` — what was approved to be built
- `state["research"]["risks"]` — risks identified during Research

Then launch a mandatory sub-agent to analyze the feature diff for security
issues. Use the Task tool:

- `subagent_type`: `"Explore"`
- `description`: `"Security analysis"`

Provide these instructions to the sub-agent (fill in the details):

> You are analyzing a feature diff for security issues in a Ruby on Rails
> application.
> Feature: <feature name from state>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Only use Bash for git commands (git diff,
> git log, git blame). Never use Bash for file existence checks, directory
> listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Approved design:
> <paste state["design"] — chosen_approach, schema_changes, model_changes,
> controller_changes, worker_changes, route_changes>
>
> Research risks:
> <paste state["research"]["risks"]>
>
> First, get the full diff:
>
> ```bash
> git diff origin/main...HEAD
> ```
>
> Read every changed file completely. Then check:
>
> <!-- PLACEHOLDER: Security checks will be designed in a separate
> conversation. For now, the sub-agent should scan for obvious security
> issues in the diff:
>
> - SQL injection (raw SQL, string interpolation in queries)
> - Mass assignment (unpermitted params, open-ended permit)
> - Authentication/authorization gaps (missing before_action, skipped checks)
> - Sensitive data exposure (secrets in logs, PII in responses, credentials)
> - CSRF protection (skipped verify_authenticity_token)
> - Insecure direct object references (IDs from params without scoping)
> - Command injection (system(), exec(), backticks with user input)
> - Open redirects (redirect_to with user-controlled URLs)
> - Missing input validation at system boundaries
>
> This is a placeholder list. The actual checks, their severity levels,
> and the reporting format will be designed separately. -->
>
> Return structured findings in two categories:
>
> 1. Security issues found (with file:line references and severity)
> 2. Security considerations reviewed and found clean
>
> If no issues are found, say so explicitly.

Wait for the sub-agent to return before proceeding.

---

## Step 2 — Review sub-agent findings

Read the sub-agent's structured findings. For each reported issue:

- Confirm against the actual code (sub-agent may have false positives)
- Classify severity: **critical** (must fix), **moderate** (should fix),
  **low** (note for awareness)

Compile the confirmed findings list for Step 3.

---

## Step 3 — Fixing Findings

For each confirmed finding:

**Critical finding** (exploitable vulnerability):
- Use AskUserQuestion:
  > "Found a critical security issue: <description>. How would you like
  > to proceed?"
  >
  > - **Fix it here in Security**
  > - **Go back to Code**
  > - **Go back to Plan**

**Moderate finding** (defense-in-depth gap):
- Fix it directly
- Describe what was fixed and why

**Low finding** (awareness only):
- Report but do not fix unless the user asks

After fixing any findings, run `/flow:commit` for the Security fixes.

Then run `bin/ci` — required before any state transition.

<HARD-GATE>
bin/ci must be green before transitioning to Reflect.
Any fix made during Security requires bin/ci to run again.
</HARD-GATE>

---

## Step 4 — Present security summary

Show a summary of what was found and fixed inside a fenced code block:

````markdown
```text
============================================
  FLOW — Phase 7: Security — SUMMARY
============================================

  Issues found     : N
  Issues fixed     : N
  Issues noted     : N (low severity, awareness only)

  Findings
  --------
  - [CRITICAL] Fixed: SQL injection in PaymentController#create
  - [MODERATE] Fixed: Missing authorization check on admin endpoint
  - [LOW] Noted: Consider rate limiting on public API

  bin/ci           : ✓ green

============================================
```
````

---

## Back Navigation

Use AskUserQuestion if a finding is too significant to fix in Security:

> - **Go back to Code** — implementation needs rework
> - **Go back to Plan** — plan was missing security considerations
> - **Go back to Design** — approach needs rethinking
> - **Go back to Research** — something fundamental was missed

Update state for all phases between current and target before invoking
the target skill.

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
  FLOW v0.8.5 — Phase 7: Security — COMPLETE (<formatted_time>)
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
- Any `# rubocop:disable` comment in the diff is an automatic finding — remove it and fix the code
