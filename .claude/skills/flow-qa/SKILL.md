---
name: flow-qa
description: "QA the FLOW plugin locally. Uninstall marketplace plugin for local testing, reinstall when done."
---

# FLOW QA

Test the FLOW plugin locally before releasing. Maintainer-only — requires the plugin to be installed.

## Usage

```text
/flow-qa
/flow-qa --start
/flow-qa --stop
```

- `/flow-qa` — auto-start if inactive, auto-restart if active
- `/flow-qa --start` — uninstall marketplace plugin, nuke cache, create `.dev-mode` marker, tell user to use `--plugin-dir`
- `/flow-qa --stop` — nuke cache, reinstall marketplace plugin, remove `.dev-mode` marker

## Flag: `--start`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it exists, print "Already in dev mode. Use `/flow-qa --stop` to exit." and stop.

### Step 2 — Uninstall marketplace plugin and nuke cache

Run:

```bash
claude plugin uninstall flow@flow-marketplace
```

Then:

```bash
rm -rf ~/.claude/plugins/cache/flow-marketplace
```

### Step 3 — Create dev mode marker

Use the Write tool to create `.flow-states/.dev-mode` with the content `active`.

### Step 4 — Announce

Print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — DEV MODE ACTIVE
============================================
```
````

Then print:

> Marketplace plugin uninstalled. To test local source, start Claude Code with:
>
> `claude --plugin-dir=$HOME/code/flow`
>
> Run `/flow-qa --stop` when done to reinstall the marketplace plugin.

## Flag: `--restart`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it does not exist, run the `--start` flow instead (skip to the `--start` section above).

### Step 2 — Nuke cache

Run:

```bash
rm -rf ~/.claude/plugins/cache/flow-marketplace
```

### Step 3 — Announce

Print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — Cache refreshed
============================================
```
````

Then print:

> Plugin cache cleared. Start Claude Code with:
>
> `claude --plugin-dir=$HOME/code/flow`

## Flag: `--stop`

### Step 1 — Check dev mode

Use the Read tool to check if `.flow-states/.dev-mode` exists.

If it does not exist, print "Not in dev mode. Nothing to stop." and stop.

### Step 2 — Nuke cache and reinstall marketplace plugin

Run:

```bash
rm -rf ~/.claude/plugins/cache/flow-marketplace
```

Then:

```bash
claude plugin install flow@flow-marketplace
```

### Step 3 — Remove dev mode marker

Use Bash to remove the marker:

```bash
rm .flow-states/.dev-mode
```

### Step 4 — Report

Print inside a fenced code block:

````markdown
```text
============================================
  FLOW QA — Dev mode stopped
============================================
```
````

## No flag (bare `/flow-qa`)

Check if `.flow-states/.dev-mode` exists using the Read tool.

- If dev mode is **active** → run the `--restart` flow above.
- If dev mode is **not active** → run the `--start` flow above.
