---
title: /flow:release
nav_order: 6
parent: Skills
---

# /flow:release

**Phase:** Any (maintainer-only)

**Usage:** `/flow:release`

Releases a new version of the FLOW plugin. Requires push access to the repo — end users cannot run this.

---

## What It Does

1. Reads current version from `plugin.json`
2. Asks what type of release: patch, minor, or major
3. Checks for uncommitted changes and pulls latest main
4. Shows what changed since the last release (commit log)
5. Bumps version in both `plugin.json` and `marketplace.json`
6. Updates `RELEASE-NOTES.md` with a new section
7. Commits, tags, pushes, and creates a GitHub Release

---

## How Updates Reach Users

Claude Code uses the `version` field to determine updates:

1. Maintainer runs `/flow:release` — bumps version, pushes, tags
2. User runs `/plugin update` or `claude plugin update flow@flow-marketplace`
3. Claude Code compares installed version with marketplace version
4. If marketplace version is higher — pulls the new code

**Without a version bump, pushing to main does nothing for existing users.** Claude Code caches plugins by version.

---

## Gates

- Working tree must be clean — no uncommitted changes
- Both `plugin.json` and `marketplace.json` must be bumped to the same version
- GitHub Release is created from the tag
