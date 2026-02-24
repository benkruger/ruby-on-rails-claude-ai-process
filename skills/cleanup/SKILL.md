---
name: cleanup
description: "Phase 8: Cleanup — remove the worktree and delete the state file. Final phase. Requires Phase 7: Reflect to be complete."
---

# FLOW Cleanup — Phase 8: Cleanup

<SOFT-GATE>
Run this phase entry check as your very first action. It always exits 0
(never blocks), but may print warnings that must be included in the
confirmation step.

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
    sys.exit(0)

state = json.loads(state_file.read_text())
prev = state.get('phases', {}).get('7', {})
if prev.get('status') != 'complete':
    status = prev.get('status', 'not started')
    print(f'WARNING: Phase 7 not complete (status: {status}).')
    sys.exit(0)

print('OK')
sys.exit(0)
PYCHECK
```
</SOFT-GATE>

## Announce

Print:

```
============================================
  FLOW — Phase 8: Cleanup — STARTING
  Recommended model: Haiku
============================================
```

## Logging

Wrap every Bash command (except the SOFT-GATE) with timestamps in the
**same Bash call** — no separate calls for logging:

```bash
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) [Phase 8] Step X — desc — START" >> /tmp/flow-<branch>.log; COMMAND; EC=$?; echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) [Phase 8] Step X — desc — DONE (exit $EC)" >> /tmp/flow-<branch>.log; exit $EC
```

Get `<branch>` from the state file or `git branch --show-current`. The gap
between DONE and the next START = Claude's processing time.

---

## Steps

### Step 1 — Read state (handle missing)

If the state file exists, read `.claude/flow-states/<branch>.json` from
the project root. Note the `worktree` and `feature` values.

If the state file is missing, infer what you can:
- `branch` from `git branch --show-current`
- Detect worktree path from `git worktree list`
- Use the branch name as the feature name

Tell the user what was inferred:
> "No state file found. Inferring from git: branch '<branch>',
> worktree '<path>'."

### Step 2 — Confirm with user

This phase is destructive and irreversible. Use AskUserQuestion.

If the SOFT-GATE printed warnings, include them in the confirmation so
the user knows what's off before confirming:

> "Ready to clean up feature '<feature>'?
> ⚠ <any warnings from the gate>
> This will remove the worktree and delete the state file permanently."
> - **Yes, clean up** — proceed
> - **No, not yet** — stop here

If there were no warnings:

> "Ready to clean up feature '<feature>'?
> This will remove the worktree and delete the state file permanently."
> - **Yes, clean up** — proceed
> - **No, not yet** — stop here

### Steps 3–6 — Cleanup

Follow the cleanup process in `docs/cleanup-process.md` (Steps 1 through 4).

### Done — Print banner

Print:

```
============================================
  FLOW — Phase 8: Cleanup — COMPLETE
  Feature '<feature>' is fully done.
============================================
```

## Rules

- Never run from inside the worktree — always navigate to project root first
- Always confirm with the user before cleanup — removal is irreversible
- State file deletion is what resets the session hook — do not skip it
- Every step after confirmation is best-effort — if one fails, continue to the next