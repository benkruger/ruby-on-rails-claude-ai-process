---
name: abort
description: "Abort the current FLOW feature. Closes the PR, deletes the remote branch, removes the worktree, and deletes the state file. Available from any phase."
---

# FLOW Abort

Abandon the current feature completely. This is the escape hatch — available
from any phase, no prerequisites.

## Entry Check

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
    print(f'WARNING: No state file found for branch "{branch}".')
    print('Will attempt best-effort cleanup using git state.')
    sys.exit(0)

state = json.loads(state_file.read_text())
print(f"Feature: {state.get('feature', 'unknown')}")
print(f"Branch: {state.get('branch', 'unknown')}")
print(f"PR: {state.get('pr_url', 'none')}")
print(f"Current phase: {state.get('current_phase', '?')}")
PYCHECK
```

If this exits non-zero, stop and show the message.

## Announce

Print:

```
============================================
  FLOW — Abort — STARTING
============================================
```

## Steps

### Step 1 — Read state

If the state file exists, read `.claude/flow-states/<branch>.json` from
the project root. Note `feature`, `branch`, `worktree`, `pr_number`,
and `pr_url`.

If the state file is missing, infer what you can:
- `branch` from `git branch --show-current`
- Detect worktree path from `git worktree list`
- Use the branch name as the feature name
- `pr_number` unknown — skip PR close step

Tell the user what was inferred.

### Step 2 — Confirm with user

This is destructive and irreversible. Use AskUserQuestion.

If the entry check printed warnings, include them in the confirmation:

> "Abort feature '<feature>'?
> ⚠ <any warnings from the entry check>
> This will close the PR, delete the remote branch, remove the worktree, and delete the state file. All uncommitted work in the worktree will be lost."
> - **Yes, abort everything** — proceed
> - **No, keep going** — stop here

### Step 3 — Navigate to project root

Follow `docs/cleanup-process.md` Step 1.

### Step 4 — Close the PR

If `pr_number` exists (from state or inferred):

```bash
gh pr close <pr_number> --comment "Aborted via /flow:abort"
```

If this fails (PR already closed/merged) or `pr_number` is unknown,
note it and continue — do not stop.

### Step 5 — Remove the worktree

Follow `docs/cleanup-process.md` Step 2.

### Step 6 — Delete the remote branch

```bash
git push origin --delete <branch-name>
```

If this fails (branch already deleted), note it and continue.

### Step 7 — Delete the local branch

From the project root (which is on main):

```bash
git branch -D <branch-name>
```

If this fails (branch already deleted), note it and continue.

### Step 8 — Delete the state file

Follow `docs/cleanup-process.md` Step 3.

### Done

Follow `docs/cleanup-process.md` Step 4 (report results), then print:

```
============================================
  FLOW — Abort — COMPLETE
  Feature '<feature>' has been abandoned.
  PR closed, remote branch deleted,
  worktree removed, state file deleted.
============================================
```

Report which steps succeeded and which were already cleaned up.

## Rules

- Available from ANY phase — no phase gate
- Never run from inside the worktree — always navigate to project root first
- Always confirm with the user — this is irreversible
- Every step after confirmation is best-effort — if one fails, continue to the next
- Never rebase, never force push — just close and delete