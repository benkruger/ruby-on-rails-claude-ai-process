# Design — Rails Framework Instructions

## Alternatives Structure

Each alternative must address:

- **Approach summary** — 2-3 sentences describing the strategy
- **Schema changes** — new tables, columns, indexes for `data/release.sql`
- **Model changes** — Base/Create split, associations, callbacks
- **Controller / route changes** — subdomain, new routes, params pattern
- **Worker changes** — any async work, which queue
- **Key trade-offs** — what you gain and what you give up

## Validation Sub-Agent Prompt

Provide these instructions to the Step 4 sub-agent (fill in the details):

> You are validating design alternatives for the FLOW design phase.
> Feature: <feature name from state>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Never use Bash for file existence checks,
> directory listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Research findings: <paste state["research"] summary, affected_files, risks>
>
> Alternatives to validate:
> <paste the 2-3 alternatives drafted in Step 3>
>
> For each alternative, check the codebase:
>
> 1. **Feasibility** — Do the files it would touch exist? Does the route
>    structure support it? Does the schema allow it?
> 2. **Conflicts** — Any naming collisions with existing code? Callback
>    chains that would interfere? Existing logic that contradicts the approach?
> 3. **Reuse opportunities** — Existing helpers, shared modules, or patterns
>    that this alternative could leverage instead of building from scratch?
> 4. **Files to modify** — Exact list of files each alternative would need
>    to create or modify.
>
> Return per-alternative:
>
> - Feasibility: confirmed / blocked (with reason)
> - Conflicts found (if any)
> - Reuse opportunities (if any)
> - Files that would need modification (full paths)

## Design Presentation Format

Show the complete design inside a fenced code block:

````text
```
============================================
  FLOW — Phase 3: Design — PROPOSAL
============================================

  Feature     : <feature description>
  Approach    : <chosen approach title>

  Schema Changes
  --------------
  <list of tables/columns/indexes — or "None">

  Model Changes
  -------------
  <Base/Create decisions, associations, callbacks>

  Controller / Route Changes
  --------------------------
  <subdomain, route, params pattern>

  Worker Changes
  --------------
  <queue, structure — or "None">

  Risks
  -----
  <risks from research that are relevant to this approach>

============================================
```
````

## Design Object Schema

Write to `.flow-states/<branch>.json` under `design`:

```json
{
  "feature_description": "<user's own words from Step 1>",
  "chosen_approach": "<approach title>",
  "rationale": "<why this approach was chosen>",
  "schema_changes": [],
  "model_changes": [],
  "controller_changes": [],
  "worker_changes": [],
  "route_changes": [],
  "risks": [],
  "approved_at": "<current UTC timestamp>"
}
```
