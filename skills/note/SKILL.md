---
name: note
description: "Invoke automatically whenever the user corrects Claude, disagrees with a response, or says something was wrong. Also invoke explicitly with /flow:note to capture any learning mid-session. Fast — captures and continues without interrupting flow."
---

# FLOW Note

Capture a correction or learning to the state file immediately.
This skill must be fast — capture and continue, no interruption.

## When to invoke automatically

Invoke this skill BEFORE replying whenever the user:
- Corrects a mistake Claude made
- Says Claude was wrong about something
- Disagrees with a Claude response
- Clarifies something Claude misunderstood
- Says "no", "that's not right", "actually", "you missed", "I disagree"

Do not wait to be asked. Capture first, then respond.

## Steps

### Step 1 — Find the state file

```bash
python3 << 'PYCHECK'
import subprocess, sys
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
    sys.exit(0)  # No feature in progress — skip silently

print(str(state_file))
PYCHECK
```

If no state file is found, skip silently — do not interrupt the session.

### Step 2 — Write the note

Read the state file. Append to `state["notes"]`:

```json
{
  "phase": <current_phase_number>,
  "phase_name": "<current_phase_name>",
  "timestamp": "<current_utc_timestamp>",
  "type": "correction",
  "note": "<what was wrong and what is correct — written as a generic pattern, not a specific complaint>"
}
```

**Write the note as a reusable pattern, not a specific complaint:**

- Bad: *"User said I was wrong about branches"*
- Good: *"Never assume branch-behind is unlikely in a multi-session workflow — multiple active sessions means branches regularly fall behind main"*

- Bad: *"I suggested rebase, user rejected"*
- Good: *"Always merge, never rebase — rebasing is forbidden in this workflow"*

The note should read as something useful to a future session, not a log of what happened.

### Step 3 — Confirm quietly

Print one line only:

```
[note captured]
```

Then continue with the response immediately.

## For explicit invocation

When the user types `/flow:note <message>`:
- Use their message as the note text directly
- Still write to `state["notes"]` with current phase and timestamp
- Print `[note captured]` and stop

## Rules

- Never interrupt the conversation — capture and continue
- Always write as a reusable pattern
- If no state file exists, skip silently — never block a session
- Notes survive compaction and session restarts
