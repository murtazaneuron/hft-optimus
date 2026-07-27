//! Integration tests for the PEV loop types, default task decomposition, stub
//! execution paths, and the verify pass threshold constant.
//!
//! None of these tests make any LLM API calls; they exercise only the
//! deterministic, pure-Rust code paths.  They pass with or without
//! `ANTHROPIC_API_KEY` in the environment.

use hft_optimus::{
    config::Config,
    pev::types::{ExecuteOutput, TradeAction, TradeTask},
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_task() -> TradeTask {
    TradeTask {
        id: "T001".into(),
        pair: "SOL/USDC".into(),
        amount: 1.0,
        action: TradeAction::AnalyseMarket,
        acceptance_criteria: vec!["Market data retrieved".into()],
    }
}

fn make_output(result: &str) -> ExecuteOutput {
    ExecuteOutput {
        task_id: "T001".into(),
        result: result.to_string(),
        confidence: 0.90,
        reasoning: result.to_string(),
        tool_calls: vec!["fetch_price_feed(SOL/USDC)".into()],
    }
}

/// Build a [`Config`] with `skip_llm` forced on, suitable for tests that must
/// not make any network calls regardless of the environment.
fn offline_config() -> Config {
    let mut cfg = Config::from_env().expect("Config::from_env must not fail");
    cfg.skip_llm = true;
    cfg
}

// ── Config tests ──────────────────────────────────────────────────────────────

/// `Config::from_env` must succeed even when `ANTHROPIC_API_KEY` is absent.
#[test]
fn test_config_from_env_succeeds_without_key() {
    // We don't clear the var here because other tests may run in parallel and
    // share the environment.  Instead we verify the function itself is infallible
    // and that skip_llm is consistent with has_api_key().
    let cfg = Config::from_env().expect("Config::from_env must not return Err");
    // skip_llm must be true when there is no key.
    assert_eq!(
        cfg.skip_llm,
        !cfg.has_api_key(),
        "skip_llm must be the logical inverse of has_api_key() \
         when SKIP_LLM env var is not set"
    );
}

/// `has_api_key()` must return `false` when the key is empty.
#[test]
fn test_config_has_api_key_empty() {
    let cfg = Config {
        anthropic_api_key: String::new(),
        skip_llm: true,
        solana_rpc_url: "https://api.devnet.solana.com".into(),
        solana_private_key: "DEMO".into(),
        dry_run: true,
    };
    assert!(
        !cfg.has_api_key(),
        "empty key → has_api_key() must be false"
    );
}

/// `has_api_key()` must return `true` when the key is non-empty.
#[test]
fn test_config_has_api_key_present() {
    let cfg = Config {
        anthropic_api_key: "sk-ant-test".into(),
        skip_llm: false,
        solana_rpc_url: "https://api.devnet.solana.com".into(),
        solana_private_key: "DEMO".into(),
        dry_run: true,
    };
    assert!(
        cfg.has_api_key(),
        "non-empty key → has_api_key() must be true"
    );
}

// ── PEV stub-mode tests ───────────────────────────────────────────────────────

/// The full PEV loop must complete in stub mode (`skip_llm=true`) without any
/// network call and return a passing score.
#[tokio::test]
async fn test_pev_run_stub_mode_passes() {
    let cfg = offline_config();
    let result = hft_optimus::pev::run(&cfg, "SOL/USDC", 1.0)
        .await
        .expect("pev::run must not fail in stub mode");
    assert!(
        result.passed,
        "stub-mode PEV must return passed=true; score={:.2}",
        result.verify_score
    );
    assert_eq!(result.tasks.len(), 4, "stub mode must produce 4 tasks");
    assert_eq!(result.outputs.len(), 4, "stub mode must produce 4 outputs");
}

/// `plan::decompose` must return default tasks immediately in stub mode.
#[tokio::test]
async fn test_plan_decompose_stub_mode() {
    let cfg = offline_config();
    let tasks = hft_optimus::pev::plan::decompose(&cfg, "SOL/USDC", 2.5)
        .await
        .expect("plan::decompose must not fail in stub mode");
    assert_eq!(tasks.len(), 4, "stub decompose must return 4 tasks");
    assert!(
        tasks.iter().all(|t| (t.amount - 2.5).abs() < f64::EPSILON),
        "stub tasks must preserve the amount"
    );
}

/// `verify::score` must return a passing stub score in stub mode.
#[tokio::test]
async fn test_verify_score_stub_mode() {
    let cfg = offline_config();
    let task = make_task();
    let output = make_output("any result");
    let (score, feedback, passed) = hft_optimus::pev::verify::score(&cfg, &task, &output)
        .await
        .expect("verify::score must not fail in stub mode");
    assert!(passed, "stub verify must pass");
    assert!(
        score >= hft_optimus::pev::verify::PASS_THRESHOLD,
        "stub score must be >= PASS_THRESHOLD"
    );
    assert!(
        feedback.contains("stub") || feedback.contains("Stub"),
        "stub feedback must mention stub mode"
    );
}

// ── original unit tests (unchanged) ──────────────────────────────────────────

/// `default_tasks_pub` should produce exactly four tasks.
#[test]
fn test_plan_default_tasks_count() {
    let tasks = hft_optimus::pev::plan::default_tasks_pub("SOL/USDC", 1.0);
    assert_eq!(tasks.len(), 4, "expected 4 default tasks");
}

/// The pass threshold constant must equal `0.80`.
#[test]
fn test_verify_pass_threshold() {
    assert!(
        (hft_optimus::pev::verify::PASS_THRESHOLD - 0.80).abs() < f64::EPSILON,
        "PASS_THRESHOLD must be 0.80"
    );
}

/// A [`TradeTask`] round-trips through JSON without data loss.
#[test]
fn test_trade_task_serialization() {
    let task = make_task();
    let json = serde_json::to_string(&task).unwrap();
    assert!(json.contains("SOL/USDC"));
    assert!(json.contains("analyse_market"));
}

/// [`ExecuteOutput`] correctly stores tool call names.
#[test]
fn test_execute_output_tool_calls() {
    let output = make_output("Market data retrieved: SOL price = $143.50");
    assert!(
        !output.tool_calls.is_empty(),
        "tool_calls must not be empty"
    );
    assert!(
        output.tool_calls[0].contains("fetch_price_feed"),
        "first tool call should reference fetch_price_feed"
    );
}
