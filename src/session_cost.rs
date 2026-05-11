//! Cost-file reads — the statusline-coupled half of account-window
//! capture. Reads per-session cost values from
//! `<project_root>/.claude/cost/<YYYY-MM>/<session_id>` (no
//! extension — matches the producer in
//! `~/.claude/statusline-command.sh`). Cost files are written by
//! Claude Code's statusline lifecycle and are frozen during
//! autonomous chains; consumers fall back to `None` rather than
//! failing when the file is absent or stale.
//!
//! Two consumer shapes:
//!
//! - **Per-flow** — `per_flow_capture::capture_for_active_state`
//!   calls [`read_cost_file`] keyed by the active session_id and
//!   patches the result into the snapshot's `session_cost_usd`
//!   field.
//! - **Status-bar aggregation** — `tui_data::load_account_metrics`
//!   calls [`read_monthly_aggregate`] to total every session's
//!   cost for the current month, displayed in the TUI header
//!   regardless of any single file's freshness.

use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the per-session cost-file path
/// `<project_root>/.claude/cost/<YYYY-MM>/<session_id>`. No
/// extension — the producer in `~/.claude/statusline-command.sh`
/// writes the file as `$cost_dir/$session_id` (line 32). The
/// month folder mirrors `read_monthly_aggregate` so the per-flow
/// snapshot reads the same file that account-monthly aggregation
/// already reads.
pub fn cost_file_path(project_root: &Path, session_id: &str) -> PathBuf {
    let now_local = chrono::Local::now();
    let year_month = now_local.format("%Y-%m").to_string();
    project_root
        .join(".claude")
        .join("cost")
        .join(year_month)
        .join(session_id)
}

/// Read a per-session cost file (a single floating-point number).
/// Missing file, malformed content, or non-finite parse returns
/// `None`. The fail-open semantics let producers continue to emit
/// snapshots even when the statusline never wrote a cost value for
/// the current session.
pub fn read_cost_file(path: &Path) -> Option<f64> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: f64 = content.trim().parse().ok()?;
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

/// Sum every per-session cost file under
/// `<project_root>/.claude/cost/<YYYY-MM>/` and return the
/// aggregate USD value. Used by the TUI header to display
/// month-to-date account spend across every session FLOW has run
/// (not just the active flow). Missing directory, unreadable
/// entries, and non-numeric content are skipped silently — a
/// single corrupt file cannot suppress the aggregate.
///
/// The month boundary is the current local-time `%Y-%m`, matching
/// the producer's convention; entries from prior months remain on
/// disk under their own directories and are invisible to this
/// aggregate.
pub fn read_monthly_aggregate(project_root: &Path) -> f64 {
    let now_local = chrono::Local::now();
    let year_month = now_local.format("%Y-%m").to_string();
    let cost_dir = project_root.join(".claude").join("cost").join(&year_month);
    let mut total = 0.0f64;
    let entries = match fs::read_dir(&cost_dir) {
        Ok(iter) => iter,
        Err(_) => return total,
    };
    for entry in entries.flatten() {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(val) = content.trim().parse::<f64>() {
                if val.is_finite() {
                    total += val;
                }
            }
        }
    }
    total
}
