//! Integration tests for `src/session_cost.rs` — per-session cost
//! reads and monthly aggregation. Every fixture path is
//! canonicalized at construction so prefix comparisons hold across
//! macOS `/var` ↔ `/private/var` symlinks per
//! `.claude/rules/testing-gotchas.md` "macOS Subprocess Path
//! Canonicalization".

use std::fs;

use tempfile::TempDir;

use flow_rs::session_cost::{cost_file_path, read_cost_file, read_monthly_aggregate};

// --- read_cost_file ---

/// Task 2: `read_cost_file` returns Some when present.
#[test]
fn session_cost_read_cost_file_returns_some_when_present() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let path = root.join("session-cost");
    fs::write(&path, "1.42").expect("write cost");
    assert_eq!(read_cost_file(&path), Some(1.42));
}

/// Task 3: `read_cost_file` returns None when absent.
#[test]
fn session_cost_read_cost_file_returns_none_when_absent() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let path = root.join("missing-cost");
    assert_eq!(read_cost_file(&path), None);
}

/// Cost file with non-numeric content → None (fail-open).
#[test]
fn read_cost_file_returns_none_for_non_numeric_content() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let path = root.join("cost");
    fs::write(&path, "not-a-number").expect("write");
    assert_eq!(read_cost_file(&path), None);
}

/// Cost file containing infinity → None (non-finite rejected to
/// avoid corrupting downstream cost summaries).
#[test]
fn read_cost_file_returns_none_for_infinity() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let path = root.join("cost");
    fs::write(&path, "inf").expect("write");
    assert_eq!(read_cost_file(&path), None);
}

/// Whitespace around the float value is trimmed before parsing.
#[test]
fn read_cost_file_trims_whitespace() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let path = root.join("cost");
    fs::write(&path, "  0.75  \n").expect("write");
    assert_eq!(read_cost_file(&path), Some(0.75));
}

// --- cost_file_path ---

/// `cost_file_path` resolves to
/// `<project_root>/.claude/cost/<YYYY-MM>/<session_id>` with no
/// extension — matching the producer in
/// `~/.claude/statusline-command.sh`.
#[test]
fn cost_file_path_resolves_no_extension_under_year_month_dir() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let path = cost_file_path(&root, "sid-abc");
    let expected = root
        .join(".claude")
        .join("cost")
        .join(&year_month)
        .join("sid-abc");
    assert_eq!(path, expected);
    assert!(
        path.file_name().unwrap().to_string_lossy() == "sid-abc",
        "no extension allowed"
    );
}

// --- read_monthly_aggregate ---

/// Task 4: `read_monthly_aggregate` sums every session cost file
/// under the current month's directory.
#[test]
fn session_cost_read_monthly_aggregate_sums_session_files() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let cost_dir = root.join(".claude").join("cost").join(&year_month);
    fs::create_dir_all(&cost_dir).expect("mkdir");
    fs::write(cost_dir.join("sid-1"), "1.50").expect("write 1");
    fs::write(cost_dir.join("sid-2"), "2.25").expect("write 2");
    fs::write(cost_dir.join("sid-3"), "0.10").expect("write 3");

    let total = read_monthly_aggregate(&root);
    assert!((total - 3.85).abs() < 1e-9, "expected 3.85, got {}", total);
}

/// Missing month directory → aggregate is 0.0 (no panic, no error).
#[test]
fn read_monthly_aggregate_returns_zero_when_directory_absent() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let total = read_monthly_aggregate(&root);
    assert_eq!(total, 0.0);
}

/// Corrupted entries are skipped silently; valid entries still
/// contribute. A single bad file cannot suppress the aggregate.
#[test]
fn read_monthly_aggregate_skips_corrupt_entries() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let cost_dir = root.join(".claude").join("cost").join(&year_month);
    fs::create_dir_all(&cost_dir).expect("mkdir");
    fs::write(cost_dir.join("sid-good"), "5.00").expect("write good");
    fs::write(cost_dir.join("sid-bad"), "garbage").expect("write bad");
    fs::write(cost_dir.join("sid-infinity"), "inf").expect("write inf");

    let total = read_monthly_aggregate(&root);
    assert!((total - 5.00).abs() < 1e-9, "expected 5.00, got {}", total);
}

/// Subdirectory inside the cost dir (e.g. a stray `.tmp/` folder)
/// is read attempted as a file; `fs::read_to_string` fails and the
/// entry is skipped without aborting the loop.
#[test]
fn read_monthly_aggregate_skips_directory_entries() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().canonicalize().expect("canonicalize");
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let cost_dir = root.join(".claude").join("cost").join(&year_month);
    fs::create_dir_all(&cost_dir).expect("mkdir");
    fs::create_dir_all(cost_dir.join("subdir")).expect("mkdir subdir");
    fs::write(cost_dir.join("sid-1"), "1.00").expect("write");

    let total = read_monthly_aggregate(&root);
    assert!((total - 1.00).abs() < 1e-9);
}
