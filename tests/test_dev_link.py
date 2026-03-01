"""Tests for lib/dev-link.py and lib/dev-unlink.py."""

import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from conftest import LIB_DIR, REPO_ROOT


def _load_module(name, filename):
    """Import a lib script as a module (handles hyphens in filename)."""
    spec = importlib.util.spec_from_file_location(name, LIB_DIR / filename)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


_link_mod = _load_module("dev_link", "dev-link.py")
_unlink_mod = _load_module("dev_unlink", "dev-unlink.py")

LINK_SCRIPT = str(LIB_DIR / "dev-link.py")
UNLINK_SCRIPT = str(LIB_DIR / "dev-unlink.py")


def _current_version():
    plugin_path = REPO_ROOT / ".claude-plugin" / "plugin.json"
    return json.loads(plugin_path.read_text())["version"]


def _make_cache(tmp_path, version):
    """Create a fake plugin cache directory structure."""
    cache_dir = tmp_path / "cache" / "flow-marketplace" / "flow" / version
    cache_dir.mkdir(parents=True)
    (cache_dir / "plugin.json").write_text('{"name": "flow"}')
    return cache_dir


# --- dev-link ---


def test_dev_link_creates_symlink(tmp_path):
    """dev-link renames original to .release and creates symlink to source."""
    version = _current_version()
    cache_dir = _make_cache(tmp_path, version)
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"

    result = _link_mod.dev_link(str(REPO_ROOT), str(cache_base))

    assert result["status"] == "ok"
    assert cache_dir.with_name(f"{version}.release").is_dir()
    assert cache_dir.is_symlink()
    assert cache_dir.resolve() == REPO_ROOT.resolve()


def test_dev_link_idempotent(tmp_path):
    """Calling dev-link when already linked is a no-op."""
    version = _current_version()
    _make_cache(tmp_path, version)
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"

    _link_mod.dev_link(str(REPO_ROOT), str(cache_base))
    result = _link_mod.dev_link(str(REPO_ROOT), str(cache_base))

    assert result["status"] == "ok"
    assert "already" in result["message"].lower()


def test_dev_link_fails_without_cache(tmp_path):
    """dev-link returns error when cache directory does not exist."""
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"

    result = _link_mod.dev_link(str(REPO_ROOT), str(cache_base))

    assert result["status"] == "error"


# --- dev-unlink ---


def test_dev_unlink_restores(tmp_path):
    """dev-unlink removes symlink and restores the original directory."""
    version = _current_version()
    _make_cache(tmp_path, version)
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"

    _link_mod.dev_link(str(REPO_ROOT), str(cache_base))
    result = _unlink_mod.dev_unlink(str(cache_base))

    cache_dir = cache_base / version
    assert result["status"] == "ok"
    assert not cache_dir.is_symlink()
    assert cache_dir.is_dir()
    assert not cache_dir.with_name(f"{version}.release").exists()


def test_dev_unlink_idempotent(tmp_path):
    """Calling dev-unlink when not linked is a no-op."""
    version = _current_version()
    _make_cache(tmp_path, version)
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"

    result = _unlink_mod.dev_unlink(str(cache_base))

    assert result["status"] == "ok"
    assert "not linked" in result["message"].lower()


def test_dev_unlink_fails_without_cache(tmp_path):
    """dev-unlink returns error when cache directory does not exist at all."""
    cache_base = tmp_path / "cache" / "flow-marketplace" / "flow"
    cache_base.mkdir(parents=True)

    result = _unlink_mod.dev_unlink(str(cache_base))

    assert result["status"] == "error"


# --- main() via subprocess ---


def _env_with_fake_home(tmp_path):
    """Build an env dict that redirects HOME so DEFAULT_CACHE_BASE misses."""
    env = os.environ.copy()
    env["HOME"] = str(tmp_path)
    return env


def test_dev_link_main_with_missing_cache(tmp_path):
    """dev-link main() exits 1 when cache is missing."""
    result = subprocess.run(
        [sys.executable, LINK_SCRIPT],
        capture_output=True, text=True, cwd=str(tmp_path),
        env=_env_with_fake_home(tmp_path),
    )
    assert result.returncode == 1
    data = json.loads(result.stdout)
    assert data["status"] == "error"


def test_dev_unlink_main_with_missing_cache(tmp_path):
    """dev-unlink main() exits 1 when cache is missing."""
    result = subprocess.run(
        [sys.executable, UNLINK_SCRIPT],
        capture_output=True, text=True, cwd=str(tmp_path),
        env=_env_with_fake_home(tmp_path),
    )
    assert result.returncode == 1
    data = json.loads(result.stdout)
    assert data["status"] == "error"
