//! `hft-optimus` - CLI entry point.
//!
//! **mAI (🧠)** | Technology Lead: Murtaza Ali Imtiaz
//!
//! Platform: Rig (Rust Inference Gateway / ARC) · AVM · `SignerContext` · PEV Loop
//!
//! ## Usage
//!
//! ```text
//! # Full pipeline (default) - requires ANTHROPIC_API_KEY
//! cargo run --release -- --mode full --pair SOL/USDC --amount 1.0
//!
//! # Full pipeline in offline/stub mode - no API key needed
//! cargo run --release -- --mode full --skip-llm
//!
//! # Subsystems that never need an API key
//! cargo run --release -- --mode sor
//! cargo run --release -- --mode signer
//! cargo run --release -- --mode reactor
//!
//! # PEV only, stub mode
//! cargo run --release -- --mode pev --skip-llm
//! ```
//!
//! Set `ANTHROPIC_API_KEY` in `.env` or the shell for live LLM execution.
//! Omit it (or pass `--skip-llm`) to run every subsystem in offline/stub mode.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use hft_optimus::{avm, config, onchain, pev, sor};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// CLI operating mode - selects which subsystem(s) to exercise.
#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// Run the full pipeline: PEV → SOR → on-chain swap → AVM audit log.
    Full,
    /// Run only the PEV loop (Plan → Execute → Verify) via rig-core.
    Pev,
    /// Run only the Smart Order Routing venue comparison.  No API key needed.
    Sor,
    /// Run only the `SignerContext` isolation demo.  No API key needed.
    Signer,
    /// Run only the AVM benchmark and Reactor GUI audit log.  No API key needed.
    Reactor,
}

/// CLI arguments parsed by [`clap`].
#[derive(Parser, Debug)]
#[command(name = "hft-optimus")]
#[command(about = "Optimal HFT platform using Rig (ARC) - mAI (🧠)")]
struct Args {
    /// Operating mode (default: `full`).
    #[arg(short, long, default_value = "full")]
    mode: Mode,

    /// Skip all LLM calls and use offline stubs for the PEV phases.
    ///
    /// Implied automatically when `ANTHROPIC_API_KEY` is absent.
    /// Pass this flag explicitly to force stub mode even when a key is set.
    #[arg(long, default_value_t = false)]
    skip_llm: bool,

    /// Enable live on-chain transactions.  Omit to stay in dry-run mode.
    #[arg(long, default_value_t = false)]
    live: bool,

    /// Trading pair forwarded to SOR and PEV (e.g. `SOL/USDC`).
    #[arg(short, long, default_value = "SOL/USDC")]
    pair: String,

    /// Amount of the base token to trade.
    #[arg(short, long, default_value_t = 1.0)]
    amount: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("hft_optimus=debug".parse()?))
        .init();

    let args = Args::parse();

    // Build config; skip_llm is already true when the key is absent,
    // but the CLI flag can force it on even when a key exists.
    let mut cfg = config::Config::from_env()?;
    if args.skip_llm {
        cfg.skip_llm = true;
    }

    info!("╔══════════════════════════════════════════════╗");
    info!("║  mAI RIG HFT  ·  Rig (ARC) Platform  ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(
        mode = ?args.mode,
        pair = %args.pair,
        amount = args.amount,
        live = args.live,
        skip_llm = cfg.skip_llm,
        "Starting platform"
    );

    if cfg.skip_llm {
        info!("⚠  LLM stub mode active - PEV phases will use offline stubs.");
        info!("   Set ANTHROPIC_API_KEY or omit --skip-llm for live LLM execution.");
    }

    match args.mode {
        Mode::Full => {
            // 1. PEV loop - uses stub when skip_llm is true
            let pev_result = pev::run(&cfg, &args.pair, args.amount).await?;
            info!(score = pev_result.verify_score, "PEV loop complete");

            // 2. Smart Order Routing - always live (no API key needed)
            let route = sor::best_route(&args.pair, args.amount).await?;
            info!(
                venue = %route.venue,
                price = route.effective_price,
                fee_bps = route.fee_bps,
                latency_ms = route.latency_ms,
                "SOR: best route selected"
            );

            // 3. On-chain execution (dry-run unless --live was passed)
            let swap_result = onchain::execute_swap(&cfg, &route, args.live).await?;
            info!(tx_sig = %swap_result.simulated_sig, "Swap simulation complete");

            // 4. AVM audit log (Reactor GUI simulation)
            avm::audit_log(&route, &swap_result)?;
        }
        Mode::Pev => {
            pev::run(&cfg, &args.pair, args.amount).await?;
        }
        Mode::Sor => {
            sor::best_route(&args.pair, args.amount).await?;
        }
        Mode::Signer => {
            onchain::demo_signer(&cfg).await?;
        }
        Mode::Reactor => {
            avm::run_benchmark()?;
        }
    }

    info!("Platform run complete. All operations logged.");
    Ok(())
}
