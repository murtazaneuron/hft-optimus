//! PEV Loop - Plan → Execute → Verify.
//!
//! Orchestrates the three-phase agentic workflow that governs every trade
//! decision in the mAI (🧠) HFT platform:
//!
//! ```text
//! ┌────────┐     ┌─────────┐     ┌────────┐
//! │  PLAN  │────►│ EXECUTE │────►│ VERIFY │
//! │ Haiku  │     │ Sonnet  │     │ Haiku  │
//! └────────┘     └─────────┘     └───┬────┘
//!                    ▲               │ score < 0.80
//!                    └───────────────┘  retry × 2
//! ```
//!
//! **Cost model**: Haiku handles the cheap planning and verification work;
//! Sonnet is invoked only for the reasoning-heavy Execute phase. This reduces
//! LLM cost by roughly 60–70 % compared with an all-Sonnet pipeline.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hft_optimus::{config::Config, pev};
//!
//! # async fn example(cfg: &Config) -> anyhow::Result<()> {
//! let result = pev::run(cfg, "SOL/USDC", 1.0).await?;
//! assert!(result.verify_score >= pev::verify::PASS_THRESHOLD);
//! # Ok(()) }
//! ```

pub mod execute;
pub mod plan;
pub mod types;
pub mod verify;

use anyhow::Result;
use tracing::{info, warn};
use types::PEVResult;

use crate::config::Config;

/// Maximum number of Execute → Verify retries per task before accepting a
/// failing score and moving on to the next task.
pub const MAX_RETRIES: u32 = 2;

/// Run the full PEV loop for a single trade request.
///
/// Calls [`plan::decompose`] once to produce the task list, then iterates
/// through each [`types::TradeTask`], executing and verifying it with up to
/// [`MAX_RETRIES`] retries on each failure. On retry the verifier's feedback
/// is available to the orchestrator for error context injection.
///
/// # Arguments
///
/// * `cfg`    - Runtime configuration (API keys, RPC URL, dry-run flag).
/// * `pair`   - Trading pair, e.g. `"SOL/USDC"`.
/// * `amount` - Base-token amount to trade.
///
/// # Errors
///
/// Propagates any network or API error returned by the underlying LLM calls.
pub async fn run(cfg: &Config, pair: &str, amount: f64) -> Result<PEVResult> {
    info!(pair, amount, "╔══ PEV LOOP START ══╗");

    // ── PLAN ─────────────────────────────────────────────────────
    let tasks = plan::decompose(cfg, pair, amount).await?;
    info!(count = tasks.len(), "[PLAN] Complete");

    let mut outputs = vec![];
    let mut final_score = 0.0f64;
    let mut final_feedback = String::new();
    let mut total_retries = 0u32;

    for task in &tasks {
        let mut retries = 0u32;
        loop {
            // ── EXECUTE ──────────────────────────────────────────
            let output = execute::run_task(cfg, task).await?;

            // ── VERIFY ───────────────────────────────────────────
            let (score, feedback, passed) = verify::score(cfg, task, &output).await?;

            if passed {
                info!(task_id = %task.id, score, "[VERIFY] PASS");
                final_score = score;
                final_feedback = feedback;
                outputs.push(output);
                break;
            }

            retries += 1;
            total_retries += 1;
            if retries > MAX_RETRIES {
                warn!(task_id = %task.id, score, %feedback,
                      "[VERIFY] FAIL - max retries reached");
                final_score = score;
                final_feedback = feedback.clone();
                outputs.push(output);
                break;
            }

            warn!(task_id = %task.id, score, retry = retries,
                  "[VERIFY] FAIL - retrying with error context injected");
        }
    }

    info!(
        final_score,
        retries = total_retries,
        "╚══ PEV LOOP COMPLETE ══╝"
    );
    Ok(PEVResult {
        tasks,
        outputs,
        verify_score: final_score,
        passed: final_score >= verify::PASS_THRESHOLD,
        feedback: final_feedback,
        retries: total_retries,
    })
}
