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

The security sub-agent runs 10 checks against the full diff:

| # | Check | What to look for |
|---|-------|-----------------|
| 1 | Authorization gaps | Missing `before_action` auth on new actions, skipped auth filters |
| 2 | Unscoped record access | `find(params[:id])` without scoping to current user/account/tenant |
| 3 | Mass assignment | `params.permit!`, overly broad `permit`, params passed directly |
| 4 | SQL injection | String interpolation in `where`, `execute`, `find_by_sql`, `select`, `order` |
| 5 | Data exposure | Sensitive fields in `as_json`/`to_json`/serializers, PII in logs, credentials |
| 6 | CSRF bypass | `skip_before_action :verify_authenticity_token` without API-only justification |
| 7 | Open redirects | `redirect_to` with user-controlled input |
| 8 | RuboCop disables | Any `# rubocop:disable` in the diff — automatic finding |
| 9 | Auth test coverage | New auth check with no test for unauthorized/forbidden case |
| 10 | Route exposure | New route to action with no auth filter |

---

## Findings

Every confirmed finding gets fixed. No severity tiers — fix one finding,
commit, then move to the next.

---

## bin/ci Rule

bin/ci runs after every fix made during Security.
Security does not transition to Reflect until bin/ci is green.

---

## What Comes Next

Phase 8: Reflect (`/flow:reflect`) — extract learnings and update
CLAUDE.md before the PR is merged.
