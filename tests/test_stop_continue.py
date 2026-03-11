"""Tests for lib/stop-continue.py — Stop hook continuation script."""

import importlib.util
import json
import subprocess
import sys

import pytest

from conftest import LIB_DIR, make_state, write_state

SCRIPT = LIB_DIR / "stop-continue.py"

_spec = importlib.util.spec_from_file_location(
    "stop_continue", SCRIPT
)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


# --- In-process tests ---


class TestCheckContinue:
    def test_flag_set_blocks_and_clears(self, git_repo, state_dir, branch, monkeypatch):
        monkeypatch.chdir(git_repo)
        state = make_state(
            current_phase="flow-code-review",
            phase_statuses={
                "flow-start": "complete",
                "flow-plan": "complete",
                "flow-code": "complete",
                "flow-code-review": "in_progress",
            },
        )
        state["_continue_pending"] = "simplify"
        write_state(state_dir, branch, state)

        should_block, skill_name = _mod.check_continue()

        assert should_block is True
        assert skill_name == "simplify"

        updated = json.loads((state_dir / f"{branch}.json").read_text())
        assert updated["_continue_pending"] == ""

    def test_flag_empty_allows(self, git_repo, state_dir, branch, monkeypatch):
        monkeypatch.chdir(git_repo)
        state = make_state(current_phase="flow-code-review")
        state["_continue_pending"] = ""
        write_state(state_dir, branch, state)

        should_block, skill_name = _mod.check_continue()

        assert should_block is False
        assert skill_name is None

    def test_flag_absent_allows(self, git_repo, state_dir, branch, monkeypatch):
        monkeypatch.chdir(git_repo)
        state = make_state(current_phase="flow-code-review")
        write_state(state_dir, branch, state)

        should_block, skill_name = _mod.check_continue()

        assert should_block is False
        assert skill_name is None

    def test_no_state_file_allows(self, git_repo, monkeypatch):
        monkeypatch.chdir(git_repo)

        should_block, skill_name = _mod.check_continue()

        assert should_block is False
        assert skill_name is None

    def test_no_branch_allows(self, tmp_path, monkeypatch):
        monkeypatch.chdir(tmp_path)

        should_block, skill_name = _mod.check_continue()

        assert should_block is False
        assert skill_name is None

    def test_corrupt_state_file_allows(self, git_repo, state_dir, branch, monkeypatch):
        monkeypatch.chdir(git_repo)
        (state_dir / f"{branch}.json").write_text("{bad json")

        should_block, skill_name = _mod.check_continue()

        assert should_block is False
        assert skill_name is None


# --- Subprocess integration tests ---


def _run_hook(stdin_data, cwd=None):
    """Run the Stop hook script as a subprocess.

    Returns (exit_code, stdout).
    """
    result = subprocess.run(
        [sys.executable, str(SCRIPT)],
        input=stdin_data,
        capture_output=True,
        text=True,
        cwd=str(cwd) if cwd else None,
    )
    return result.returncode, result.stdout.strip()


class TestSubprocess:
    def test_flag_set_outputs_block_json(self, git_repo, state_dir, branch):
        state = make_state(
            current_phase="flow-code-review",
            phase_statuses={
                "flow-start": "complete",
                "flow-plan": "complete",
                "flow-code": "complete",
                "flow-code-review": "in_progress",
            },
        )
        state["_continue_pending"] = "simplify"
        write_state(state_dir, branch, state)

        stdin = json.dumps({})
        exit_code, stdout = _run_hook(stdin, cwd=git_repo)

        assert exit_code == 0
        output = json.loads(stdout)
        assert output["decision"] == "block"
        assert "simplify" in output["reason"]

    def test_flag_empty_no_output(self, git_repo, state_dir, branch):
        state = make_state(current_phase="flow-code-review")
        state["_continue_pending"] = ""
        write_state(state_dir, branch, state)

        stdin = json.dumps({})
        exit_code, stdout = _run_hook(stdin, cwd=git_repo)

        assert exit_code == 0
        assert stdout == ""

    def test_malformed_stdin_no_output(self, git_repo):
        exit_code, stdout = _run_hook("not json at all", cwd=git_repo)

        assert exit_code == 0
        assert stdout == ""

    def test_no_state_dir_no_output(self, git_repo):
        stdin = json.dumps({})
        exit_code, stdout = _run_hook(stdin, cwd=git_repo)

        assert exit_code == 0
        assert stdout == ""
