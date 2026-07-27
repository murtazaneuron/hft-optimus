//! EXECUTE phase - agentic task execution via Sonnet.
//!
//! Each [`TradeTask`] is handed to a `claude-sonnet-4-6` agent that reasons
//! step-by-step and invokes the appropriate tool.  Tool calls are simulated in
//! this demo; in production they would be real [`rig_core::tool::Tool`]
//! implementations backed by live market data APIs.
//!
//! When [`crate::config::Config::skip_llm`] is `true` (no API key or
//! `--skip-llm` flag), the function returns a deterministic stub
//! [`ExecuteOutput`] without any network call.
//!
//! Sonnet is the most capable - and most expensive - model in the PEV
//! pipeline.  It is deliberately confined to the Execute phase only; Plan and
//! Verify use Haiku to keep overall LLM costs down.
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
use tracing::{debug, info};

use super::types::{ExecuteOutput, TradeAction, TradeTask};
use crate::config::Config;

/// System preamble for the executor agent.
///
/// Instructs Sonnet to think step-by-step, invoke the right tool, and return
/// a concise result string that the Verify phase can score against acceptance
/// criteria.
const EXECUTE_PREAMBLE: &str = r"
You are the EXECUTE agent in a PEV HFT pipeline running on Rig (ARC).
You receive a single TradeTask and must complete it using available tools.
Think step-by-step. Call the appropriate tool. Return a concise result string.
";

/// Execute a single [`TradeTask`] using the Sonnet agent.
///
/// When [`crate::config::Config::skip_llm`] is `true` returns a deterministic
/// stub output immediately with no network call.
///
/// Otherwise builds a structured prompt from the task fields, sends it to
/// `claude-sonnet-4-6`, and maps the [`TradeAction`] variant to a list of
/// simulated tool-call names for the audit log.
///
/// # Arguments
///
/// * `cfg`  - Runtime configuration; provides the Anthropic API key and `skip_llm`.
/// * `task` - The atomic task to execute.
///
/// # Errors
///
/// Returns `Err` if `anthropic::Client::new` fails, or if the LLM API call
/// itself fails (network error, authentication error, rate-limit).
pub async fn run_task(cfg: &Config, task: &TradeTask) -> Result<ExecuteOutput> {
    info!(task_id = %task.id, action = ?task.action, "[EXECUTE] Running task");

    if cfg.skip_llm {
        info!(task_id = %task.id, "[EXECUTE] skip_llm=true - returning stub output");
        return Ok(stub_output(task));
    }

    // Client::new is fallible in rig-core 0.36+.
    // Sonnet is chosen for the Execute phase: best reasoning + tool-use capability.
    let client = anthropic::Client::new(&cfg.anthropic_api_key)?;
    let executor = client
        .agent("claude-sonnet-4-6")
        .preamble(EXECUTE_PREAMBLE)
        .build();

    let prompt = format!(
        "Task ID: {}\nAction: {:?}\nPair: {}\nAmount: {} SOL\n\
         Acceptance criteria: {:?}\nExecute this task now.",
        task.id, task.action, task.pair, task.amount, task.acceptance_criteria
    );

    let response: String = executor.prompt(&prompt).await?;
    debug!(raw = %response, task_id = %task.id, "[EXECUTE] Raw response");

    // Map each action to its canonical tool call for the audit log.
    let tool_calls = action_tool_calls(&task.action);
    for tool in &tool_calls {
        info!(tool = %tool, "[EXECUTE] Tool called");
    }

    Ok(ExecuteOutput {
        task_id: task.id.clone(),
        result: response.clone(),
        confidence: 0.87, // fixed in demo; production: parse from LLM or compute
        reasoning: response,
        tool_calls,
    })
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a deterministic stub [`ExecuteOutput`] for offline / no-key runs.
fn stub_output(task: &TradeTask) -> ExecuteOutput {
    let result = format!(
        "[STUB] Task {} ({:?}) executed offline. Pair: {}, Amount: {}",
        task.id, task.action, task.pair, task.amount
    );
    let tool_calls = action_tool_calls(&task.action);
    for tool in &tool_calls {
        info!(tool = %tool, "[EXECUTE] Tool called (stub)");
    }
    ExecuteOutput {
        task_id: task.id.clone(),
        result: result.clone(),
        confidence: 0.90,
        reasoning: result,
        tool_calls,
    }
}

/// Map a [`TradeAction`] to its canonical tool-call name(s).
fn action_tool_calls(action: &TradeAction) -> Vec<String> {
    match action {
        TradeAction::AnalyseMarket => vec!["fetch_price_feed(SOL/USDC)".into()],
        TradeAction::SelectRoute => {
            vec!["query_raydium_pool()".into(), "query_orca_pool()".into()]
        }
        TradeAction::ValidateSlippage => vec!["calculate_slippage(amount)".into()],
        TradeAction::SimulateExecution => vec!["jupiter_swap_dry_run()".into()],
    }
}
