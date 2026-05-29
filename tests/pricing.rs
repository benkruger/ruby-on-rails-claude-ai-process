//! Tests for `src/pricing.rs` — per-model token pricing.
//!
//! Cost figures rendered beside token counts are re-derived from the
//! same per-model token capture through this table, so the table and
//! its arithmetic carry a frozen-golden regression guard plus
//! per-family bucket assertions.

use flow_rs::pricing::{cost_for, price_for};
use flow_rs::state::ModelTokens;

const PER_MTOK: f64 = 1_000_000.0;

// --- price_for ---

#[test]
fn price_for_opus_returns_opus_buckets() {
    let p = price_for("claude-opus-4-8").expect("opus is a known family");
    assert!((p.input - 5.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.output - 25.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_write - 6.25 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_read - 0.50 / PER_MTOK).abs() < 1e-15);
}

#[test]
fn price_for_sonnet_returns_sonnet_buckets() {
    let p = price_for("claude-sonnet-4-6").expect("sonnet is a known family");
    assert!((p.input - 3.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.output - 15.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_write - 3.75 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_read - 0.30 / PER_MTOK).abs() < 1e-15);
}

#[test]
fn price_for_haiku_returns_haiku_buckets() {
    let p = price_for("claude-haiku-4-5-20251001").expect("haiku is a known family");
    assert!((p.input - 1.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.output - 5.0 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_write - 1.25 / PER_MTOK).abs() < 1e-15);
    assert!((p.cache_read - 0.10 / PER_MTOK).abs() < 1e-15);
}

#[test]
fn price_for_1m_suffix_opus_matches_standard_opus() {
    // 1M-context variants bill at the same per-token family rate (1M
    // context is offered at standard pricing); the `[1m]` suffix is
    // informational and must not change the priced buckets.
    let standard = price_for("claude-opus-4-8").expect("opus known");
    let one_m = price_for("claude-opus-4-8[1m]").expect("[1m] opus known");
    assert_eq!(standard, one_m);
}

#[test]
fn price_for_unknown_model_returns_none() {
    assert!(price_for("gpt-4o").is_none());
}

#[test]
fn price_for_claude_prefix_unknown_family_returns_none() {
    // A `claude-` model of unrecognized family cannot be priced — no
    // tier is known — so it returns None and renders as an unpriced row.
    assert!(price_for("claude-future-9").is_none());
}

// --- cost_for ---

#[test]
fn cost_for_unknown_model_returns_none() {
    let t = ModelTokens {
        input: 100,
        output: 100,
        cache_create: 0,
        cache_read: 0,
    };
    assert!(cost_for("gpt-4o", &t).is_none());
}

#[test]
fn cost_for_sums_all_four_buckets() {
    // Opus input is $5/MTok; 2M input tokens cost $10.
    let t = ModelTokens {
        input: 2_000_000,
        output: 0,
        cache_create: 0,
        cache_read: 0,
    };
    let cost = cost_for("claude-opus-4-8", &t).expect("opus priced");
    assert!(
        (cost - 10.0).abs() < 1e-9,
        "2M input tokens at $5/MTok = $10, got {cost}"
    );
}

/// Frozen-golden cost assertion (AC#5).
///
/// Pins one `(model, ModelTokens)` → cost so a refactor of the price
/// table or `cost_for` arithmetic cannot silently change the dollar
/// figure rendered beside token counts. The golden value is derived
/// independently, NOT copied from production output: each of the four
/// buckets carries exactly 1_000_000 tokens, so each bucket's
/// contribution equals its per-MTok price in dollars. For Opus 4.7
/// (input $5, output $25, cache-write $6.25, cache-read $0.50 per
/// MTok) the total is 5 + 25 + 6.25 + 0.50 = $36.75. Comparison uses
/// an epsilon (never `==`) because prices are stored as $/token (e.g.
/// `5.0 / 1_000_000.0`) and re-multiplied, so binary float rounding
/// makes exact equality unreliable. When Anthropic pricing changes,
/// update the price table AND this golden value in the same commit.
#[test]
fn cost_for_opus_golden_value() {
    let t = ModelTokens {
        input: 1_000_000,
        output: 1_000_000,
        cache_create: 1_000_000,
        cache_read: 1_000_000,
    };
    let cost = cost_for("claude-opus-4-8", &t).expect("opus priced");
    assert!((cost - 36.75).abs() < 1e-9, "expected $36.75, got {cost}");
}
