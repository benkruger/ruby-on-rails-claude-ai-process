---
title: /flow:security
nav_order: 9
parent: Skills
---

# /flow:security

**Phase:** 7 — Security

**Usage:** `/flow:security`

Security analysis of the feature diff. Uses an Explore sub-agent
to run 10 security checks, then fixes every confirmed finding
one at a time with a commit per fix.

---

## What It Checks

| # | Check | What |
|---|-------|------|
| 1 | Authorization gaps | Missing `before_action` auth on new actions, skipped filters |
| 2 | Unscoped record access | `find(params[:id])` without scoping to current user/account/tenant |
| 3 | Mass assignment | `params.permit!`, overly broad `permit`, params passed directly |
| 4 | SQL injection | String interpolation in `where`, `execute`, `find_by_sql`, `select`, `order` |
| 5 | Data exposure | Sensitive fields in `as_json`/`to_json`/serializers, PII in logs |
| 6 | CSRF bypass | `skip_before_action :verify_authenticity_token` without API-only justification |
| 7 | Open redirects | `redirect_to` with user-controlled input |
| 8 | RuboCop disables | Any `# rubocop:disable` in the diff — automatic finding |
| 9 | Auth test coverage | New auth check with no test for the reject path |
| 10 | Route exposure | New route to action with no auth filter |

---

## Fixing Findings

Every confirmed finding gets fixed directly:

1. Fix one finding
2. Run `bin/ci`
3. Commit via `/flow:commit`
4. Mark finding as fixed in state
5. Next finding

---

## Gates

- Phase 6: Review must be complete
- bin/ci must be green after every fix
- bin/ci must be green before transitioning to Reflect
- Full diff must be read before analysis begins
