# Design — Python Framework Instructions

## Alternatives Structure

Each alternative must address:

- **Approach summary** — 2-3 sentences describing the strategy
- **Module changes** — new modules, modified modules, imports
- **Test changes** — new test files, fixtures needed
- **Script changes** — CLI scripts, entry points, argument parsing
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
> 1. **Feasibility** — Do the files it would touch exist? Does the module
>    structure support it? Are dependencies available?
> 2. **Conflicts** — Any naming collisions with existing code? Circular
>    import risks? Existing logic that contradicts the approach?
> 3. **Reuse opportunities** — Existing utilities, shared modules, or patterns
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

  Module Changes
  --------------
  <new/modified modules, imports>

  Test Changes
  ------------
  <new test files, fixtures — or "None">

  Script Changes
  --------------
  <CLI scripts, entry points — or "None">

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
  "module_changes": [],
  "test_changes": [],
  "script_changes": [],
  "risks": [],
  "approved_at": "<current UTC timestamp>"
}
```
