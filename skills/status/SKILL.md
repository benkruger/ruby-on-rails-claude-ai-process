---
name: status
description: "Show current ROR phase, PR link, phase checklist, and what comes next. Use any time you want to know where you are in the workflow."
---

# ROR Status

Show where you are in the ROR workflow at any moment.

## Announce

Print:

```
============================================
  ROR — ror:status — STARTING
============================================
```

## Steps

### Step 1 — Find the current PR

```bash
gh pr list --head $(git branch --show-current) --json number,title,url,body
```

If no PR is found, report: "No open PR found for this branch. Has Phase 0 been run?"

### Step 2 — Parse the phase checklist

Read the PR body and extract the Status checklist. Identify:
- Which phases are checked `[x]` — completed
- Which phases are unchecked `[ ]` — remaining
- The first unchecked phase — current phase

### Step 3 — Print status

Print a clear status report:

```
============================================
  ROR — Current Status
============================================

  Feature : <PR title>
  Branch  : <current branch>
  PR      : <PR URL>

  Phases
  ------
  [x] Phase 0: Start
  [ ] Phase 1: Research   <-- YOU ARE HERE
  [ ] Phase 2: Design
  [ ] Phase 3: Plan
  [ ] Phase 4: Implement
  [ ] Phase 5: Test
  [ ] Phase 6: Review
  [ ] Phase 7: Ship
  [ ] Phase 8: Reflect
  [ ] Phase 9: Cleanup

  Next: /ror:research

============================================
```

### Step 4 — If all phases complete

If all phases are checked, print:

```
============================================
  ROR — All phases complete!
  This feature is ready to merge.
============================================
```

## Rules

- Never modify the PR or any files — this skill is read-only
- If the PR body has no Status checklist, report that Phase 0 may not have completed correctly
