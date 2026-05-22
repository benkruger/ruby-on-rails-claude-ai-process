//! Tests for `src/merge_approval.rs` — the branch-scoped, single-use
//! merge-approval marker store.
//!
//! The marker store is the "proceed" half of the Complete-phase merge
//! gate: `bin/flow confirm-merge` writes a marker after the user
//! confirms the squash-merge, and the merge surfaces
//! (`complete_merge`, `complete_fast::freshness_and_merge`) consult
//! and consume it immediately before `gh pr merge` when the resolved
//! `flow-complete` mode is `manual`. The contract this file locks in:
//! single-use consumption (a marker authorizes exactly one merge so a
//! `ci_rerun` loop-back forces a fresh confirmation), per-branch scope
//! (an approval written for branch A never satisfies a check for
//! branch B), corruption resilience (any unreadable / unparseable /
//! oversized marker fails closed → no approval → the merge stays
//! refused), and branch-path-safety (a `/`/`.`/`..`/NUL/empty branch
//! never reaches filesystem path construction and never panics).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{create_git_repo_with_remote, parse_output};
use flow_rs::merge_approval::{
    check_and_consume_approval, marker_path, run_impl_main, run_impl_main_with_cwd_result,
    write_approval, Args,
};
use serde_json::{json, Value};

// --- marker_path ---

#[test]
fn marker_path_valid_branch_is_some_under_branch_dir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").expect("valid branch yields a path");
    let s = p.to_string_lossy();
    assert!(
        s.contains(".flow-states/feat-x/merge-approval"),
        "marker must live under the branch-scoped state dir: {s}"
    );
}

#[test]
fn marker_path_invalid_branch_is_none() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    assert!(marker_path(root, "").is_none());
    assert!(marker_path(root, ".").is_none());
    assert!(marker_path(root, "..").is_none());
    assert!(marker_path(root, "a/b").is_none());
    assert!(marker_path(root, "a\0b").is_none());
}

// --- write_approval / check_and_consume_approval ---

#[test]
fn write_then_consume_returns_true_once_then_false() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_approval(root, "feat-x").expect("write succeeds");
    assert!(
        check_and_consume_approval(root, "feat-x"),
        "first consume returns true"
    );
    assert!(
        !check_and_consume_approval(root, "feat-x"),
        "second consume returns false (single-use)"
    );
}

#[test]
fn consume_deletes_the_marker_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_approval(root, "feat-x").expect("write succeeds");
    let p = marker_path(root, "feat-x").unwrap();
    assert!(p.exists(), "marker exists after write");
    assert!(check_and_consume_approval(root, "feat-x"));
    assert!(!p.exists(), "marker deleted after consume");
}

#[test]
fn consume_without_marker_returns_false() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn per_branch_scope_approval_under_a_does_not_satisfy_b() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_approval(root, "feat-x").expect("write succeeds");
    assert!(
        !check_and_consume_approval(root, "feat-y"),
        "approval written under feat-x must not satisfy feat-y"
    );
    // The feat-x approval is untouched and still consumable.
    assert!(check_and_consume_approval(root, "feat-x"));
}

// --- corruption resilience (fail closed → no approval) ---

#[test]
fn corruption_empty_marker_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "").unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_non_json_marker_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "not json {{{").unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_wrong_root_type_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "[1, 2, 3]").unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_approved_false_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, r#"{"approved": false, "branch": "feat-x"}"#).unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_branch_mismatch_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Hand-write a marker at feat-x's path slot but with a mismatched
    // `branch` field — per-branch scope is enforced at the marker body
    // too (defense-in-depth alongside the branch-scoped path).
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, r#"{"approved": true, "branch": "feat-y"}"#).unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_non_utf8_marker_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, [0xff_u8, 0xfe, 0xfd]).unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

#[test]
fn corruption_oversized_marker_no_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let p = marker_path(root, "feat-x").unwrap();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    // A marker larger than MARKER_BYTE_CAP (64 KiB): the capped read
    // truncates it, the truncated bytes fail JSON parse, no approval.
    fs::write(&p, "x".repeat(70 * 1024)).unwrap();
    assert!(!check_and_consume_approval(root, "feat-x"));
}

// --- branch-path-safety (never panic, never approve) ---

#[test]
fn invalid_branch_check_returns_false_never_panics() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for b in ["", ".", "..", "a/b", "a\0b"] {
        assert!(
            !check_and_consume_approval(root, b),
            "invalid branch {b:?} must yield no approval"
        );
    }
}

#[test]
fn invalid_branch_write_returns_err_never_panics() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for b in ["", ".", "..", "a/b", "a\0b"] {
        assert!(
            write_approval(root, b).is_err(),
            "invalid branch {b:?} must fail to write"
        );
    }
}

#[test]
fn write_approval_errors_when_root_is_a_file() {
    // root is a regular file, so `<root>/.flow-states/<branch>/...`
    // cannot be created — `fs::create_dir_all` returns Err and
    // `write_approval` surfaces it rather than silently approving.
    let dir = tempfile::tempdir().unwrap();
    let file_root = dir.path().join("not-a-dir");
    fs::write(&file_root, "x").unwrap();
    assert!(write_approval(&file_root, "feat-x").is_err());
}

// --- confirm-merge subcommand: run_impl_main ---

fn write_state(repo: &Path, branch: &str, state: &Value) {
    let branch_dir = repo.join(".flow-states").join(branch);
    fs::create_dir_all(&branch_dir).unwrap();
    fs::write(
        branch_dir.join("state.json"),
        serde_json::to_string_pretty(state).unwrap(),
    )
    .unwrap();
}

#[test]
fn confirm_merge_writes_marker_and_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let args = Args {
        branch: Some("feat-x".to_string()),
    };
    let (v, code) = run_impl_main(&args, &repo, &repo);
    assert_eq!(code, 0);
    assert_eq!(v["status"], "ok");
    assert_eq!(v["branch"], "feat-x");
    // The marker is now consumable exactly once.
    assert!(check_and_consume_approval(&repo, "feat-x"));
}

#[test]
fn confirm_merge_rejects_on_cwd_drift() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    // `cwd_scope::enforce` resolves the branch from git (the fixture
    // clone is on `main`). A state file under `main` with
    // `relative_cwd="api"` and cwd at the repo root is a drift the
    // guard rejects before the `--branch` override is read.
    write_state(
        &repo,
        "main",
        &json!({ "branch": "main", "relative_cwd": "api" }),
    );
    let args = Args {
        branch: Some("feat-x".to_string()),
    };
    let (v, code) = run_impl_main(&args, &repo, &repo);
    assert_eq!(code, 1);
    assert_eq!(v["reason"], "cwd_drift");
}

#[test]
fn confirm_merge_rejects_when_branch_undetectable() {
    // Non-git cwd and no `--branch` override: `resolve_branch_in`
    // returns None.
    let dir = tempfile::tempdir().unwrap();
    let nongit = dir.path().join("plain");
    fs::create_dir_all(&nongit).unwrap();
    let args = Args { branch: None };
    let (v, code) = run_impl_main(&args, dir.path(), &nongit);
    assert_eq!(code, 1);
    assert_eq!(v["reason"], "invalid_branch");
}

#[test]
fn confirm_merge_rejects_slash_bearing_branch() {
    // `resolve_branch_in` returns the `--branch` override verbatim;
    // `FlowPaths::try_new` then rejects the `/`-bearing value.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let args = Args {
        branch: Some("a/b".to_string()),
    };
    let (v, code) = run_impl_main(&args, &repo, &repo);
    assert_eq!(code, 1);
    assert_eq!(v["reason"], "invalid_branch");
}

#[test]
fn confirm_merge_rejects_when_write_fails() {
    // `.flow-states` pre-exists as a regular file, so
    // `write_approval`'s `fs::create_dir_all` cannot create the
    // branch directory and the subcommand surfaces `write_failed`.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    fs::write(repo.join(".flow-states"), "x").unwrap();
    let args = Args {
        branch: Some("feat-x".to_string()),
    };
    let (v, code) = run_impl_main(&args, &repo, &repo);
    assert_eq!(code, 1);
    assert_eq!(v["reason"], "write_failed");
}

// --- confirm-merge subcommand: run_impl_main_with_cwd_result ---

#[test]
fn confirm_merge_cwd_result_ok_delegates() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let args = Args {
        branch: Some("feat-x".to_string()),
    };
    let (v, code) = run_impl_main_with_cwd_result(&args, &repo, Ok(repo.clone()));
    assert_eq!(code, 0);
    assert_eq!(v["status"], "ok");
}

#[test]
fn confirm_merge_cwd_result_err_falls_back_to_dot() {
    // `current_dir()` failure → cwd = ".". The explicit `--branch`
    // override makes the marker path deterministic regardless of the
    // host process cwd, so the write still succeeds.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let args = Args {
        branch: Some("feat-x".to_string()),
    };
    let (v, code) = run_impl_main_with_cwd_result(
        &args,
        &repo,
        Err(std::io::Error::other("simulated current_dir failure")),
    );
    assert_eq!(code, 0);
    assert_eq!(v["status"], "ok");
}

// --- confirm-merge subcommand: real-binary dispatch (main.rs arm) ---

#[test]
fn confirm_merge_binary_writes_marker_and_exits_zero() {
    // Exercises the `main.rs` ConfirmMerge dispatch arm end to end:
    // project_root resolution, current_dir, run_impl_main_with_cwd_result,
    // and dispatch_json.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let output = Command::new(env!("CARGO_BIN_EXE_flow-rs"))
        .env_remove("FLOW_CI_RUNNING")
        .arg("confirm-merge")
        .args(["--branch", "feat-x"])
        .current_dir(&repo)
        .env("HOME", dir.path())
        .output()
        .expect("spawn flow-rs confirm-merge");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "ok");
    assert_eq!(data["branch"], "feat-x");
    // The marker the binary wrote is consumable exactly once.
    assert!(check_and_consume_approval(&repo, "feat-x"));
}
