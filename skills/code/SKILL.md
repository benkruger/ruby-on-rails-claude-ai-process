---
name: code
description: "Phase 5: Code — execute plan tasks one at a time with TDD. Review diff before each commit. bin/ci must pass before moving to the next task. Framework architecture standards enforced."
model: opus
---

# FLOW Code — Phase 5: Code

<HARD-GATE>
Run this phase entry check as your very first action. If any check fails,
stop immediately and show the error to the user.

1. Run both commands in parallel (two Bash calls in one response):
   - `git worktree list --porcelain` — note the path on the first `worktree` line (this is the project root).
   - `git branch --show-current` — this is the current branch.
2. Use the Read tool to read `<project_root>/.flow-states/<branch>.json`.
   - If the file does not exist: STOP. "BLOCKED: No FLOW feature in progress.
     Run /flow:start first."
3. Check `phases.4.status` in the JSON.
   - If not `"complete"`: STOP. "BLOCKED: Phase 4: Plan must be
     complete. Run /flow:plan first."
</HARD-GATE>

Keep the project root, branch, and state data from the gate in context —
all subsequent steps use them directly. Do not re-read the state file or
re-run git commands to gather the same information.

## Announce

At the very start, print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````text
```
============================================
  FLOW v0.13.1 — Phase 5: Code — STARTING
============================================
```
````

## Update State

Using the state data from the gate, cd into the worktree and update Phase 5:
- `status` → `in_progress`
- `started_at` → current UTC timestamp (only if null — never overwrite)
- `session_started_at` → current UTC timestamp
- `visit_count` → increment by 1
- `current_phase` → `5`

## Framework Fragment

Read the framework-specific instructions from
`${CLAUDE_PLUGIN_ROOT}/skills/code/<framework>.md`
where `<framework>` is the `framework` field from the state file
(`.flow-states/<branch>.json`).

The fragment provides architecture checks, the targeted test command,
CI failure fix order, and framework-specific hard rules referenced below.

## Logging

After every Bash command completes, log it to `.flow-states/<branch>.log`.

Run the command directly — do not append any suffix:

```bash
COMMAND
```

Then Read `.flow-states/<branch>.log` (empty string if it does not
exist yet) and Write it back with this line appended:

```text
YYYY-MM-DDTHH:MM:SSZ [Phase 5] Step X — desc (exit EC)
```

Get `<branch>` from the state file.

---

## Resuming Mid-Code

If any tasks in `state["plan"]["tasks"]` have `status: "in_progress"`,
this is a resume. Print inside a fenced code block:

````text
```
============================================
  FLOW — Resuming Code
============================================
  Resuming at Task <id>: <description>
  Tasks complete: <n> of <total>
============================================
```
````

Continue from the first task with `status: "in_progress"` or `"pending"`.

---

## Task Loop

Work through `state["plan"]["tasks"]` in order. For each task:

### Before Starting a Task

Update the task in state: `status → in_progress`, `started_at → now`.

Print inside a fenced code block:

````text
```
============================================
  Task <id> of <total> — <type>
  <description>
  Files: <files>
============================================
```
````

### Architecture Check

Follow the **Architecture Check** from the framework fragment. Check based
on task type as described there before writing any code.

---

### TDD Cycle

**For every implementation task, there is a paired test task that runs first.**

**Step A — Write the failing test**

Write the test file. Follow the test task description exactly.

Run the **Targeted Test Command** from the framework fragment to confirm
it fails.

The test MUST fail before proceeding. If it passes immediately, the test
is not testing the right thing — rewrite it until it fails for the right reason.

**Step B — Write minimal implementation**

Write only what is needed to make the test pass. No over-engineering.

Run the **Targeted Test Command** again to confirm it passes.

**Step C — Refactor**

Clean up without changing behaviour. Run the test again to confirm it
still passes.

---

### Review

After the TDD cycle passes, show the diff for this task and ask for
review before touching `bin/ci` or committing.

Run `git status` and `git diff HEAD` as two separate commands, then
render the output inline:

**Status**

```text
modified:   <path/to/implementation_file>
new file:   <path/to/test_file>
```

**Diff**

```diff
+ added lines
- removed lines
```

Then use AskUserQuestion:

> "Task <id>: <description> — does this look right?"
>
> - **Yes, run bin/ci and commit**
> - **Needs changes** — describe what to fix

**If "Needs changes"** — fix the issue, re-run the test, show the diff
again. Loop until approved.

---

### bin/ci Gate

Run `bin/ci`. This must be green before committing.

**If bin/ci fails:**

- Read the output carefully
- Fix each failure following the **CI Failure Fix Order** from the framework fragment
- Re-run `bin/ci` after each fix
- Max 3 attempts — if still failing after 3, stop and report exactly what is failing

<HARD-GATE>
Do NOT commit and do NOT move to the next task until bin/ci is green.
</HARD-GATE>

---

### Commit

Use `/flow:commit` to review and commit this task's changes.

The commit message subject should reference the task:

```text
Add <what was built> — Task <id> of <total>
```

---

### Complete the Task

Update the task in state:
- `status → complete`
- `completed_at → now`

Print inside a fenced code block:

````text
```
Task <id> complete. <n> of <total> done.
```
````

Then move to the next task. Loop.

---

## Back Navigation

At any point during the task loop, if something fundamental is wrong:

Use AskUserQuestion:
> - **Go back to Plan** — task description is wrong or missing tasks
> - **Go back to Design** — the approach itself needs rethinking
> - **Go back to Research** — something was missed that changes everything

**Go back to Plan:** update Phase 5 to `pending`, Phase 4 to
`in_progress`, then invoke `flow:plan`.

**Go back to Design:** update Phases 5 and 4 to `pending`, Phase 3 to
`in_progress`, then invoke `flow:design`.

**Go back to Research:** update Phases 5, 4, and 3 to `pending`, Phase 2 to
`in_progress`, then invoke `flow:research`.

---

## All Tasks Complete

Once every task in `state["plan"]["tasks"]` is `complete`:

**Final bin/ci sweep:**

```bash
bin/ci
```

Then check coverage:

```bash
cat coverage/uncovered.txt
```

If there are uncovered lines:
- Write tests for each uncovered line
- Run `bin/ci` again
- Repeat until `coverage/uncovered.txt` is empty

<HARD-GATE>
Do NOT transition to Review until bin/ci is green AND coverage/uncovered.txt
is empty. 100% coverage is mandatory.
</HARD-GATE>

## Done — Update state and complete phase

Update Phase 5 in state:
1. `cumulative_seconds` += `current_time - session_started_at`. Do not print the calculation.
2. `status` → `complete`
3. `completed_at` → current UTC timestamp
4. `session_started_at` → `null`
5. `current_phase` → `6`

Format `cumulative_seconds` as `<formatted_time>`: `Xh Ym` if ≥ 3600, `Xm` if ≥ 60, `<1m` if < 60.

Print inside a fenced code block:

````text
```
============================================
  FLOW v0.13.1 — Phase 5: Code — COMPLETE (<formatted_time>)
============================================
```
````

Invoke `flow:status`, then use AskUserQuestion:

> "Phase 5: Code is complete. Ready to begin Phase 6: Review?"
>
> - **Yes, start Phase 6 now** — invoke `flow:review`
> - **Not yet** — print paused banner
> - **I have a correction or learning to capture**

**If "I have a correction or learning to capture":**
1. Ask the user what they want to capture
2. Invoke `/flow:note` with their message
3. Re-ask with only "Yes, start Phase 6 now" and "Not yet"

**If Yes** — invoke `flow:review` using the Skill tool.

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

- **Never skip the TDD cycle** — test must fail before code is written
- **Never skip the review** — user approves every task before bin/ci runs
- **Never skip bin/ci** — must be green before every commit
- **Never move to the next task** until the current task is committed
- **Never rebase** — always merge
- Plus the **Framework-Specific Hard Rules** from the framework fragment
