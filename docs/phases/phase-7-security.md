---
title: "Phase 7: Security"
nav_order: 8
---

# Phase 7: Security

**Command:** `/flow:security`

Security analysis of the feature diff. Scans for vulnerabilities,
authentication gaps, data exposure, and injection risks. Review
confirmed that the code matches the design — Security confirms
the code is safe.

---

## What Security Checks

The security sub-agent scans the full diff for:

- SQL injection (raw SQL, string interpolation in queries)
- Mass assignment (unpermitted params, open-ended permit)
- Authentication/authorization gaps (missing before\_action, skipped checks)
- Sensitive data exposure (secrets in logs, PII in responses)
- CSRF protection (skipped verify\_authenticity\_token)
- Insecure direct object references (IDs from params without scoping)
- Command injection (system(), exec(), backticks with user input)
- Open redirects (redirect\_to with user-controlled URLs)
- Missing input validation at system boundaries

---

## Findings

- **Critical** — exploitable vulnerability, must fix before proceeding
- **Moderate** — defense-in-depth gap, fixed directly in Security
- **Low** — noted for awareness, not fixed unless user asks

---

## bin/ci Rule

bin/ci runs after every fix made during Security.
Security does not transition to Reflect until bin/ci is green.

---

## What Comes Next

Phase 8: Reflect (`/flow:reflect`) — extract learnings and update
CLAUDE.md before the PR is merged.
