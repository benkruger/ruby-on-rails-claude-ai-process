"""Version gate — verify /flow:flow-init has been run with matching version.

Usage: bin/flow init-check

Output (JSON to stdout):
  Success: {"status": "ok", "framework": "rails|python"}
  Failure: {"status": "error", "message": "..."}
"""

import json
import sys
from pathlib import Path


def main():
    project_root = Path.cwd()
    flow_json = project_root / ".flow.json"

    if not flow_json.exists():
        print(json.dumps({
            "status": "error",
            "message": "FLOW not initialized. Run /flow:flow-init first.",
        }))
        return

    init_data = json.loads(flow_json.read_text())

    plugin_version = json.loads(
        (Path(__file__).resolve().parent.parent
         / ".claude-plugin" / "plugin.json").read_text()
    )["version"]

    if init_data.get("flow_version") != plugin_version:
        print(json.dumps({
            "status": "error",
            "message": (
                f"FLOW version mismatch: initialized for "
                f"v{init_data.get('flow_version')}, plugin is "
                f"v{plugin_version}. Run /flow:flow-init to upgrade."
            ),
        }))
        return

    framework = init_data.get("framework")
    if framework not in ("rails", "python"):
        print(json.dumps({
            "status": "error",
            "message": "Missing framework in .flow.json. Run /flow:flow-init to configure.",
        }))
        return

    print(json.dumps({
        "status": "ok",
        "framework": framework,
    }))


if __name__ == "__main__":
    main()
