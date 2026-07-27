//! Shared data types for the PEV (Plan → Execute → Verify) loop.
//!
//! These types flow through all three phases:
//!
//! ```text
//! Plan    ──► Vec<TradeTask>
//! Execute ──► Vec<ExecuteOutput>
//! Verify  ──► (score: f64, feedback: String, passed: bool)
//! ```

use serde::{Deserialize, Serialize};

/// A single atomic trade operation produced by the [`crate::pev::plan`] phase.
///
/// The planner (Haiku) decomposes a high-level trade request into exactly four
/// `TradeTask` items, one per [`TradeAction`] variant. Each task carries its
/// own acceptance criteria so the verifier can score it independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTask {
    /// Short task identifier, e.g. `"T001"`.
    pub id: String,

    /// Trading pair in `BASE/QUOTE` notation, e.g. `"SOL/USDC"`.
    pub pair: String,

    /// Amount of the base token to trade.
    pub amount: f64,

    /// The category of work this task requires.
    pub action: TradeAction,

    /// Human-readable criteria the [`crate::pev::execute`] output must satisfy
    /// to pass verification.
    pub acceptance_criteria: Vec<String>,
}

/// Discriminates the four atomic steps in a single trade lifecycle.
///
/// Serialises to and deserialises from `snake_case` strings so that the LLM
/// can produce and consume JSON without capitalisation mismatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeAction {
    /// Fetch and interpret current market data for the pair.
    AnalyseMarket,
    /// Query DEX venues and choose the best execution route.
    SelectRoute,
    /// Confirm that expected slippage is within the allowed tolerance.
    ValidateSlippage,
    /// Perform a dry-run swap and record the simulated result.
    SimulateExecution,
}

/// Output produced by the [`crate::pev::execute`] phase for one [`TradeTask`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOutput {
    /// Matches [`TradeTask::id`] of the task that produced this output.
    pub task_id: String,

    /// Raw text result returned by the Sonnet executor agent.
    pub result: String,

    /// Self-reported confidence score in `[0.0, 1.0]`. In production this is
    /// parsed from the LLM response; in the demo it is fixed at `0.87`.
    pub confidence: f64,

    /// Step-by-step reasoning from the executor (mirrors `result` in the demo;
    /// would be a separate chain-of-thought field in production).
    pub reasoning: String,

    /// Names of tools invoked during execution, e.g.
    /// `"fetch_price_feed(SOL/USDC)"`. Populated by mapping the [`TradeAction`].
    pub tool_calls: Vec<String>,
}

/// Aggregated result returned by [`crate::pev::run`] after a full PEV loop.
#[derive(Debug, Clone)]
pub struct PEVResult {
    /// The tasks originally produced by the Plan phase.
    pub tasks: Vec<TradeTask>,

    /// One output per task, in the same order as [`PEVResult::tasks`].
    pub outputs: Vec<ExecuteOutput>,

    /// Verify score of the *last* task processed, in `[0.00, 1.00]`.
    ///
    /// The pass threshold is [`crate::pev::verify::PASS_THRESHOLD`] (`0.80`).
    pub verify_score: f64,

    /// `true` when `verify_score >= PASS_THRESHOLD`.
    pub passed: bool,

    /// Human-readable verifier feedback for the final task.
    pub feedback: String,

    /// Total retry attempts accumulated across all tasks in this loop run.
    pub retries: u32,
}
