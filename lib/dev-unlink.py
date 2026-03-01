"""Restore the original plugin cache directory.

Removes the symlink at <version>/ and renames <version>.release back
to <version>/.

Usage: bin/flow dev-unlink
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


def dev_unlink(cache_base):
    """Unlink the plugin cache, restoring the original directory.

    Args:
        cache_base: The cache base directory containing version dirs.

    Returns:
        dict with status and message.
    """
    version = _read_version()
    cache_base = Path(cache_base)
    version_dir = cache_base / version
    release_dir = cache_base / f"{version}.release"

    if not version_dir.is_symlink():
        if version_dir.is_dir():
            return {"status": "ok", "message": f"Not linked — {version_dir} is a regular directory"}
        return {
            "status": "error",
            "message": f"Cache directory not found: {version_dir}",
        }

    version_dir.unlink()

    if release_dir.is_dir():
        release_dir.rename(version_dir)

    return {"status": "ok", "message": f"Restored {version_dir}"}


def main():
    result = dev_unlink(str(DEFAULT_CACHE_BASE))
    print(json.dumps(result))
    if result["status"] == "error":
        sys.exit(1)


if __name__ == "__main__":
    main()
