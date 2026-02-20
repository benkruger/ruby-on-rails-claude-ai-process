---
name: review
description: "Phase 6: Review — systematic code review against design, research risks, and Rails anti-patterns. Fixes issues found, runs bin/ci after any fix, then transitions to Reflect."
---

# FLOW Review — Phase 6: Review

<HARD-GATE>
Run this phase entry check as your very first action. If it exits
non-zero, stop immediately and show the error to the user.

```bash
python3 << 'PYCHECK'
import json, subprocess, sys
from pathlib import Path

def project_root():
    r = subprocess.run(['git', 'worktree', 'list', '--porcelain'],
                       capture_output=True, text=True)
    for line in r.stdout.split('\n'):
        if line.startswith('worktree '):
            return Path(line.split(' ', 1)[1].strip())
    return Path('.')

branch = subprocess.run(['git', 'branch', '--show-current'],
                        capture_output=True, text=True).stdout.strip()
state_file = project_root() / '.claude' / 'flow-states' / f'{branch}.json'

if not state_file.exists():
    print('BLOCKED: No FLOW feature in progress. Run /flow:start first.')
    sys.exit(1)

state = json.loads(state_file.read_text())
prev = state.get('phases', {}).get('5', {})
if prev.get('status') != 'complete':
    print('BLOCKED: Phase 5: Code must be complete before Review.')
    print('Run /flow:code first.')
    sys.exit(1)
PYCHECK
```
</HARD-GATE>

## Announce

Print:

```
============================================
  FLOW — Phase 6: Review — STARTING
============================================
```

## Update State

Read `.claude/flow-states/<branch>.json`. cd into the worktree.

Update Phase 6:
- `status` → `in_progress`
- `started_at` → current UTC timestamp (only if null — never overwrite)
- `session_started_at` → current UTC timestamp
- `visit_count` → increment by 1
- `current_phase` → `6`

---

## Step 1 — Load context

Read the following from the state file:
- `state["design"]` — what was approved to be built
- `state["plan"]["tasks"]` — what was planned
- `state["research"]["risks"]` — risks identified during Research

Get the full feature diff:

```bash
git diff origin/main...HEAD
```

This shows every change made in this feature branch. Read it completely
before starting the review.

---

## Step 2 — Design Alignment Check

Compare the implementation against `state["design"]`:

- Do the schema changes match `design["schema_changes"]`?
- Do the model decisions match `design["model_changes"]`?
- Do the controller/route changes match `design["controller_changes"]` and `design["route_changes"]`?
- Do the worker changes match `design["worker_changes"]`?

For each mismatch — flag it. Minor drift is a finding. Major drift means
go back to Code.

---

## Step 3 — Research Risk Coverage

Read `state["research"]["risks"]` one by one.

For each risk, confirm it was properly handled in the implementation.
A risk identified in Research and not addressed is a bug waiting to happen.

Flag any risk that was not accounted for.

---

## Step 4 — Rails Anti-Pattern Review

Read every changed file as if seeing it for the first time. Check each
of the following explicitly — do not skip any:

**Associations:**
- Every `belongs_to` has `inverse_of:`
- Every `has_many` has `dependent:` specified
- Every `has_many` has `inverse_of:`
- `class_name:` is explicit on all associations

**Queries:**
- No N+1 queries — check controllers, workers, and mailers
- No database queries in views
- `.where` not used when a named scope or association would be cleaner
- No `.first` or `.last` to pick a "default" record — if the choice matters, find it by a meaningful attribute

**Callbacks:**
- Callbacks in parent classes that set values from `Current` — confirm the implementation uses `Current` correctly, not direct parameter passing
- No `update_column` anywhere — only `update!`

**Models:**
- `self.table_name =` set explicitly in all namespaced Base classes
- No STI (`self.inheritance_column = :_type_disabled` if needed)

**Soft deletes:**
- `.unscoped` used correctly where deleted records are intentionally accessed
- Queries that should not include soft-deleted records do not use `.unscoped`

**Workers:**
- `halt!` called in `pre_perform!` for missing/invalid records
- Queue name matches `config/sidekiq.yml`

**Tests:**
- `create_*!` helpers used — no `Model::Create.create!` directly
- Both branches of every conditional tested
- Tests have assertions — no empty test methods
- No inline comments in tests

**Code clarity:**
- Descriptive variable names — no `bu`, `data`, `values`
- No `owner` as a variable name — use `parent` or something descriptive
- No inline comments — code should be self-documenting
- No over-engineering beyond what the task required

---

## Step 5 — Fixing Findings

For each finding:

**Minor finding** (style, missing option, small oversight):
- Fix it directly
- Describe what was fixed and why

**Significant finding** (logic error, missing risk coverage, design mismatch):
- Use AskUserQuestion:
  > "Found a significant issue: <description>. How would you like to proceed?"
  > - **Fix it here in Review**
  > - **Go back to Code**
  > - **Go back to Plan**

After fixing any findings, run `/flow:commit` for the Review fixes.

Then run `bin/ci` — required before any state transition.

<HARD-GATE>
bin/ci must be green before transitioning to Reflect.
Any fix made during Review requires bin/ci to run again.
</HARD-GATE>

---

## Step 6 — Present review summary

Show a summary of what was found and fixed:

```
============================================
  FLOW — Phase 6: Review — SUMMARY
============================================

  Design alignment  : ✓ matches approved design
  Research risks    : ✓ all risks accounted for

  Findings fixed
  --------------
  - Added inverse_of: to Payment::Base associations
  - Removed N+1 query in PaymentWebhookWorker#perform!
  - Added dependent: :destroy to Account has_many :payments

  bin/ci            : ✓ green

============================================
```

---

## Back Navigation

Use AskUserQuestion if a finding is too significant to fix in Review:

> - **Go back to Code** — implementation issue
> - **Go back to Plan** — plan was missing something
> - **Go back to Design** — approach needs rethinking
> - **Go back to Research** — something fundamental was missed

Update state for all phases between current and target before invoking
the target skill.

---

## Done — Update state and complete phase

Update Phase 6 in state:
1. `cumulative_seconds` += `current_time - session_started_at`
2. `status` → `complete`
3. `completed_at` → current UTC timestamp
4. `session_started_at` → `null`
5. `current_phase` → `7`

Invoke `flow:status`, then use AskUserQuestion:

> "Phase 6: Review is complete. Ready to begin Phase 7: Reflect?"
> - **Yes, start Phase 7 now** — invoke `flow:reflect`
> - **Not yet** — print paused banner

**If Yes**, print:

```
============================================
  FLOW — Phase 6: Review — COMPLETE
============================================
```

**If Not yet**, print:

```
============================================
  FLOW — Paused
  Run /flow:resume when ready to continue.
============================================
```

---

## Hard Rules

- Always run `bin/ci` after any fix made during Review
- Never transition to Reflect unless bin/ci is green
- Never skip the design alignment check
- Never skip the research risk coverage check
- Read the full diff before starting — no partial reviews
