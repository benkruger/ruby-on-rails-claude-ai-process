"""Tests for bin/dependencies — the framework dependency updater."""

import os
import subprocess

from conftest import BIN_DIR


def test_script_is_executable():
    dep = BIN_DIR / "dependencies"
    assert dep.exists(), "bin/dependencies must exist"
    assert os.access(dep, os.X_OK), "bin/dependencies must be executable"


def test_script_is_valid_bash(tmp_path):
    dep = BIN_DIR / "dependencies"
    result = subprocess.run(
        ["bash", "-n", str(dep)],
        capture_output=True, text=True,
    )
    assert result.returncode == 0, f"Syntax error: {result.stderr}"


def test_skips_when_no_requirements_or_pyproject(tmp_path):
    """Running in a directory with neither requirements.txt nor pyproject.toml exits cleanly."""
    script = tmp_path / "dependencies"
    script.write_text((BIN_DIR / "dependencies").read_text())
    script.chmod(0o755)
    result = subprocess.run(
        ["bash", str(script)],
        capture_output=True, text=True, cwd=str(tmp_path),
    )
    assert result.returncode == 0
