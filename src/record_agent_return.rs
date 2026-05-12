//! `bin/flow record-agent-return` — record a verified sub-agent
//! return in `phases.<phase>.agents_returned` so `phase-finalize`'s
//! required-agents gate can confirm every required agent ran.
//!
//! Invoked by the calling phase skill (`flow-review` Step 2,
//! `flow-learn` Step 1) immediately after the agent's tool_result
//! lands. The subcommand calls
//! `transcript_walker::verify_agent_returned_in_phase` to confirm
//! the persisted Claude Code transcript carries an Agent tool_use /
//! tool_result pair for `subagent_type: "flow:<agent>"` after the
//! most recent `phase-enter --phase <phase>` Bash marker. Only on
//! verification success does the subcommand append `{agent,
//! timestamp}` to the state's `phases.<phase>.agents_returned`
//! array.
//!
//! A model that did not actually invoke the agent has no
//! tool_use/tool_result pair in the transcript and cannot reach
//! the state-write path via this CLI — closing the inline-synthesis
//! bypass.
//!
//! Tests live at `tests/record_agent_return.rs` per
//! `.claude/rules/test-placement.md`.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{json, Value};

use crate::flow_paths::FlowPaths;
use crate::hooks::transcript_walker::{normalize_gate_input, verify_agent_returned_in_phase};
use crate::lock::mutate_state;
use crate::per_flow_capture::derive_transcript_path;
use crate::required_agents::{is_known_agent, required_agents_for_phase};
use crate::session_metrics::{is_safe_session_id, is_safe_transcript_path};
use crate::utils::now;

#[derive(Parser, Debug)]
#[command(
    name = "record-agent-return",
    about = "Record a verified sub-agent return for the current phase"
)]
pub struct Args {
    /// Branch name. Validated through `FlowPaths::try_new` per
    /// `.claude/rules/branch-path-safety.md`.
    #[arg(long)]
    pub branch: String,
    /// Agent name (e.g., `reviewer`, `pre-mortem`, `adversarial`,
    /// `documentation`, `learn-analyst`). Must normalize to a
    /// member of `required_agents::REQUIRED_AGENTS`.
    #[arg(long)]
    pub agent: String,
    /// Phase the agent belongs to (e.g., `flow-review`,
    /// `flow-learn`). Must normalize to a phase key present in
    /// `required_agents::REQUIRED_AGENTS`.
    #[arg(long)]
    pub phase: String,
}

/// Append `{agent, timestamp}` to
/// `state.phases[phase].agents_returned`. Initializes intermediate
/// objects when missing and resets non-array `agents_returned` to
/// an empty array per `.claude/rules/rust-patterns.md` "State
/// Mutation Object Guards".
///
/// State-root shape is guaranteed an Object by the caller:
/// `run_impl_main` calls `resolve_transcript_path` first, which
/// returns `None` for any non-Object state (string `get`s on
/// `Value::Null`, `Value::Array`, `Value::Number`, `Value::String`,
/// `Value::Bool` all yield `None` for `session_id`/`transcript_path`).
/// `None` short-circuits the closure before this helper runs, so
/// the wrong-root-type guard from sibling `apply_skip_mutation`
/// would be unreachable here and is intentionally omitted.
fn apply_return_mutation(state: &mut Value, phase: &str, agent: &str, timestamp: &str) {
    if !state["phases"].is_object() {
        state["phases"] = json!({});
    }
    if !state["phases"][phase].is_object() {
        state["phases"][phase] = json!({});
    }
    if !state["phases"][phase]["agents_returned"].is_array() {
        state["phases"][phase]["agents_returned"] = json!([]);
    }
    let arr = state["phases"][phase]["agents_returned"]
        .as_array_mut()
        .expect("agents_returned is an array after the guard above");
    arr.push(json!({
        "agent": agent,
        "timestamp": timestamp,
    }));
}

/// Read state's `session_id` and `transcript_path` and resolve a
/// validated transcript PathBuf. Returns `None` when neither field
/// is usable: `transcript_path` fails the safe-path validator AND
/// `session_id` cannot derive a safe path. The caller maps `None`
/// to a `no_transcript_path` error.
fn resolve_transcript_path(state: &Value, home: &Path, project_root: &Path) -> Option<PathBuf> {
    let session_id = state
        .get("session_id")
        .and_then(|v| v.as_str())
        .filter(|s| is_safe_session_id(s))
        .map(|s| s.to_string());
    state
        .get("transcript_path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .filter(|p| is_safe_transcript_path(p, home))
        .or_else(|| {
            session_id
                .as_ref()
                .map(|sid| derive_transcript_path(home, project_root, sid))
                .filter(|p| is_safe_transcript_path(p, home))
        })
}

/// Main-arm dispatcher. Returns `(value, exit_code)` where exit
/// code is always `0` per the FLOW business-error convention;
/// callers parse the JSON `status` field.
pub fn run_impl_main(args: &Args, root: &Path, home: &Path) -> (Value, i32) {
    let agent_norm = normalize_gate_input(&args.agent);
    let phase_norm = normalize_gate_input(&args.phase);
    if !is_known_agent(&agent_norm) {
        return (
            json!({
                "status": "error",
                "reason": "unknown_agent",
                "message": format!(
                    "agent {:?} is not a member of REQUIRED_AGENTS",
                    args.agent
                ),
            }),
            0,
        );
    }
    if required_agents_for_phase(&phase_norm).is_empty() {
        return (
            json!({
                "status": "error",
                "reason": "unknown_phase",
                "message": format!(
                    "phase {:?} has no required agents in REQUIRED_AGENTS",
                    args.phase
                ),
            }),
            0,
        );
    }
    let paths = match FlowPaths::try_new(root, &args.branch) {
        Some(p) => p,
        None => {
            return (
                json!({
                    "status": "error",
                    "reason": "invalid_branch",
                    "message": format!("invalid branch name: {:?}", args.branch),
                }),
                0,
            );
        }
    };
    let state_path = paths.state_file();
    if !state_path.exists() {
        return (
            json!({
                "status": "error",
                "reason": "no_state_file",
                "message": format!("state file not found: {}", state_path.display()),
            }),
            0,
        );
    }
    let timestamp = now();
    // Capture an in-closure failure (transcript-path resolution or
    // verifier rejection) so the run_impl_main caller can return the
    // structured error without performing the mutation. The closure
    // never writes when `failure` is Some(_).
    let failure: RefCell<Option<Value>> = RefCell::new(None);
    let agent_for_mut = agent_norm.clone();
    let phase_for_mut = phase_norm.clone();
    let result = mutate_state(&state_path, &mut |st| {
        let transcript_path = match resolve_transcript_path(st, home, root) {
            Some(p) => p,
            None => {
                *failure.borrow_mut() = Some(json!({
                    "status": "error",
                    "reason": "no_transcript_path",
                    "message": "state has no usable session_id or transcript_path",
                }));
                return;
            }
        };
        if let Err(reason) =
            verify_agent_returned_in_phase(&transcript_path, home, &agent_for_mut, &phase_for_mut)
        {
            *failure.borrow_mut() = Some(json!({
                "status": "error",
                "reason": "transcript_verification_failed",
                "verification_reason": reason,
                "message": format!("transcript verification failed: {}", reason),
            }));
            return;
        }
        apply_return_mutation(st, &phase_for_mut, &agent_for_mut, &timestamp);
    });
    if let Some(v) = failure.into_inner() {
        return (v, 0);
    }
    match result {
        Ok(_) => (
            json!({
                "status": "ok",
                "agent": agent_norm,
                "phase": phase_norm,
                "timestamp": timestamp,
            }),
            0,
        ),
        Err(e) => (
            json!({
                "status": "error",
                "reason": "state_write_failed",
                "message": format!("failed to record agent-return: {}", e),
            }),
            0,
        ),
    }
}
