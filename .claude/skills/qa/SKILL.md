---
name: qa
description: "QA the FLOW plugin locally. Links plugin cache to source repo, waits for manual testing, then unlinks."
---

# FLOW QA

Test the FLOW plugin locally before releasing. Maintainer-only — requires the plugin to be installed.

## Announce

Print:

```
============================================
  FLOW QA — STARTING
============================================
```

## Step 1 — Gate on bin/ci

Run `bin/ci`. If it fails, stop:

> "bin/ci failed. Fix the failures before QA testing."

## Step 2 — Link plugin cache to source

Run:

```bash
bin/flow dev-link
```

If the output JSON has `"status": "error"`, print the error message and stop.

## Step 3 — Wait for manual testing

Print:

> Dev mode active. The plugin cache now points to this source repo.
>
> Open a **new** Claude Code session in a target project to test.
> Changes to skill files here are immediately visible in the new session.
>
> Return here when done.

Then ask:

```
AskUserQuestion: "Did QA pass?"
  - "Yes — QA passed"
  - "No — QA failed"
  - "Not done yet — keep dev-link active"
```

If **"Not done yet"**: re-ask the same question (loop until Yes or No).

## Step 4 — Unlink plugin cache

Run:

```bash
bin/flow dev-unlink
```

## Step 5 — Report

If QA passed:

```
============================================
  FLOW QA — PASSED
============================================
```

If QA failed:

```
============================================
  FLOW QA — FAILED
============================================
```
