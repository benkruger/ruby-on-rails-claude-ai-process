---
name: qa
description: "QA the FLOW plugin locally. Switch marketplace to local source, test in a live session, restore when done."
---

# FLOW QA

Test the FLOW plugin locally before releasing. Maintainer-only — requires the plugin to be installed.

## Usage

```text
/qa
/qa --start
/qa --stop
/qa --refresh
```

- `/qa` — check dev mode status. If active, ask "Stop QA or refresh?" If not, ask "Start QA?"
- `/qa --start` — switch marketplace to local source, create `.dev-mode` marker
- `/qa --stop` — restore production marketplace, remove `.dev-mode` marker
- `/qa --refresh` — re-run marketplace update to pick up new changes

## Flag: `--start`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it exists, print "Already in dev mode. Use `/qa --stop` to exit or `/qa --refresh` to pick up changes." and stop.

### Step 2 — Gate on bin/ci

Run:

```bash
bin/ci
```

If it fails, stop:

> "bin/ci failed. Fix the failures before QA testing."

### Step 3 — Switch marketplace to local source

Get the project root from `git worktree list --porcelain` (first `worktree` line).

Run:

```bash
claude plugin marketplace add <project_root>
```

Then:

```bash
claude plugin marketplace update flow-marketplace
```

### Step 4 — Create dev mode marker

Use the Write tool to create `.flow-states/.dev-mode` with the content `active`.

### Step 5 — Announce

Print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — DEV MODE ACTIVE
============================================
```
````

Then print:

> Plugin cache now contains local source.
>
> Open a **new** Claude Code session in a target project to test.
> Run `/qa --refresh` after making changes to pick them up.
> Run `/qa --stop` when done.

## Flag: `--stop`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it does not exist, print "Not in dev mode. Nothing to stop." and stop.

### Step 2 — Ask pass/fail

Use AskUserQuestion:

> "Did QA pass?"
>
> - **Yes — QA passed**
> - **No — QA failed**

### Step 3 — Restore production marketplace

Run:

```bash
claude plugin marketplace add benkruger/flow
```

Then:

```bash
claude plugin marketplace update flow-marketplace
```

### Step 4 — Remove dev mode marker

Use Bash to remove the marker:

```bash
rm .flow-states/.dev-mode
```

### Step 5 — Report

If QA passed, print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — PASSED
============================================
```
````

If QA failed, print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — FAILED
============================================
```
````

## Flag: `--refresh`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it does not exist, print "Not in dev mode. Run `/qa --start` first." and stop.

### Step 2 — Refresh plugin cache

Run:

```bash
claude plugin marketplace update flow-marketplace
```

### Step 3 — Confirm

Print:

> Plugin cache refreshed. New sessions will use the latest local source.

## No flag (bare `/qa`)

Check if `.flow-states/.dev-mode` exists using the Read tool.

If dev mode is **active**, use AskUserQuestion:

> "Dev mode is active. What would you like to do?"
>
> - **Stop QA** — runs `/qa --stop`
> - **Refresh cache** — runs `/qa --refresh`

Then invoke the chosen flag.

If dev mode is **not active**, use AskUserQuestion:

> "Start QA dev mode?"
>
> - **Yes, start** — runs `/qa --start`
> - **No, cancel** — stop

Then invoke `--start` if chosen.
