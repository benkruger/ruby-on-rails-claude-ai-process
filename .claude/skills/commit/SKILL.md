---
name: commit
description: "Review the full diff, approve or deny, then git add + commit + push. Maintainer commit skill for the FLOW plugin repo."
---

# Commit

Review all pending changes as a diff before committing. You must get explicit approval before touching git.

## Announce

At the very start, print inside a fenced code block (triple backticks) so it renders as plain monospace text and not as a markdown heading:

````
```
============================================
  Commit — STARTING
============================================
```
````

On completion (whether approved or denied), print the same way:

````
```
============================================
  Commit — COMPLETE
============================================
```
````

## Process

Follow the commit process in `docs/commit-process.md` (Steps 1 through 5).