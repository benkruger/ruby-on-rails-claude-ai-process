---
name: reflect
description: "Phase 7: Reflect — review what went wrong, capture learnings in CLAUDE.md, note plugin improvements. Runs before the PR is merged. The only commits are CLAUDE.md and .claude/ changes."
---

# FLOW Reflect — Phase 7: Reflect

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
prev = state.get('phases', {}).get('6', {})
if prev.get('status') != 'complete':
    print('BLOCKED: Phase 6: Review must be complete before Reflect.')
    print('Run /flow:review first.')
    sys.exit(1)
PYCHECK
```
</HARD-GATE>

## Announce

Print:

```
============================================
  FLOW — Phase 7: Reflect — STARTING
  Recommended model: Sonnet
============================================
```

## Update State

Read `.claude/flow-states/<branch>.json`. cd into the worktree.

Update Phase 7:
- `status` → `in_progress`
- `started_at` → current UTC timestamp (only if null — never overwrite)
- `session_started_at` → current UTC timestamp
- `visit_count` → increment by 1
- `current_phase` → `7`

## Logging

Wrap every Bash command (except the HARD-GATE) with timestamps in the
**same Bash call** — no separate calls for logging:

```bash
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) [Phase 7] Step X — desc — START" >> /tmp/flow-<branch>.log; COMMAND; EC=$?; echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) [Phase 7] Step X — desc — DONE (exit $EC)" >> /tmp/flow-<branch>.log; exit $EC
```

Get `<branch>` from the state file. The gap between DONE and the next
START = Claude's processing time.

---

## Step 1 — Gather all sources

Read and synthesise from three sources before asking the user anything:

### Source A — State file data

For each phase, note:
- `visit_count` > 1 → this phase had friction, was revisited
- `cumulative_seconds` unusually high → this phase took much longer than expected
- `state["notes"]` → explicit corrections captured during the session
- `state["research"]["risks"]` → risks found, check if any caused problems
- `state["research"]["open_questions"]` → anything that was unresolved
- `state["design"]["rationale"]` → why this approach was chosen, did it hold up?
- Plan sections that needed multiple revisions
- Review findings that were caught late

### Source B — Captured notes

Read `state["notes"]` in full. These are corrections and learnings
captured during the session via `/flow:note`. They are the most direct
signal of what went wrong.

### Source C — Conversation context

Review the current conversation for:
- Moments where the user corrected Claude
- Responses where Claude was overruled or pushed back
- Misunderstandings that required clarification
- Suggestions Claude made that were rejected

Note: context may have been compacted. Use what is available.
Sources A and B are the guaranteed record.

---

## Step 2 — Follow the reflection process

With the evidence gathered in Step 1, follow the shared reflection
process in `docs/reflection-process.md` (Steps A through E).

When Step E says to commit, use `/flow:commit`. The commit goes onto the
feature branch so CLAUDE.md improvements merge to main with the feature.

---

## Step 3 — Plugin improvement notes

Present the plugin gaps as a separate list — these are not committed:

```
============================================
  FLOW — Plugin Improvements to Consider
============================================

  These are improvements to the FLOW process itself.
  They are not committed — review and open issues on
  the plugin repo if you want to address them.

  - Research skill: explicitly ask about Sidekiq queue names
  - Plan skill: prompt for multi-session git workflow awareness
  - flow:commit: add note about branch-behind being common

============================================
```

Use AskUserQuestion:

> "Would you like to add anything to the plugin improvement notes
> before we close out Reflect?"
> - **No, that's everything**
> - **Yes, add this** — describe in Other

---

## Done — Update state and complete phase

Update Phase 7 in state:
1. `cumulative_seconds` += `current_time - session_started_at`
2. `status` → `complete`
3. `completed_at` → current UTC timestamp
4. `session_started_at` → `null`
5. `current_phase` → `8`

Invoke `flow:status`, then use AskUserQuestion:

> "Phase 7: Reflect is complete. The PR now includes CLAUDE.md improvements. Ready to begin Phase 8: Cleanup?"
> - **Yes, start Phase 8 now** — invoke `flow:cleanup`
> - **Not yet** — print paused banner
> - **I have a correction or learning to capture**

**If "I have a correction or learning to capture":**
1. Ask the user what they want to capture
2. Invoke `/flow:note` with their message
3. Re-ask with only "Yes, start Phase 8 now" and "Not yet"

**If Yes**, print:

```
============================================
  FLOW — Phase 7: Reflect — COMPLETE
  Merge the PR, then run /flow:cleanup.
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

- Never commit application code in Reflect — only CLAUDE.md and .claude/
- Always read all three sources before presenting findings
- Follow `docs/reflection-process.md` exactly — do not skip or reorder steps
- Plugin improvement notes are presented only — never committed