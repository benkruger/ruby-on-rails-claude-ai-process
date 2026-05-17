//! Smoke-test artifact contract for `hello.sh`.
//!
//! `hello.sh` is the FLOW plugin's designated smoke-test artifact
//! for end-to-end lifecycle regression passes. Each QA pass updates
//! line 2 to record the QA date; this test pins the current greeting
//! so a future edit that diverges fails CI.

mod common;

use std::fs;

#[test]
fn hello_sh_greets_qa_pass_2026_05_16() {
    let path = common::repo_root().join("hello.sh");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let expected = r#"echo "Hello, FLOW! (QA 2026-05-16)""#;
    assert!(
        content.contains(expected),
        "hello.sh must contain the QA-dated greeting `{}`; got:\n{}",
        expected,
        content
    );
}
