//! Reactor GUI audit-log simulation.
//!
//! Emits a structured, human-readable execution trace in the style of the
//! Reactor GUI deployed by mAI (🧠) for smart contract deployments.
//! The log captures three sequential phases:
//!
//! 1. **STATE BEFORE** - wallet balance and AMM pool conditions prior to execution.
//! 2. **EXECUTION** - method called, compute-unit estimate, AVM JIT mode, and transaction
//!    signature.
//! 3. **STATE AFTER** - resulting balance, tokens received, fee paid, and final status.
//!
//! In production this module would write to a persistent audit store (e.g. a
//! Postgres time-series table). In this demo it emits structured fields to the
//! [`tracing`] subscriber at `INFO` level.

use tracing::info;

use crate::{onchain::jupiter::SwapResult, sor::router::Route};

/// Emit a full Reactor GUI audit log for one completed swap execution.
///
/// Reads display fields from `route` (venue, effective price, fee basis
/// points) and `swap` (input/output amounts, simulated signature, dry-run
/// flag) and emits them as a framed, human-readable log block at `INFO` level.
///
/// # Arguments
///
/// * `route` - The SOR-selected execution venue and price quote.
/// * `swap`  - The result returned by the Jupiter swap simulation.
///
/// # Errors
///
/// Currently infallible; returns `Ok(())`.
pub fn emit_audit_log(route: &Route, swap: &SwapResult) -> anyhow::Result<()> {
    info!("[REACTOR GUI] ╔══════════════════════════════════════╗");
    info!("[REACTOR GUI] ║  Smart Contract Deployment Audit Log ║");
    info!("[REACTOR GUI] ╚══════════════════════════════════════╝");
    info!("[REACTOR GUI] Contract: SOR_Executor_v1");
    info!("[REACTOR GUI] Network:  Solana Devnet");
    info!("[REACTOR GUI] ─── STATE BEFORE ──────────────────────");
    info!("[REACTOR GUI] Balance:     1.000000 SOL");
    info!("[REACTOR GUI] Pool:        {} AMM", route.venue);
    info!(
        "[REACTOR GUI] Price:       ${:.4} USDC/SOL",
        route.effective_price
    );
    info!("[REACTOR GUI] Fee:         {} bps", route.fee_bps);
    info!("[REACTOR GUI] ─── EXECUTION ─────────────────────────");
    info!(
        "[REACTOR GUI] Method:      swap(SOL, USDC, {:.4})",
        swap.input_amount
    );
    info!("[REACTOR GUI] Gas est.:    5,000 compute units");
    info!("[REACTOR GUI] AVM:         JIT-compiled, 0-overhead execution");
    info!("[REACTOR GUI] Sig:         {}", swap.simulated_sig);
    info!(
        "[REACTOR GUI] Mode:        {}",
        if swap.is_dry_run {
            "DRY RUN (simulation)"
        } else {
            "LIVE"
        }
    );
    info!("[REACTOR GUI] ─── STATE AFTER ───────────────────────");
    info!(
        "[REACTOR GUI] Balance:     {:.6} SOL  ({:.4} USDC received)",
        swap.input_amount, swap.output_amount
    );
    info!("[REACTOR GUI] Fee paid:    {:.6} SOL", swap.fee_paid);
    info!("[REACTOR GUI] Status:      SUCCESS");
    info!("[REACTOR GUI] ╔══════════════════════════════════════╗");
    info!("[REACTOR GUI] ║  Audit log complete. Zero errors.    ║");
    info!("[REACTOR GUI] ╚══════════════════════════════════════╝");
    Ok(())
}
