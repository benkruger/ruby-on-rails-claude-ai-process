---
title: /flow:security
nav_order: 9
parent: Skills
---

# /flow:security

**Phase:** 7 — Security

**Usage:** `/flow:security`

Security analysis of the feature diff. Uses an Explore sub-agent
to scan for vulnerabilities, then presents findings classified by
severity. Fixes critical and moderate issues, runs bin/ci after
every fix, then transitions to Reflect.

---

## What It Checks

| Area | What |
|------|------|
| SQL injection | Raw SQL, string interpolation in queries |
| Mass assignment | Unpermitted params, open-ended permit |
| Auth gaps | Missing before\_action, skipped authorization checks |
| Data exposure | Secrets in logs, PII in responses, credentials |
| CSRF | Skipped verify\_authenticity\_token |
| IDOR | IDs from params without scoping to current user/account |
| Command injection | system(), exec(), backticks with user input |
| Open redirects | redirect\_to with user-controlled URLs |
| Input validation | Missing validation at system boundaries |

---

## Fixing Findings

- Critical → AskUserQuestion: fix here or go back to Code/Plan/Design/Research
- Moderate → fix directly, commit, re-run bin/ci
- Low → note for awareness, no fix unless user asks

---

## Gates

- Phase 6: Review must be complete
- bin/ci must be green after every fix
- bin/ci must be green before transitioning to Reflect
- Full diff must be read before analysis begins
- Can return to Code, Plan, Design, or Research
