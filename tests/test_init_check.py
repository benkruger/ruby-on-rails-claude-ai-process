"""Tests for lib/init-check.py — standalone version gate."""

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from conftest import LIB_DIR

SCRIPT = str(LIB_DIR / "init-check.py")


def _current_plugin_version():
    """Read the current version from plugin.json."""
    plugin_path = Path(__file__).resolve().parent.parent / ".claude-plugin" / "plugin.json"
    return json.loads(plugin_path.read_text())["version"]


def _run(cwd):
    """Run init-check.py inside the given directory."""
    result = subprocess.run(
        [sys.executable, SCRIPT],
        capture_output=True, text=True, cwd=str(cwd),
    )
    return result


def test_fails_when_flow_json_missing(tmp_path):
    """init-check.py returns error when .flow.json is missing."""
    result = _run(tmp_path)
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["status"] == "error"
    assert "/flow:init" in data["message"]


def test_fails_when_flow_version_mismatch(tmp_path):
    """init-check.py returns error when .flow.json has wrong version."""
    (tmp_path / ".flow.json").write_text(
        json.dumps({"flow_version": "0.0.0", "framework": "rails"})
    )
    result = _run(tmp_path)
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["status"] == "error"
    assert "mismatch" in data["message"]


def test_fails_when_framework_missing(tmp_path):
    """init-check.py returns error when .flow.json has no framework field."""
    version = _current_plugin_version()
    (tmp_path / ".flow.json").write_text(
        json.dumps({"flow_version": version})
    )
    result = _run(tmp_path)
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["status"] == "error"
    assert "framework" in data["message"].lower()


def test_happy_path_returns_ok(tmp_path):
    """init-check.py returns ok with framework when everything matches."""
    version = _current_plugin_version()
    (tmp_path / ".flow.json").write_text(
        json.dumps({"flow_version": version, "framework": "rails"})
    )
    result = _run(tmp_path)
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["status"] == "ok"
    assert data["framework"] == "rails"


def test_happy_path_python_framework(tmp_path):
    """init-check.py returns python framework correctly."""
    version = _current_plugin_version()
    (tmp_path / ".flow.json").write_text(
        json.dumps({"flow_version": version, "framework": "python"})
    )
    result = _run(tmp_path)
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["status"] == "ok"
    assert data["framework"] == "python"
