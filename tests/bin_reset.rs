//! Tests for bin/reset — the FLOW state-wipe shell script.
//!
//! Validates that `bin/reset` removes `<project_root>/.flow-states/`
//! when invoked from the project root AND when invoked from inside a
//! worktree, and rejects bare repositories. The script's
//! `git rev-parse --git-common-dir` resolution is what makes the
//! worktree case work: from a worktree, the command points at the
//! main repo's `.git`, and `..` walks up to the main repo root. Tests
//! cover both project-root and worktree invocation contexts, the
//! bare-repository rejection branch, and the executable-mode contract
//! that the marketplace ships the file as a runnable binary.

mod common;

use std::fs;
use std::process::Command;

fn run_reset(cwd: &std::path::Path) -> std::process::Output {
    let script = common::bin_dir().join("reset");
    Command::new("bash")
        .arg(&script)
        .current_dir(cwd)
        .output()
        .expect("spawn bin/reset")
}

/// Initialize a git repo at `dir`, configure user, make an empty
/// commit so worktree operations are possible.
fn init_repo(dir: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .expect("git init");
    for (key, val) in [
        ("user.email", "test@test.com"),
        ("user.name", "Test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new("git")
            .args(["config", key, val])
            .current_dir(dir)
            .output()
            .expect("git config");
    }
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir)
        .output()
        .expect("git commit");
}

/// Populate `.flow-states/` with representative content so the test
/// can assert the directory is removed in full.
fn seed_flow_states(project_root: &std::path::Path) {
    let states = project_root.join(".flow-states");
    let branch_dir = states.join("test-branch");
    fs::create_dir_all(&branch_dir).expect("create branch dir");
    fs::write(branch_dir.join("state.json"), "{}").expect("write state.json");
    fs::write(states.join("orchestrate-queue.json"), "{}").expect("write queue");
}

/// From the project root, bin/reset removes `<root>/.flow-states/`
/// and exits 0.
#[test]
fn reset_removes_flow_states_from_project_root() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonicalize");
    init_repo(&root);
    seed_flow_states(&root);
    assert!(root.join(".flow-states").is_dir(), "fixture precondition");

    let output = run_reset(&root);

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !root.join(".flow-states").exists(),
        ".flow-states/ should be removed"
    );
}

/// From inside a worktree, bin/reset still removes the MAIN repo's
/// `.flow-states/` — git-common-dir resolution lands at the main
/// repo, not the worktree.
#[test]
fn reset_removes_main_repo_flow_states_from_worktree() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonicalize");
    init_repo(&root);
    seed_flow_states(&root);

    // Create a worktree under .worktrees/feat.
    let worktree_path = root.join(".worktrees").join("feat");
    fs::create_dir_all(worktree_path.parent().unwrap()).expect("create .worktrees");
    let wt_output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat",
            &worktree_path.to_string_lossy(),
        ])
        .current_dir(&root)
        .output()
        .expect("git worktree add");
    assert!(
        wt_output.status.success(),
        "git worktree add failed: stderr={}",
        String::from_utf8_lossy(&wt_output.stderr)
    );

    let worktree = worktree_path.canonicalize().expect("canonicalize worktree");
    assert!(
        root.join(".flow-states").is_dir(),
        "main .flow-states/ precondition"
    );

    let output = run_reset(&worktree);

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !root.join(".flow-states").exists(),
        "main repo's .flow-states/ should be removed when invoked from worktree"
    );
}

/// Inside a bare repository, bin/reset refuses to operate because
/// there is no working tree to compute a project root against and
/// naive resolution would target the bare repo's parent directory.
#[test]
fn reset_rejects_bare_repository() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonicalize");
    let bare = root.join("bare.git");
    fs::create_dir_all(&bare).expect("create bare dir");

    let init_output = Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .current_dir(&bare)
        .output()
        .expect("git init --bare");
    assert!(
        init_output.status.success(),
        "git init --bare failed: stderr={}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    // Seed a .flow-states/ in the bare repo's parent to verify the
    // script doesn't reach it. A buggy resolution would target
    // `<bare>/..` and wipe this directory.
    seed_flow_states(&root);

    let output = run_reset(&bare);

    assert!(
        !output.status.success(),
        "expected non-zero exit on bare repo, got 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to operate inside a bare repository"),
        "expected bare-repo rejection message, got stderr: {}",
        stderr
    );
    assert!(
        root.join(".flow-states").is_dir(),
        "bare repo's parent .flow-states/ must not be touched"
    );
}

/// The bin/reset script must be tracked in git with executable mode
/// 0755 so the marketplace ships an executable binary.
#[test]
fn reset_script_is_tracked_with_executable_mode() {
    let repo = common::repo_root();
    let output = Command::new("git")
        .args(["ls-files", "--stage", "bin/reset"])
        .current_dir(&repo)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    assert!(
        !line.is_empty(),
        "bin/reset must be tracked by git (got empty ls-files output)"
    );
    assert!(
        line.starts_with("100755"),
        "bin/reset must be tracked with mode 100755 (executable); got: {}",
        line
    );
}
