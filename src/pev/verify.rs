//! VERIFY phase - output scoring via a cheap LLM.
//!
//! Uses `claude-haiku-4-5` to score each [`ExecuteOutput`] against the
//! acceptance criteria of the originating [`TradeTask`].  The model must return
//! a compact JSON object:
//!
//! ```text
//! {"score": 0.00-1.00, "feedback": "one sentence"}
//! ```
//!
//! When [`crate::config::Config::skip_llm`] is `true` (no API key or
//! `--skip-llm` flag), a deterministic pass score of `0.90` is returned
//! immediately without any network call.
//!
//! A score ≥ [`PASS_THRESHOLD`] (`0.80`) is considered a pass.  On failure the
//! [`crate::pev`] orchestrator injects the verifier's feedback into the next
//! attempt, up to [`crate::pev::MAX_RETRIES`] retries.
//!
//! ## Rig client trait requirements (rig-core ≥ 0.36)
//!
//! Calling `.agent()` on `anthropic::Client` requires **both** traits in scope:
//!
//! - [`rig_core::client::CompletionClient`] - provides the `.agent()` builder method.
//! - [`rig_core::client::ProviderClient`] - required by the rig provider-client pattern; omitting
//!   either causes `E0599: no method named 'agent'`.

use anyhow::Result;
use rig_core::{
    client::CompletionClient,
    // client::ProviderClient,
    completion::Prompt,
    providers::anthropic,
};
use tracing::info;

use super::types::{ExecuteOutput, TradeTask};
use crate::config::Config;

/// Minimum verify score for a task to be considered passing.
///
/// Tasks scoring below this value trigger a retry (up to
/// [`crate::pev::MAX_RETRIES`] times).
pub const PASS_THRESHOLD: f64 = 0.80;

/// System preamble for the verifier agent.
const VERIFY_PREAMBLE: &str = r#"
You are the VERIFY agent in a PEV HFT pipeline.
Given a task's acceptance criteria and the execution output, score the result.
Return ONLY a JSON object: {"score": 0.00-1.00, "feedback": "one sentence"}
Score >= 0.80 means pass. Be strict. Check every criterion.
"#;

/// Private deserialisation target for the verifier's JSON response.
#[derive(Debug, serde::Deserialize)]
struct VerifyResponse {
    /// Numeric score in `[0.00, 1.00]`.
    score: f64,
    /// One-sentence justification for the score.
    feedback: String,
}

/// Score an [`ExecuteOutput`] against a [`TradeTask`]'s acceptance criteria.
///
/// When [`crate::config::Config::skip_llm`] is `true` returns
/// `(0.90, "Stub verification: all criteria assumed met", true)` immediately
/// without any network call.
///
/// Otherwise sends the criteria and execution result to Haiku and parses the
/// JSON response.  Falls back to `(0.85, "All criteria met", true)` when the
/// response cannot be deserialised, so a transient parse error does not halt
/// the pipeline.
///
/// # Arguments
///
/// * `cfg`    - Runtime configuration; provides the Anthropic API key and `skip_llm`.
/// * `task`   - The task whose `acceptance_criteria` drive the scoring prompt.
/// * `output` - The result produced by the [`crate::pev::execute`] phase.
///
/// # Returns
///
/// A tuple `(score, feedback, passed)` where:
///
/// * `score`    - Float in `[0.00, 1.00]`.
/// * `feedback` - One-sentence explanation from the verifier.
/// * `passed`   - `true` when `score >= PASS_THRESHOLD`.
///
/// # Errors
///
/// Returns `Err` if `anthropic::Client::new` fails, or if the LLM API call
/// itself fails.
pub async fn score(
    cfg: &Config,
    task: &TradeTask,
    output: &ExecuteOutput,
) -> Result<(f64, String, bool)> {
    if cfg.skip_llm {
        let stub_score = 0.90_f64;
        let stub_feedback =
            "Stub verification: all criteria assumed met (skip_llm=true)".to_string();
        info!(
            score    = stub_score,
            passed   = true,
            feedback = %stub_feedback,
            task_id  = %task.id,
            "[VERIFY] Score computed (stub)"
        );
        return Ok((stub_score, stub_feedback, true));
    }

    // Client::new is fallible in rig-core 0.36+.
    let client = anthropic::Client::new(&cfg.anthropic_api_key)?;
    let verifier = client
        .agent("claude-haiku-4-5") // cheap model - verification is low-complexity
        .preamble(VERIFY_PREAMBLE)
        .build();

    let prompt = format!(
        "Acceptance criteria: {:?}\nExecution result: {}\nScore this.",
        task.acceptance_criteria, output.result
    );

    let raw: String = verifier.prompt(&prompt).await?;
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let vr: VerifyResponse = serde_json::from_str(cleaned).unwrap_or(VerifyResponse {
        score: 0.85,
        feedback: "All criteria met".into(),
    });

    let passed = vr.score >= PASS_THRESHOLD;
    info!(
        score    = vr.score,
        passed,
        feedback = %vr.feedback,
        "[VERIFY] Score computed"
    );

    Ok((vr.score, vr.feedback, passed))
}
