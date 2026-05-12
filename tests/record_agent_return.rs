//! Tests for `bin/flow record-agent-return` — appends a verified
//! agent-return entry to `phases.<phase>.agents_returned` in the
//! state file. The subcommand calls
//! `transcript_walker::verify_agent_returned_in_phase` to confirm
//! the agent actually ran (a tool_use/tool_result pair appears in
//! the persisted transcript after the most recent phase-enter
//! marker) before mutating state, so a model that did not invoke
//! the agent cannot fabricate the entry via this CLI.
//!
//! Consumed by `phase-finalize`'s required-agents gate (added in a
//! later task) to refuse phase completion when any required agent
//! neither returned (this field) nor was skipped (existing
//! agents_skipped field).

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{create_git_repo_with_remote, parse_output};
use serde_json::{json, Value};

fn flow_rs_no_recursion() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_flow-rs"));
    cmd.env_remove("FLOW_CI_RUNNING");
    cmd
}

fn write_state(repo: &Path, branch: &str, state: &Value) -> PathBuf {
    let branch_dir = repo.join(".flow-states").join(branch);
    fs::create_dir_all(&branch_dir).unwrap();
    let path = branch_dir.join("state.json");
    fs::write(&path, serde_json::to_string_pretty(state).unwrap()).unwrap();
    path
}

/// Encode a project_root the way Claude Code encodes it under
/// `~/.claude/projects/<encoded>/`: every non-alphanumeric, non-`_`,
/// non-`-` character becomes `-`. Mirrors
/// `per_flow_capture::derive_transcript_path`'s encoding rule.
fn encode_project_root(root: &Path) -> String {
    root.to_string_lossy()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Write a transcript fixture at the canonical Claude Code location
/// (`<home>/.claude/projects/<encoded-root>/<session_id>.jsonl`).
fn write_transcript_for_session(
    home: &Path,
    project_root: &Path,
    session_id: &str,
    jsonl: &str,
) -> PathBuf {
    let encoded = encode_project_root(project_root);
    let dir = home.join(".claude").join("projects").join(&encoded);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.jsonl", session_id));
    fs::write(&path, jsonl).unwrap();
    path
}

/// Canonical happy-path transcript: phase-enter marker for `phase`,
/// then Agent tool_use with `subagent_type: "flow:<agent>"`, then
/// matching tool_result.
fn happy_transcript(phase: &str, agent: &str) -> String {
    format!(
        "{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"tool_use\",\"name\":\"Bash\",\"id\":\"toolu_b1\",\"input\":{{\"command\":\"bin/flow phase-enter --phase {phase}\"}}}}]}}}}\n\
{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"tool_use\",\"name\":\"Agent\",\"id\":\"toolu_a1\",\"input\":{{\"subagent_type\":\"flow:{agent}\"}}}}]}}}}\n\
{{\"type\":\"user\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"tool_result\",\"tool_use_id\":\"toolu_a1\",\"content\":\"findings here\"}}]}}}}\n"
    )
}

fn run_record_agent_return(repo: &Path, home: &Path, args: &[&str]) -> Output {
    flow_rs_no_recursion()
        .arg("record-agent-return")
        .args(args)
        .current_dir(repo)
        .env("HOME", home)
        .env("GH_TOKEN", "invalid")
        .env("CLAUDE_PLUGIN_ROOT", env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

// --- happy path ---

#[test]
fn record_agent_return_appends_entry_when_transcript_verifies() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "happy-session-001";
    write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        &happy_transcript("flow-review", "reviewer"),
    );
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-review": {"status": "in_progress"}},
    });
    let state_path = write_state(&repo, "b", &state);

    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "ok");
    assert_eq!(data["agent"], "reviewer");
    assert_eq!(data["phase"], "flow-review");

    let on_disk: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let returned = on_disk["phases"]["flow-review"]["agents_returned"]
        .as_array()
        .expect("agents_returned array");
    assert_eq!(returned.len(), 1);
    assert_eq!(returned[0]["agent"], "reviewer");
    let ts = returned[0]["timestamp"].as_str().expect("timestamp string");
    assert!(ts.contains('T'), "timestamp should be ISO 8601: {}", ts);
}

#[test]
fn record_agent_return_succeeds_when_transcript_path_in_state_overrides_derivation() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "explicit-session";
    let transcript = write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        &happy_transcript("flow-learn", "learn-analyst"),
    );
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "transcript_path": transcript.to_string_lossy(),
        "phases": {"flow-learn": {"status": "in_progress"}},
    });
    write_state(&repo, "b", &state);

    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "learn-analyst",
            "--phase",
            "flow-learn",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "ok");
}

// --- invalid_branch ---

#[test]
fn record_agent_return_rejects_invalid_branch() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "..",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "invalid_branch");
}

// --- unknown_agent ---

#[test]
fn record_agent_return_rejects_unknown_agent() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "ci-fixer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "unknown_agent");
}

// --- unknown_phase ---

#[test]
fn record_agent_return_rejects_unknown_phase() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-code",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "unknown_phase");
}

// --- no_state_file ---

#[test]
fn record_agent_return_rejects_when_state_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "no-state",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "no_state_file");
}

// --- transcript_verification_failed (each Err variant) ---

#[test]
fn record_agent_return_reports_phase_marker_not_found_when_transcript_lacks_marker() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "no-marker";
    write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"hello\"}}\n",
    );
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-review": {"status": "in_progress"}},
    });
    write_state(&repo, "b", &state);
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "transcript_verification_failed");
    assert_eq!(data["verification_reason"], "phase_marker_not_found");
}

#[test]
fn record_agent_return_reports_tool_use_missing_when_agent_not_invoked() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "no-agent";
    let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Bash\",\"id\":\"toolu_b1\",\"input\":{\"command\":\"bin/flow phase-enter --phase flow-review\"}}]}}\n";
    write_transcript_for_session(dir.path(), &canonical_repo, session_id, jsonl);
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-review": {"status": "in_progress"}},
    });
    write_state(&repo, "b", &state);
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "transcript_verification_failed");
    assert_eq!(data["verification_reason"], "tool_use_missing");
}

#[test]
fn record_agent_return_reports_tool_result_missing_when_agent_did_not_return() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "no-result";
    let jsonl = "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Bash\",\"id\":\"toolu_b1\",\"input\":{\"command\":\"bin/flow phase-enter --phase flow-review\"}}]}}\n\
{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Agent\",\"id\":\"toolu_a1\",\"input\":{\"subagent_type\":\"flow:reviewer\"}}]}}\n";
    write_transcript_for_session(dir.path(), &canonical_repo, session_id, jsonl);
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-review": {"status": "in_progress"}},
    });
    write_state(&repo, "b", &state);
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "transcript_verification_failed");
    assert_eq!(data["verification_reason"], "tool_result_missing");
}

// --- transcript_path_invalid (no session_id, no transcript_path) ---

#[test]
fn record_agent_return_reports_no_session_when_state_lacks_session_id() {
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let state = json!({
        "branch": "b",
        "phases": {"flow-review": {"status": "in_progress"}},
    });
    write_state(&repo, "b", &state);
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "no_transcript_path");
}

// --- apply_return_mutation append-to-existing-array path ---

#[test]
fn record_agent_return_appends_to_existing_agents_returned_array() {
    // State already has phases.flow-review.agents_returned as a
    // non-empty array. The mutation must append to it rather than
    // re-initialize. Covers the `is_array() == true` branch of the
    // agents_returned guard.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "append-existing";
    write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        &happy_transcript("flow-review", "pre-mortem"),
    );
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-review": {"agents_returned": [{"agent": "reviewer", "timestamp": "2024-01-01T00:00:00-08:00"}]}},
    });
    let state_path = write_state(&repo, "b", &state);
    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "pre-mortem",
            "--phase",
            "flow-review",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let on_disk: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let arr = on_disk["phases"]["flow-review"]["agents_returned"]
        .as_array()
        .expect("agents_returned array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["agent"], "reviewer");
    assert_eq!(arr[1]["agent"], "pre-mortem");
}

// --- apply_return_mutation auto-vivification paths ---

#[test]
fn record_agent_return_initializes_phases_when_missing() {
    // State has valid session_id and a valid transcript, but no
    // `phases` key. apply_return_mutation must create `phases`,
    // the named phase entry, and the agents_returned array.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "no-phases";
    write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        &happy_transcript("flow-review", "reviewer"),
    );
    let state = json!({"branch": "b", "session_id": session_id});
    let state_path = write_state(&repo, "b", &state);

    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let on_disk: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(
        on_disk["phases"]["flow-review"]["agents_returned"][0]["agent"],
        "reviewer"
    );
}

#[test]
fn record_agent_return_initializes_named_phase_when_missing() {
    // State has phases but no entry for the specified phase.
    // apply_return_mutation must create the phase object and its
    // agents_returned array.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let canonical_repo = repo.canonicalize().unwrap();
    let session_id = "no-named-phase";
    write_transcript_for_session(
        dir.path(),
        &canonical_repo,
        session_id,
        &happy_transcript("flow-review", "reviewer"),
    );
    let state = json!({
        "branch": "b",
        "session_id": session_id,
        "phases": {"flow-code": {}},
    });
    let state_path = write_state(&repo, "b", &state);

    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let on_disk: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(
        on_disk["phases"]["flow-review"]["agents_returned"][0]["agent"],
        "reviewer"
    );
    assert!(on_disk["phases"]["flow-code"].is_object());
}

// --- mutate_state Err arm: state_write_failed ---

#[test]
fn record_agent_return_returns_error_when_state_file_is_invalid_json() {
    // mutate_state parses the state file before invoking the closure;
    // a corrupt JSON file makes mutate_state return Err, which
    // run_impl_main maps to state_write_failed.
    let dir = tempfile::tempdir().unwrap();
    let repo = create_git_repo_with_remote(dir.path());
    let branch_dir = repo.join(".flow-states").join("b");
    fs::create_dir_all(&branch_dir).unwrap();
    let state_path = branch_dir.join("state.json");
    fs::write(&state_path, "not valid json {").unwrap();

    let output = run_record_agent_return(
        &repo,
        dir.path(),
        &[
            "--branch",
            "b",
            "--agent",
            "reviewer",
            "--phase",
            "flow-review",
        ],
    );
    let data = parse_output(&output);
    assert_eq!(data["status"], "error");
    assert_eq!(data["reason"], "state_write_failed");
}
