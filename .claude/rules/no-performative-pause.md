# No Performative Pause

When the autonomous loop is running, a turn that ends with a tool
call that will re-fire the loop is a continuation. Framing such a
turn as a halt — "I'm pausing", "boundary reached", "awaiting your
direction" — is dishonest, because the next turn fires regardless
of the framing. The model must describe what it is doing, not
position a continuation as a stop.

## The Semantics

Each turn-end IS a stop. The Stop hook
(`stop_continue::check_autonomous_stop` in
`src/hooks/stop_continue.rs`) runs after the turn has ended; the
hook can ask the harness to queue another turn by emitting a
refusal payload, but it cannot retroactively prevent the turn from
ending. The model's "stopping" is real every time it ends a turn.
The dishonest framing is positioning a continuation as a pause.

"Stop Refused" in the hook's refusal message means "the autonomous
flow's end is refused" — the next turn will fire with the refusal
text as hook feedback. It does NOT mean "the model cannot end the
turn."

## The Rule

During any phase configured `continue: auto` (per
`.claude/rules/autonomous-phase-discipline.md`), the model must
not produce output that frames the turn as a halt when the same
turn ends with a tool call that re-fires the loop. The
forbidden framings include:

- Announcing a halt ("I'm pausing", "I am pausing")
- Citing an inferred boundary ("boundary reached")
- Routing the next action to the user ("awaiting your direction",
  "let me know when you want", "ready when you are", "your call.")
- Naming the antipattern as if doing it intentionally
  ("performative pause", "performative stop")

These framings are forbidden in autonomous mode regardless of
whether they appear in user-visible text or in tool-call narration.

## The Bound

The rule does NOT forbid honest pauses. A model that has genuinely
completed its scope AND ends the turn with no continuation tool
call is producing an honest pause. The rule fires when the model
produces a tool call that re-enters the loop AND the framing
positions that re-entry as "pausing."

The distinguishing test: would the next turn fire because of a
tool call this turn just emitted? If yes, framing this turn as a
halt is performative. If genuinely no, the pause is honest.

## Code-Phase Scope-Deferral Subcase

When a model in Code phase cites a Plan-phase rule —
`.claude/rules/extract-helper-refactor.md`,
`.claude/rules/scope-expansion.md`,
`.claude/rules/docs-with-behavior.md` "Plan-phase enumeration
requirement" — as permission to defer arbitrarily-sized work,
that is the same antipattern in rule-citation form.

Plan-phase rules describe what the Plan must contain BEFORE
Code begins. They are NOT Code-phase halt permission. A Code
phase that cites a Plan-phase rule to stop short of the plan's
scope is using the rule as a deferral fig leaf.

The correct Code-phase response when the plan's enumeration or
scope is missing or wrong: log a deviation per
`.claude/rules/plan-commit-atomicity.md` "Plan Signature
Deviations Must Be Logged" naming the gap, then proceed with the
work. A Plan-phase gap is not a Code-phase exit.

## Forbidden Phrasings (Catalog)

The corpus contract test
`corpus_free_of_performative_pause_phrasings` in
`tests/skill_contracts.rs` enforces the following catalog
(case-insensitive match, with U+2019 right single quotation mark
normalized to U+0027 ASCII apostrophe so smart-quote editors
cannot bypass the apostrophe-bearing entries) across CLAUDE.md,
`.claude/rules/*.md` (except this file), every
`skills/<name>/SKILL.md`, every direct-child
`.claude/skills/<name>/SKILL.md`, and every `agents/*.md`:

- `I'm pausing`
- `I am pausing`
- `boundary reached`
- `awaiting your direction`
- `let me know when you want`
- `ready when you are`
- `your call.`
- `your call?`
- `performative pause`
- `performative stop`

`your call` is split into the two terminal-punctuation forms above
so the catalog catches the canonical deferral shape (a turn ending
"...your call.") without tripping on legitimate prose words like
`your callback` or `your calling convention` where the substring
appears mid-token.

`agents/*.md` is in scope because agent prompts are read by
Claude Code as instructions every time the agent runs — the same
dynamic-instruction surface as `skills/`, which the rule's
autonomous-mode discipline targets.

When the catalog needs to grow (a new phrasing surfaces as the
same antipattern), add it to BOTH this section AND the
`PERFORMATIVE_PAUSE_PHRASINGS` constant in
`tests/skill_contracts.rs` in the same commit.

## Opt-Out Grammar

The scanner exempts legitimate citations via two mechanisms:

- **Path skip.** The scanner walks every file in scope EXCEPT
  this rule file (the catalog source). The rule body contains
  every forbidden phrasing by design.
- **Sentinel-comment opt-out.** Legitimate citations elsewhere
  in the corpus (a Learn-phase audit log, a meta-rule discussing
  the antipattern, a doc comment explaining what the scanner
  enforces) carry the sentinel comment on a sanctioned position
  relative to the forbidden phrasing.

The sentinel comment:

```text
<!-- no-performative-pause: legitimate-citation -->
```

Placement grammar mirrors
`.claude/rules/extract-helper-refactor.md`'s opt-out
exactly. The sentinel exempts EVERY forbidden-phrasing match on
the line at the sanctioned position — the discipline stays
per-line, so a multi-line citation requires multi-line sentinels.
The sanctioned positions are:

- the same line as the forbidden phrasing,
- the line directly above the forbidden phrasing, OR
- two lines above the forbidden phrasing with exactly one
  empty-or-whitespace-only line between them.

Larger gaps do not chain. The opt-out is per-line, not
per-file — 50 distinct lines of citations need 50 sentinels.
This per-line friction is intentional. It prevents the
"ever-growing exemption list" failure mode named in
`.claude/rules/tests-guard-real-regressions.md` "Forbidden
patterns".

## How to Apply

Before ending a turn during autonomous mode, ask:

> "Will the next turn fire because of a tool call I am about to
> make?"

If yes, the turn-end is a continuation. Framing must describe
the work: name the task, do the action, move on. No "pausing"
language, no deferral framing, no "your decision" routing.

If honestly no AND scope is genuinely complete, an honest pause
is fine. Say what was done; say the work is complete; end the
turn.

When a Plan-phase rule's enumeration is missing or wrong and the
Code phase discovers this mid-task, do not stop. Log the
deviation per
`.claude/rules/plan-commit-atomicity.md` "Plan Signature
Deviations Must Be Logged", then proceed with the work the plan
intended even when the plan's mechanics did not capture it.

## Cross-References

- `.claude/rules/autonomous-phase-discipline.md` — the parent
  rule covering autonomous-mode discipline more broadly. The
  Stop-hook two-exit halt model lives there; the
  performative-pause antipattern is one specific failure mode
  that discipline must catch.
- `.claude/rules/work-as-partners.md` "Self-protective routing"
  — the upstream principle (every menu-of-options or
  "what-do-you-want" framing is deflection). The
  performative-pause antipattern is the autonomous-mode shape
  of the same principle.
- `.claude/rules/plan-commit-atomicity.md` "Plan Signature
  Deviations Must Be Logged" — the sanctioned alternative
  when the Code phase encounters a Plan-phase gap.
- `.claude/rules/extract-helper-refactor.md` — Plan-phase
  enumeration rule that has been cited mid-Code as deferral
  permission. The Scope subsection in that file documents
  Plan-phase scope explicitly.
