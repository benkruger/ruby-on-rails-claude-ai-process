---
name: flow-issues
description: "Group open issues by label into four sections (Blocked, Other, Vanilla, Decomposed) with mechanical sort and a copy-pasteable command per row."
---

# FLOW Issues

Fetch all open issues for the current repository, bucket them by label,
and render four tables. Read-only — never create, edit, or close issues.

## Usage

```text
/flow:flow-issues
/flow:flow-issues --ready
/flow:flow-issues --blocked
/flow:flow-issues --decomposed
/flow:flow-issues --quick-start
/flow:flow-issues --label Bug
/flow:flow-issues --label Bug --label "Tech Debt"
/flow:flow-issues --milestone v1.2
/flow:flow-issues --label Bug --ready
```

## Filter Flags

Filter flags shape which sections render. They are mutually exclusive
within each family.

- `--ready` — drop the Blocked section.
- `--blocked` — render only the Blocked section.
- `--decomposed` — render only the Decomposed section.
- `--quick-start` — render the Decomposed section without the colored
  Flow-In-Progress cluster.
- `--label <name>` — server-side filter passed to `gh issue list`
  (repeatable; multiple labels combine with AND).
- `--milestone <title>` — server-side milestone filter
  (single value; by title or number).

`--label` and `--milestone` compose with the section flags. No flag
renders all four sections.

## Concurrency

This flow is one of potentially many running simultaneously — on this
machine (multiple worktrees) and across machines (multiple engineers).
Your state file (`.flow-states/<branch>/state.json`) is yours alone. Never
read or write another branch's state. All local artifacts (logs, plan
files, temp files) are scoped by branch name. GitHub state (PRs, issues,
labels) is shared across all engineers — operations that create or modify
shared state must be idempotent.

## Announce

At the very start, output the following banner in your response (not via Bash) inside a fenced code block:

````markdown
```text
──────────────────────────────────────────────────
  FLOW v2.0.1 — flow:flow-issues — STARTING
──────────────────────────────────────────────────
```
````

## Step 1 — Fetch and Analyze

Run the analysis script. It calls `gh issue list` internally and emits
a single flat `issues` array with per-row label flags, assignees, and
URL-bearing `blocked_by` entries:

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --ready
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --blocked
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --decomposed
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --quick-start
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --label Bug
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --label Bug --label "Tech Debt"
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --milestone v1.2
```

```bash
${CLAUDE_PLUGIN_ROOT}/bin/flow analyze-issues --label Bug --ready
```

Use the first form when no filter flag was passed. Use the matching form
when a flag was passed.

Parse the JSON output. The shape is:

```json
{
  "status": "ok",
  "total": 12,
  "issues": [
    {
      "number": 1547,
      "title": "...",
      "url": "https://github.com/owner/repo/issues/1547",
      "labels": ["Decomposed"],
      "decomposed": true,
      "blocked": false,
      "native_blocked": false,
      "blocked_by": [
        {"number": 1525, "url": "https://github.com/owner/repo/issues/1525"}
      ],
      "assignees": ["alice"],
      "vanilla": false,
      "flow_in_progress": false,
      "triage_in_progress": false
    }
  ]
}
```

If `status` is `"error"`, show the error message and stop.
If `total` is 0, print the COMPLETE banner and stop.

## Step 2 — Render the four sections

Render four markdown tables in order: **Blocked**, **Other**,
**Vanilla**, **Decomposed**. Each row belongs to exactly one section;
flags resolve membership and sort order.

### Bucket assignment

Walk the `issues` array once. For each row, assign to the first
section whose condition matches:

1. **Blocked** — `blocked == true` (label OR native_blocked).
2. **Decomposed** — `decomposed == true` AND `blocked == false`.
3. **Vanilla** — `vanilla == true` AND `decomposed == false` AND
   `blocked == false`.
4. **Other** — everything else.

### Columns

The Blocked section renders five columns:

| Issue # | Title | Assignee | Blocked By | Command |
|---|---|---|---|---|

The Other, Vanilla, and Decomposed sections render four columns:

| Issue # | Title | Assignee | Command |
|---|---|---|---|

### Cell rules

- **Issue #** is `[#N](url)` — a markdown link to the issue. Always
  rendered.
- **Title** is the issue title. Bold (`**title**`) for rows where
  `flow_in_progress` or `triage_in_progress` is true; plain otherwise.
- **Assignee** is the first entry in `assignees`, or `—` when the array
  is empty. (Comma-separate additional logins if present.)
- **Blocked By** (Blocked section only) is a comma-separated list of
  `[#N](url)` entries from `blocked_by`, or `—` when `blocked_by` is
  empty but `blocked == true` (label-only block).
- **Command** depends on the bucket:
  - Blocked section: `—`.
  - Other section, NOT triage-in-progress: ```/flow:flow-explore work on issue #N```
  - Other section, triage-in-progress: `—` (a 🔍 row signals work in
    flight; the Command cell stays empty).
  - Vanilla section: ```/flow:flow-plan #N```
  - Decomposed section, NOT flow-in-progress: ```/flow:flow-start #N```
  - Decomposed section, flow-in-progress: `—` (a 🟡 row is already
    being worked; the Command cell stays empty).
- **Empty-cell convention.** Every empty cell renders as `—`.

### Color treatment

Rows carrying the canonical FLOW labels get visual treatment:

- `flow_in_progress == true` (Flow In-Progress label, Decomposed
  section only) → 🟡 prefix on the bold Title cell, Command suppressed.
- `triage_in_progress == true` (Triage In-Progress label, Other
  section only) → 🔍 prefix on the bold Title cell, Command suppressed.

Both prefixes co-occur with bold Title and a suppressed Command cell
per the bucket rules above.

### Sort rules

- **Blocked** and **Vanilla** sections: sort by issue `number`
  descending (newest issue numbers first).
- **Other** and **Decomposed** sections: sort colored rows first
  (Decomposed section: 🟡 rows; Other section: 🔍 rows), then by issue
  `number` descending within each cluster.

### Filter flag effect

- No flag → render all four sections in order.
- `--ready` → skip the Blocked section.
- `--blocked` → render only the Blocked section.
- `--decomposed` → render only the Decomposed section.
- `--quick-start` → render the Decomposed section without the 🟡 colored
  cluster.
- `--label` / `--milestone` → render whichever sections the surviving
  rows populate.

After the sections are rendered, output the following banner in your response (not via Bash) inside a fenced code block:

````markdown
```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ FLOW v2.0.1 — flow:flow-issues — COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
````

## Hard Rules

- Read-only — never create, edit, or close issues.
- Bucketing and sort are mechanical — no LLM judgment.
- Colored rows are visual-only; the Command cell stays suppressed per
  the bucket rules so the row signals "someone else owns this".
- No AskUserQuestion — this is a display-only skill.
- Never use Bash to print banners — output them as text in your response.
