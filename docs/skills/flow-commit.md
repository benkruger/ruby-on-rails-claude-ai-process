---
title: /flow:commit
nav_order: 2
parent: Skills
---

# /flow:commit

**Phase:** Any

**Usage:** `/flow:commit` or `/flow:commit --auto`

Reviews all pending changes before committing. You see the full diff and proposed commit message, then approve or deny before anything is pushed. This is the only way commits are made in the FLOW workflow.

---

## What It Does

1. Runs `git status` and `git diff HEAD` separately and displays the diff
2. Proposes a commit message in the `tl;dr` format
3. Asks for explicit approval before touching git
4. On approval — `git add -A`, commits with the proposed message, pushes
5. On denial — asks what needs to be fixed and stops

---

## Commit Message Format

```text
Short subject line (imperative verb, under 72 characters)

tl;dr

One or two sentences explaining the WHY.

- path/to/file.rb: What changed and why
- path/to/other.rb: What changed and why
```

Subject starts with an imperative verb — Add, Fix, Update, Remove, Refactor. No prefix jargon.

---

## Gates

- Never commits without showing the diff first
- Never skips the approval step
- Never uses `--no-verify`
- Warns if `bin/ci` has not been run since the last code change

---

## Auto Mode

Pass `--auto` to skip the approval prompt when you already know the change is good:

```text
/flow:commit --auto
```

Everything else stays identical: `bin/ci` runs first, the full diff is displayed, the commit message is generated and shown, and pull-before-push happens. The only difference is that Step 3 (approval prompt) is skipped.

`--auto` is user-invoked only. Claude must never call `/flow:commit --auto` programmatically.
