#!/usr/bin/env python3
"""
PreToolUse hook for ExitPlanMode — enforces plan_file storage.

Blocks ExitPlanMode if the state file exists, current phase is
flow-plan, and plan_file has not been stored yet. This prevents
context compaction from losing the plan location.

Exit 0 — allow
Exit 2 — block (error message on stderr)
"""

import json
import subprocess
import sys
from pathlib import Path


def validate(state_path):
    """Validate that plan_file is stored before exiting plan mode.

    Returns (allowed: bool, message: str).
    """
    if state_path is None or not Path(state_path).exists():
        return (True, "")

    try:
        state = json.loads(Path(state_path).read_text())
    except (json.JSONDecodeError, ValueError):
        return (True, "")

    if state.get("current_phase") != "flow-plan":
        return (True, "")

    if state.get("plan_file") is not None:
        return (True, "")

    return (False,
            "BLOCKED: Store plan_file before exiting plan mode. "
            "Run: bin/flow set-timestamp --set plan_file=<path>")


def _current_branch():
    """Get current branch name via git."""
    result = subprocess.run(
        ["git", "branch", "--show-current"],
        capture_output=True, text=True,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def _project_root():
    """Get project root via git worktree list."""
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        return None
    for line in result.stdout.splitlines():
        if line.startswith("worktree "):
            return line.split(" ", 1)[1]
    return None


def main():
    try:
        json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        sys.exit(0)

    branch = _current_branch()
    root = _project_root()
    if not branch or not root:
        sys.exit(0)

    state_path = Path(root) / ".flow-states" / f"{branch}.json"
    allowed, message = validate(str(state_path))
    if not allowed:
        print(message, file=sys.stderr)
        sys.exit(2)

    sys.exit(0)


if __name__ == "__main__":
    main()
