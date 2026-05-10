//! Per-session "utility skill in progress" marker file.
//!
//! Multi-step utility skills (currently just `flow:flow-create-issue`)
//! invoke the Skill tool mid-skill to delegate to a child skill. The
//! Skill tool's return is a structural surface where the model often
//! treats the handoff as a natural stopping point and returns control
//! to the user — breaking the unattended-flow contract that
//! flow-create-issue promises to its consumers (issue #1412).
//!
//! `write_marker` (called immediately after the skill's Announce
//! banner) and `clear_marker` (called immediately before the COMPLETE
//! banner and on every error-exit path) keep a JSON marker on disk at
//! `<home>/.claude/flow/utility-in-progress-<session_id>.json` for the
//! skill's full lifecycle. The Stop hook reads the marker for the
//! current Claude Code session_id and refuses turn-end with
//! `{"decision":"block"}` if a marker is present and names a known
//! multi-step utility skill.
//!
//! The marker is per-session (not per-flow): it lives under the
//! user's HOME, not `.flow-states/`, because flow-create-issue runs
//! outside any active FLOW phase. Concurrent Claude Code sessions
//! each get their own marker file because the filename includes
//! `session_id`, so cleaning up after a crashed session is a no-op
//! for other live sessions.
//!
//! Tests live at `tests/commands/utility_marker.rs` per
//! `.claude/rules/test-placement.md` — no inline `#[cfg(test)]` here.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::utils::now;
use crate::window_snapshot::is_safe_session_id;

/// The set of multi-step utility skills the Stop hook protects from
/// mid-skill turn-end. Currently only `flow:flow-create-issue` because
/// that is the only skill that delegates to another Skill tool
/// invocation mid-pipeline. Add to this list when a future utility
/// skill grows the same shape.
pub const MULTI_STEP_UTILITY_SKILLS: &[&str] = &["flow:flow-create-issue"];

/// Subdirectory under HOME where markers live. A future expansion to
/// other FLOW machine-global state can share this directory.
pub const UTILITY_MARKER_SUBDIR: &str = ".claude/flow";

/// Filename prefix for the marker file. The full filename is
/// `<MARKER_FILENAME_PREFIX><session_id>.json`.
pub const MARKER_FILENAME_PREFIX: &str = "utility-in-progress-";

/// Maximum length for a skill name — bounds the JSON payload size
/// and keeps validation cheap.
const SKILL_NAME_MAX_LEN: usize = 64;

/// Validate a `skill` argument. Accepts ASCII alphanumeric plus the
/// punctuation that appears in canonical FLOW skill names
/// (`flow:flow-<name>` — `:`, `-`, `_`). Rejects empty, anything
/// over `SKILL_NAME_MAX_LEN`, and any character outside the allow
/// set so a corrupted state-file or hostile CLI argument cannot
/// inject a path-traversal segment, NUL byte, slash, or backslash
/// into the JSON payload.
pub fn is_safe_skill_name(s: &str) -> bool {
    if s.is_empty() || s.len() > SKILL_NAME_MAX_LEN {
        return false;
    }
    if s == "." || s == ".." {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
}

/// Construct the canonical marker path for a given home directory
/// and session_id, returning `None` when the session_id fails
/// validation. Per `.claude/rules/external-input-path-construction.md`,
/// the validator runs BEFORE path construction.
pub fn marker_path(home: &Path, session_id: &str) -> Option<PathBuf> {
    if !is_safe_session_id(session_id) {
        return None;
    }
    Some(
        home.join(UTILITY_MARKER_SUBDIR)
            .join(format!("{}{}.json", MARKER_FILENAME_PREFIX, session_id)),
    )
}

/// Write the marker file for the given skill and session_id. Creates
/// the parent directory if missing. Validates `skill` and `session_id`
/// before constructing any filesystem path. The marker JSON contains
/// `skill`, `session_id`, and `started_at` (Pacific-time ISO 8601).
pub fn write_marker(home: &Path, skill: &str, session_id: &str) -> Result<PathBuf, String> {
    if !is_safe_skill_name(skill) {
        return Err(format!("invalid skill name: {:?}", skill));
    }
    let path = marker_path(home, session_id)
        .ok_or_else(|| format!("invalid session_id: {:?}", session_id))?;
    let parent = path
        .parent()
        .expect("marker_path always carries a parent (<home>/.claude/flow)");
    fs::create_dir_all(parent).map_err(|e| format!("create dir failed: {}", e))?;
    let payload = json!({
        "skill": skill,
        "session_id": session_id,
        "started_at": now(),
    });
    // serialization is structurally infallible for a json!() literal whose
    // values are validated strings — no nested types that could fail
    let serialized = serde_json::to_string_pretty(&payload)
        .expect("utility-marker JSON has only string values; serialize never fails");
    fs::write(&path, serialized).map_err(|e| format!("write failed: {}", e))?;
    Ok(path)
}

/// Remove the marker file for the given skill and session_id. Returns
/// `Ok(true)` when the file existed and was removed, `Ok(false)` when
/// it was already absent (idempotent). Validation runs first so a
/// corrupted state-file value cannot escape the canonical directory
/// even when the call is a clear.
pub fn clear_marker(home: &Path, skill: &str, session_id: &str) -> Result<bool, String> {
    if !is_safe_skill_name(skill) {
        return Err(format!("invalid skill name: {:?}", skill));
    }
    let path = marker_path(home, session_id)
        .ok_or_else(|| format!("invalid session_id: {:?}", session_id))?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("remove failed: {}", e)),
    }
}

/// Resolve the session_id for a CLI invocation. When the caller passed
/// `--session-id`, use it directly. When the caller omitted it,
/// fall back to the capture file written at SessionStart by
/// `crate::hooks::capture_session::run` — that file holds the active
/// Claude Code session_id and is the only path by which the skill can
/// reach a session_id matching what the Stop hook receives in its
/// stdin payload.
fn resolve_session_id(home: &Path, explicit: Option<&str>) -> Option<String> {
    if let Some(s) = explicit {
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    crate::hooks::capture_session::read_captured_session(home).map(|(sid, _)| sid)
}

/// CLI entry for `bin/flow set-utility-in-progress`. Accepts the
/// resolved HOME directory as a parameter so tests can drive the
/// real production path with a `TempDir` fixture. When `session_id`
/// is `None`, falls back to the SessionStart capture file so the
/// skill (which has no env-var path to Claude Code's session_id)
/// can omit the flag and still get a marker keyed by the active
/// session.
pub fn run_set_main(home: &Path, skill: &str, session_id: Option<&str>) -> (Value, i32) {
    let resolved = match resolve_session_id(home, session_id) {
        Some(s) => s,
        None => {
            return (
                json!({
                    "status": "error",
                    "message": "no session_id available: pass --session-id or run inside an active Claude Code session with a populated capture file",
                }),
                0,
            );
        }
    };
    match write_marker(home, skill, &resolved) {
        Ok(path) => (json!({"status": "ok", "path": path.to_string_lossy()}), 0),
        Err(message) => (json!({"status": "error", "message": message}), 0),
    }
}

/// CLI entry for `bin/flow clear-utility-in-progress`. Same shape as
/// `run_set_main` — returns JSON to stdout and exit code 0 for
/// business outcomes per the project convention. Same capture-file
/// fallback for `--session-id`.
pub fn run_clear_main(home: &Path, skill: &str, session_id: Option<&str>) -> (Value, i32) {
    let resolved = match resolve_session_id(home, session_id) {
        Some(s) => s,
        None => {
            return (
                json!({
                    "status": "error",
                    "message": "no session_id available: pass --session-id or run inside an active Claude Code session with a populated capture file",
                }),
                0,
            );
        }
    };
    match clear_marker(home, skill, &resolved) {
        Ok(removed) => (json!({"status": "ok", "removed": removed}), 0),
        Err(message) => (json!({"status": "error", "message": message}), 0),
    }
}
