---
title: /flow-triage-issue
nav_order: 16
parent: Skills
---

# /flow-triage-issue

**Phase:** Any

**Usage:**

```text
/flow-triage-issue <issue-number>
```

Triage a single open GitHub issue from a senior-PM-with-engineering-literacy
lens. Fetches the issue, reads referenced code (or grep-locates behavior when
the issue body names no files), checks `gh pr list --search "<num>"` and
`git log --all --grep "#<num>"` for already-shipped work, answers ten
triage questions, and produces a verdict in `{close, decompose}` with
confidence and a flip-condition. The triage Process is inlined directly in
the skill — no sub-agent dispatch. The skill renders the verdict inline and
stops — no auto-actions.

---

## What It Does

1. Parses the argument as a positive integer issue number; rejects
   non-numeric input and prompts when the argument is missing.
2. Applies a "Triage In-Progress" label to the issue so concurrent
   sessions can see the in-progress signal in the GitHub UI. Creates
   the label idempotently if it does not yet exist in the repo.
3. Fetches the issue via `gh issue view --json`. If the issue is closed
   or the fetch fails, renders an out-of-scope envelope and proceeds to
   label removal.
4. Reads every file referenced by backtick-quoted path or `path:line`
   in the issue body. If the body names no files, searches the codebase
   for the described behavior, then reads the implementation. The grep
   locates code; verification is a separate Read of the current
   implementation.
5. Checks `gh pr list --search "<num>"` (merged + open) and
   `git log --all --grep "#<num>"` for already-shipped work. Reads the
   cited code from any merged PR to verify what actually shipped.
6. Answers the 10 triage questions using plain English, citing
   `file:line` for every code claim. Applies Premise → Trace →
   Conclude reasoning per `.claude/rules/semi-formal-reasoning.md`,
   and treats mechanical blocks as presumptively intentional per
   `.claude/rules/filing-issues.md`.
7. Produces the 5-field verdict card inline (Disposition, Summary,
   Evidence, Confidence, This flips if).
8. Removes the "Triage In-Progress" label so the issue no longer
   signals active triage. The remove fires on every exit path —
   verdict rendered or out-of-scope envelope rendered.
9. Prints a brief prose hint pointing at the next manual step based
   on the disposition. The PM types the next command themselves.

---

## The 10-Question Lens

The agent answers ten questions in plain English, citing `file:line` for
every code claim:

1. Real? (evidence-grounded)
2. Still real? (current code state)
3. Framing — actual problem or symptom?
4. What (plain English)
5. Why care (plain English)
6. Who's affected and how severely?
7. How urgent?
8. How would this be fixed?
9. What does success look like?
10. Risk of the fix.

---

## The 2-Disposition Verdict

| Disposition | Meaning | Next manual step |
|---|---|---|
| `close` | No longer a real problem (already shipped, framing wrong, behavior changed) | `gh issue close <num>` after reading evidence |
| `decompose` | Real and ready for implementation planning; needs an Implementation Plan before `/flow:flow-start` | `/flow:flow-plan #N` to decompose the vanilla problem statement into a linked decomposed issue |

The set is closed in v1. Adding new dispositions requires a separate
design conversation.

---

## What This Skill Does NOT Do

- **Never closes issues.** No `gh issue close`. The PM closes manually
  after reading the evidence.
- **Never applies any label other than "Triage In-Progress".** That
  one label is the skill's only label mutation, applied in step 2 and
  removed in step 8.
- **Never comments.** No `gh issue comment`.
- **Never auto-invokes follow-on skills.** Render the verdict, stop,
  print the next-step hint. The PM types the next command.
- **Never triages closed issues.** v1 refuses closed issues with an
  out-of-scope envelope.
- **Never triages PRs.** PR review is handled by separate review
  skills.

---

## Gates

- Mutates a single label ("Triage In-Progress") on the triaged issue;
  no other GitHub state is mutated. The investigation steps
  (`gh issue view`, `gh pr list`, `git log`) are read-only.
- Display-only after the verdict is produced — no auto-actions.
- The 5-field verdict card or the out-of-scope envelope is produced
  inline by the skill itself.
