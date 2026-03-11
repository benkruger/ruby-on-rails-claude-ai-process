"""Tests for lib/validate-exit-plan.py — PreToolUse hook for ExitPlanMode."""

import json
import subprocess
import sys

from conftest import LIB_DIR, make_state, write_state

from importlib.util import spec_from_file_location, module_from_spec

SCRIPT = LIB_DIR / "validate-exit-plan.py"


def _load_module():
    """Load validate-exit-plan as a module for in-process testing."""
    spec = spec_from_file_location("validate_exit_plan", SCRIPT)
    mod = module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _run_hook(git_repo, stdin_json=None):
    """Run the hook script as a subprocess in the given git repo.

    Returns (exit_code, stderr).
    """
    if stdin_json is None:
        stdin_json = json.dumps({"tool_input": {}})
    result = subprocess.run(
        [sys.executable, str(SCRIPT)],
        input=stdin_json,
        capture_output=True,
        text=True,
        cwd=str(git_repo),
    )
    return result.returncode, result.stderr.strip()


# --- In-process validate() tests ---


def test_validate_allows_no_state_file(tmp_path):
    mod = _load_module()
    allowed, message = mod.validate(str(tmp_path / "nonexistent.json"))
    assert allowed is True
    assert message == ""


def test_validate_allows_none_state_path():
    mod = _load_module()
    allowed, message = mod.validate(None)
    assert allowed is True
    assert message == ""


def test_validate_allows_invalid_json(tmp_path):
    mod = _load_module()
    bad_file = tmp_path / "bad.json"
    bad_file.write_text("not json at all")
    allowed, message = mod.validate(str(bad_file))
    assert allowed is True
    assert message == ""


def test_validate_allows_non_plan_phase(state_dir, branch):
    mod = _load_module()
    state = make_state(
        current_phase="flow-code",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "complete",
            "flow-code": "in_progress",
        },
    )
    path = write_state(state_dir, branch, state)
    allowed, message = mod.validate(str(path))
    assert allowed is True
    assert message == ""


def test_validate_allows_plan_phase_with_plan_file(state_dir, branch):
    mod = _load_module()
    state = make_state(
        current_phase="flow-plan",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "in_progress",
        },
    )
    state["plan_file"] = "/home/user/.claude/plans/plan-abc123.md"
    path = write_state(state_dir, branch, state)
    allowed, message = mod.validate(str(path))
    assert allowed is True
    assert message == ""


def test_validate_blocks_plan_phase_without_plan_file(state_dir, branch):
    mod = _load_module()
    state = make_state(
        current_phase="flow-plan",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "in_progress",
        },
    )
    state["plan_file"] = None
    path = write_state(state_dir, branch, state)
    allowed, message = mod.validate(str(path))
    assert allowed is False
    assert "BLOCKED" in message
    assert "plan_file" in message


def test_validate_blocks_plan_phase_with_missing_plan_file_key(state_dir, branch):
    mod = _load_module()
    state = make_state(
        current_phase="flow-plan",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "in_progress",
        },
    )
    del state["plan_file"]
    path = write_state(state_dir, branch, state)
    allowed, message = mod.validate(str(path))
    assert allowed is False
    assert "BLOCKED" in message


# --- Subprocess (full hook) tests ---


def test_hook_allows_no_state_file(git_repo):
    code, stderr = _run_hook(git_repo)
    assert code == 0
    assert stderr == ""


def test_hook_allows_with_plan_file_set(git_repo, state_dir, branch):
    state = make_state(
        current_phase="flow-plan",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "in_progress",
        },
    )
    state["plan_file"] = "/home/user/.claude/plans/plan-abc123.md"
    write_state(state_dir, branch, state)
    code, stderr = _run_hook(git_repo)
    assert code == 0
    assert stderr == ""


def test_hook_blocks_without_plan_file(git_repo, state_dir, branch):
    state = make_state(
        current_phase="flow-plan",
        phase_statuses={
            "flow-start": "complete",
            "flow-plan": "in_progress",
        },
    )
    state["plan_file"] = None
    write_state(state_dir, branch, state)
    code, stderr = _run_hook(git_repo)
    assert code == 2
    assert "BLOCKED" in stderr


def test_hook_allows_invalid_json_stdin(git_repo):
    code, stderr = _run_hook(git_repo, stdin_json="not json")
    assert code == 0


def test_hook_allows_outside_git_repo(tmp_path):
    """Running outside a git repo — branch/root detection fails, allow through."""
    empty = tmp_path / "not-a-repo"
    empty.mkdir()
    code, stderr = _run_hook(empty)
    assert code == 0


# --- In-process helper function tests ---


def test_project_root_returns_none_on_git_failure(tmp_path, monkeypatch):
    """_project_root returns None when git worktree list fails."""
    mod = _load_module()
    monkeypatch.chdir(tmp_path)
    result = mod._project_root()
    assert result is None


def test_project_root_returns_none_when_no_worktree_line(monkeypatch):
    """_project_root returns None when output has no worktree line."""
    mod = _load_module()
    import subprocess as sp

    class FakeResult:
        returncode = 0
        stdout = "bare\n"

    monkeypatch.setattr(sp, "run", lambda *a, **kw: FakeResult())
    result = mod._project_root()
    assert result is None
