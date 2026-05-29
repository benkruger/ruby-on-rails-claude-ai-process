//! Cost-file path resolution and month-to-date aggregation. The
//! per-session cost file lives at
//! `<project_root>/.claude/cost/<YYYY-MM>/<session_id>` (no
//! extension). FLOW's `write-session-cost` SessionStart hook writes
//! a token-derived cost there (priced from the session's `by_model`
//! token rollup via `pricing`), so month-to-date spend reconciles
//! with the token counts. Consumers fall back to `0.0` rather than
//! failing when a file is absent or stale.
//!
//! Consumer:
//!
//! - **Status-bar aggregation** — `tui_data::load_account_metrics`
//!   calls [`read_monthly_aggregate`] to total every session's
//!   cost for the current month, displayed in the TUI header
//!   regardless of any single file's freshness.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::session_metrics::is_safe_session_id;

/// Hard cap on bytes read per cost file. A per-session cost file
/// (written by `write-session-cost` or the user's statusline)
/// holds a single floating-point number on one line — under 30
/// bytes in practice. 1 KB bounds the read against a runaway or hostile
/// file (a symlink pointed at a large system log, a
/// multi-megabyte padding attack) while leaving generous headroom
/// for any future single-line cost format. The cap applies to
/// every per-entry read in [`read_monthly_aggregate`]'s directory
/// walk, per `.claude/rules/external-input-path-construction.md`
/// "Enforce a documented size cap on every external read".
const COST_FILE_BYTE_CAP: u64 = 1024;

/// Resolve the per-session cost-file path
/// `<project_root>/.claude/cost/<YYYY-MM>/<session_id>`. No
/// extension — `write-session-cost` writes the active session's
/// token-derived cost here, and the user's statusline writes its
/// own cost to the same `$cost_dir/$session_id` path. The month
/// folder is the one [`read_monthly_aggregate`] sums for the
/// month-to-date total.
///
/// Returns `None` when `session_id` fails
/// [`crate::session_metrics::is_safe_session_id`] — empty, `.`,
/// `..`, path separators (`/`, `\`), NUL bytes, oversized
/// strings, or any character outside the closed alphanumeric +
/// `-` + `_` set. Per
/// `.claude/rules/external-input-path-construction.md` "Validate
/// before constructing", the validator runs at the function
/// boundary so any caller — present or future — gets the same
/// gate independent of upstream validation.
pub fn cost_file_path(project_root: &Path, session_id: &str) -> Option<PathBuf> {
    if !is_safe_session_id(session_id) {
        return None;
    }
    let now_local = chrono::Local::now();
    let year_month = now_local.format("%Y-%m").to_string();
    Some(
        project_root
            .join(".claude")
            .join("cost")
            .join(year_month)
            .join(session_id),
    )
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
///
/// Each entry read is capped at [`COST_FILE_BYTE_CAP`] (1 KB)
/// via `file.take(COST_FILE_BYTE_CAP)` so a single oversized file
/// in the cost directory cannot OOM the walker.
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
        if let Ok(file) = fs::File::open(entry.path()) {
            let mut content = String::new();
            if file
                .take(COST_FILE_BYTE_CAP)
                .read_to_string(&mut content)
                .is_ok()
            {
                if let Ok(val) = content.trim().parse::<f64>() {
                    // Cost is non-negative. Skip finite-but-negative
                    // values (corrupt write, hand edit, or hostile file)
                    // so a single entry cannot drive the month-to-date
                    // aggregate negative and bury every other session's
                    // cost — the same corruption-resilience invariant the
                    // `is_finite` filter enforces for `inf`/`NaN`.
                    if val.is_finite() && val >= 0.0 {
                        total += val;
                    }
                }
            }
        }
    }
    total
}
