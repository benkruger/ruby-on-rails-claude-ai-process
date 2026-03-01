"""Symlink plugin cache to source repo for local testing.

Renames the installed cache directory to <version>.release and creates
a symlink from <version>/ to the current working directory (the source repo).

Usage: bin/flow dev-link
"""

import json
import sys
from pathlib import Path

PLUGIN_JSON = Path(__file__).resolve().parent.parent / ".claude-plugin" / "plugin.json"

DEFAULT_CACHE_BASE = (
    Path.home() / ".claude" / "plugins" / "cache" / "flow-marketplace" / "flow"
)


def _read_version():
    """Read the version from plugin.json."""
    return json.loads(PLUGIN_JSON.read_text())["version"]


def dev_link(source_root, cache_base):
    """Link the plugin cache to the source repo.

    Args:
        source_root: Path to the plugin source repo.
        cache_base: The cache base directory containing version dirs.

    Returns:
        dict with status and message.
    """
    version = _read_version()
    cache_base = Path(cache_base)
    version_dir = cache_base / version
    release_dir = cache_base / f"{version}.release"

    if version_dir.is_symlink():
        return {"status": "ok", "message": f"Already linked — {version_dir} is a symlink"}

    if not version_dir.is_dir():
        return {
            "status": "error",
            "message": f"Cache directory not found: {version_dir}",
        }

    version_dir.rename(release_dir)
    version_dir.symlink_to(Path(source_root).resolve())

    return {"status": "ok", "message": f"Linked {version_dir} → {source_root}"}


def main():
    source_root = Path.cwd()
    result = dev_link(str(source_root), str(DEFAULT_CACHE_BASE))
    print(json.dumps(result))
    if result["status"] == "error":
        sys.exit(1)


if __name__ == "__main__":
    main()
