//! Per-model token pricing for cost reconstruction.
//!
//! The Token Cost table and the statusline month-to-date figure must
//! reconcile against the token counts rendered beside them. They
//! reconcile only when cost is derived from the SAME per-model token
//! capture the counts come from, priced through one table at one
//! instant — never sampled from a separate cost source on a separate
//! clock. This module is that single table: [`cost_for`] prices a
//! model's [`ModelTokens`] buckets, and [`price_for`] exposes the
//! per-token rates for a model name.
//!
//! Pricing is keyed by model family (Opus / Sonnet / Haiku) detected
//! as a substring of the `claude-` model name, mirroring the
//! lookup-table discipline of `session_metrics::context_window_size`:
//! a `claude-` model of unrecognized family — and any non-`claude-`
//! model — returns `None` so an unpriced row renders as `—` rather
//! than a misleading `$0`. The `[1m]` 1M-context variants bill at the
//! same per-token family rate (1M context is offered at standard
//! pricing), so the suffix needs no separate branch — the family
//! substring still matches.
//!
//! Maintenance path: the dollar figures below are Anthropic's
//! published per-million-token API rates. When Anthropic changes a
//! rate, update the matching `ModelPrice` literal here AND the
//! frozen-golden value in `tests/pricing.rs::cost_for_opus_golden_value`
//! in the same commit.

use crate::state::ModelTokens;

/// Per-token prices in USD for one model's four token buckets.
///
/// Stored as dollars per single token (the published per-million rate
/// divided by 1_000_000) so [`cost_for`] multiplies directly against
/// the raw `i64` token counts in [`ModelTokens`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPrice {
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
}

/// Number of tokens in the per-million-token rate unit.
const PER_MILLION: f64 = 1_000_000.0;

/// Build a [`ModelPrice`] from per-million-token dollar rates.
const fn per_million(input: f64, output: f64, cache_write: f64, cache_read: f64) -> ModelPrice {
    ModelPrice {
        input: input / PER_MILLION,
        output: output / PER_MILLION,
        cache_write: cache_write / PER_MILLION,
        cache_read: cache_read / PER_MILLION,
    }
}

/// Look up per-token USD prices for a model name.
///
/// Returns `Some(ModelPrice)` when the name is a `claude-` model of a
/// known family (Opus / Sonnet / Haiku), detected by family substring
/// so the `[1m]` 1M-context variants match their base family. Returns
/// `None` for any non-`claude-` model and for a `claude-` model whose
/// family is unrecognized — the caller renders an unpriced row rather
/// than a misleading zero.
pub fn price_for(model: &str) -> Option<ModelPrice> {
    if !model.starts_with("claude-") {
        return None;
    }
    if model.contains("opus") {
        return Some(per_million(5.0, 25.0, 6.25, 0.50));
    }
    if model.contains("sonnet") {
        return Some(per_million(3.0, 15.0, 3.75, 0.30));
    }
    if model.contains("haiku") {
        return Some(per_million(1.0, 5.0, 1.25, 0.10));
    }
    None
}

/// Total USD cost for one model's token usage.
///
/// Multiplies each of the four [`ModelTokens`] buckets by its
/// per-token rate and sums. Returns `None` when the model is unpriced
/// (see [`price_for`]) so the caller can propagate the unpriced state
/// through the existing `Option<f64>` cost plumbing.
pub fn cost_for(model: &str, t: &ModelTokens) -> Option<f64> {
    let p = price_for(model)?;
    Some(
        t.input as f64 * p.input
            + t.output as f64 * p.output
            + t.cache_create as f64 * p.cache_write
            + t.cache_read as f64 * p.cache_read,
    )
}
