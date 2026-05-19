//! Smoke-test artifact contract for `hello.sh`.
//!
//! `hello.sh` is the FLOW plugin's designated smoke-test artifact
//! for end-to-end lifecycle regression passes. Each QA pass rewrites
//! the script to carry the current QA date. This test pins the
//! script's exact byte content — the shebang plus a single dated
//! greeting line and a trailing newline — so any unintended edit
//! fails CI: a stale prior-date line surviving alongside the new
//! one, a missing or altered shebang, or extra appended content all
//! diverge from the exact expected bytes, where a looser substring
//! check would let them pass green.

mod common;

use std::fs;

#[test]
fn hello_sh_is_exactly_the_current_qa_artifact() {
    let path = common::repo_root().join("hello.sh");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let expected = "#!/usr/bin/env bash\necho \"Hello, FLOW! (QA 2026-05-19)\"\n";
    assert_eq!(
        content, expected,
        "hello.sh must be exactly the current QA-pass artifact \
         (shebang + single dated greeting line + trailing newline); \
         any divergence means a stale line survived, the shebang \
         changed, or extra content was appended"
    );
}
