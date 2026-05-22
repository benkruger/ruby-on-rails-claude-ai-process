//! Branch-scoped, single-use merge-approval marker store, plus the
//! `bin/flow confirm-merge` subcommand that writes the marker.
//!
//! `flow-complete` is the Phase 5 terminal skill. Its autonomy mode
//! (`auto` = merge without asking, `manual` = confirm first) is
//! configured per-project under `skills.flow-complete` and resolved
//! at runtime via `resolve_skill_mode::resolve`. When the mode is
//! `manual`, the squash-merge is gated: both merge surfaces
//! (`complete_merge`, `complete_fast::freshness_and_merge`) require a
//! merge-approval marker immediately before `gh pr merge` and refuse
//! with `{"status":"error","reason":"merge_not_confirmed"}` when it
//! is absent. `confirm-merge` is the "proceed" half — the
//! flow-complete skill invokes it on the user's "Yes, merge" answer.
//!
//! Three invariants the store enforces:
//!
//! - **Single-use.** Consumption deletes the marker. A merge that
//!   loops back through the confirmation prompt (a `ci_rerun`
//!   re-verification) finds no marker and requires a fresh
//!   confirmation. There is no "consumed" flag — file presence IS
//!   the unconsumed state.
//! - **Per-branch scope.** The marker lives under the per-branch
//!   state directory, so a marker written for branch A is never
//!   visible to a check for branch B. The marker body ALSO carries
//!   the branch and `check_and_consume_approval` re-verifies it, so
//!   a hand-moved marker file cannot satisfy a check for a
//!   different branch.
//! - **Fail-closed corruption resilience.** Any unreadable,
//!   oversized, unparseable, wrong-root-type, `approved != true`, or
//!   branch-mismatched marker yields no approval. The merge then
//!   stays refused — a corrupt marker can never become an escape
//!   hatch.
//!
//! Markers live at `<project_root>/.flow-states/<branch>/merge-approval`
//! — branch-scoped under `.flow-states/` (project root, never the
//! worktree) so concurrent flows never collide and
//! `flow-abort`/`flow-complete` cleanup removes them with the branch
//! subdirectory.
//!
//! The branch reaches filesystem path construction only through
//! `FlowPaths::try_new`, which rejects empty / `.` / `..` /
//! `/`-bearing / NUL-bearing branches per
//! `.claude/rules/branch-path-safety.md`.
//!
//! Tests live at `tests/merge_approval.rs` per
//! `.claude/rules/test-placement.md`.

use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{json, Value};

use crate::flow_paths::FlowPaths;
use crate::git::resolve_branch_in;

/// Maximum bytes read from a marker file. Markers this module writes
/// are a few dozen bytes of JSON; the cap bounds I/O when the marker
/// path holds a corrupted or hostile oversized file (a hand-edit, an
/// interrupted unrelated write, a symlink to a large file). Per
/// `.claude/rules/external-input-path-construction.md` every external
/// read enforces a documented byte cap.
const MARKER_BYTE_CAP: u64 = 64 * 1024;

/// Marker filename under the per-branch state directory.
const MARKER_FILENAME: &str = "merge-approval";

/// The marker file path for `branch`, or `None` when `branch` fails
/// `FlowPaths::is_valid_branch` (empty / `.` / `..` / `/`-bearing /
/// NUL-bearing). Callers treat `None` as "no approval possible" — the
/// merge gate keeps refusing, the subcommand returns a structured
/// error.
pub fn marker_path(root: &Path, branch: &str) -> Option<PathBuf> {
    let paths = FlowPaths::try_new(root, branch)?;
    Some(paths.branch_dir().join(MARKER_FILENAME))
}

/// Write a merge-approval marker authorizing exactly one subsequent
/// squash-merge of `branch`. Creates the branch-scoped state
/// directory if absent. Returns `Err` when `branch` is invalid (no
/// path can be constructed) or on any filesystem failure — the caller
/// surfaces a structured error rather than silently approving.
pub fn write_approval(root: &Path, branch: &str) -> io::Result<()> {
    let path = marker_path(root, branch).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid branch name: {branch:?}"),
        )
    })?;
    // `marker_path` always yields `<branch_dir>/merge-approval` — a
    // path at least three components deep — so `.parent()` is
    // structurally `Some`. The `.expect` documents the invariant; it
    // is unreachable, not a panic vector.
    let parent = path
        .parent()
        .expect("marker_path yields a path with a parent directory");
    fs::create_dir_all(parent)?;
    let body = json!({ "approved": true, "branch": branch });
    fs::write(&path, body.to_string())
}

/// Consult and consume the merge-approval marker for `branch`.
///
/// Returns `true` iff a valid, unconsumed marker existed AND was
/// successfully deleted (single-use consume-on-allow). Every other
/// outcome returns `false` so the merge gate keeps refusing:
///
/// - invalid branch (no marker path constructible)
/// - missing / unreadable marker
/// - marker larger than `MARKER_BYTE_CAP`
/// - non-JSON or wrong-root-type content
/// - `approved` not boolean `true`
/// - `branch` field absent or not equal to `branch`
/// - the marker existed and validated but `fs::remove_file` failed
///   (fail-closed: if it cannot be consumed it must not authorize,
///   so a subsequent merge cannot reuse the same marker)
pub fn check_and_consume_approval(root: &Path, branch: &str) -> bool {
    let path = match marker_path(root, branch) {
        Some(p) => p,
        None => return false,
    };
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = String::new();
    if BufReader::new(file.take(MARKER_BYTE_CAP))
        .read_to_string(&mut buf)
        .is_err()
    {
        return false;
    }
    let parsed: Value = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return false,
    };
    if obj.get("approved").and_then(Value::as_bool) != Some(true) {
        return false;
    }
    if obj.get("branch").and_then(Value::as_str) != Some(branch) {
        return false;
    }
    // Valid + unconsumed: deleting the marker IS the consume. Only
    // report approval when the delete succeeds, so a failed remove
    // cannot leave a reusable marker behind.
    fs::remove_file(&path).is_ok()
}

#[derive(Parser, Debug)]
#[command(
    name = "confirm-merge",
    about = "Record a single-use user confirmation to squash-merge the flow's PR"
)]
pub struct Args {
    /// Branch whose `.flow-states/<branch>/` holds the marker.
    /// Optional; resolved from the worktree cwd when absent.
    #[arg(long)]
    pub branch: Option<String>,
}

fn err(reason: &str, message: impl Into<String>) -> (Value, i32) {
    (
        json!({"status": "error", "reason": reason, "message": message.into()}),
        1,
    )
}

/// Main-arm dispatcher that accepts `cwd` as a `Result` so the
/// `current_dir()`-failure fallback (deleted-cwd / chroot) lives in
/// the module where a unit test can drive it — keeping the
/// `src/main.rs` arm a closure-free one-liner. Mirrors
/// `approve_shared_config::run_impl_main_with_cwd_result`.
pub fn run_impl_main_with_cwd_result(
    args: &Args,
    root: &Path,
    cwd_result: std::io::Result<PathBuf>,
) -> (Value, i32) {
    let cwd = cwd_result.unwrap_or(PathBuf::from("."));
    run_impl_main(args, root, &cwd)
}

/// Main-arm dispatcher. `cwd` is the subcommand's working directory
/// (inside the flow worktree). Exit code is `1` on every rejection so
/// a non-confirmation can never silently produce an approval marker;
/// `0` with `{"status":"ok"}` when the marker is written.
pub fn run_impl_main(args: &Args, root: &Path, cwd: &Path) -> (Value, i32) {
    // State-mutator cwd guard (rust-patterns "Guard Universality
    // Across CLI Entry Points"): this subcommand writes a marker, so
    // it enforces the same drift guard as other state mutators.
    if let Err(message) = crate::cwd_scope::enforce(cwd, root) {
        return err("cwd_drift", message);
    }

    let branch = match resolve_branch_in(args.branch.as_deref(), cwd, root) {
        Some(b) => b,
        None => return err("invalid_branch", "could not determine branch"),
    };
    // Branch-path-safety: reject `/`-bearing and other escape shapes
    // before any `.flow-states/` path construction.
    if FlowPaths::try_new(root, &branch).is_none() {
        return err("invalid_branch", format!("invalid branch: {branch:?}"));
    }

    match write_approval(root, &branch) {
        Ok(()) => (json!({"status": "ok", "branch": branch}), 0),
        Err(e) => err("write_failed", format!("failed to write approval: {e}")),
    }
}
